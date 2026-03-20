use std::rc::Rc;
use std::{cell::RefCell, collections::VecDeque};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{MessageEvent, Worker, WorkerOptions};

use puzzle_explorer_math::canon::OrbitCanonizer;

pub enum DreadnautJob {
    Orbit(OrbitCanonizer),
    Error(String),
}

impl DreadnautJob {
    pub fn generate_script(&mut self) -> Result<String, String> {
        match self {
            DreadnautJob::Orbit(canonizer) => canonizer.generate_script(),
            DreadnautJob::Error(e) => Err(e.clone()),
        }
    }

    pub fn process_script_result(&mut self, s: &str) -> Result<(), String> {
        match self {
            DreadnautJob::Orbit(canonizer) => canonizer.process_script_result(s),
            DreadnautJob::Error(e) => Err(e.clone()),
        }
    }

    #[cfg(test)]
    pub fn unwrap_orbit(self) -> OrbitCanonizer {
        match self {
            DreadnautJob::Orbit(canonizer) => canonizer,
            DreadnautJob::Error(e) => panic!("unwrap_orbit called on error: {}", e),
            //_ => panic!("unwrap_orbit called on non-orbit job"),
        }
    }
}

pub struct DreadnautManager {
    pub worker: Option<Worker>,
    pub task_start_time: Option<f64>,
    pub is_computing: bool,
    pub queue: VecDeque<(usize, DreadnautJob)>,
    pub completed_jobs: Vec<(usize, DreadnautJob)>,
    pub pending_responses: Rc<RefCell<Vec<String>>>,
    wakeup: Rc<dyn Fn()>,
    cur_result: Rc<RefCell<Option<String>>>,
    _on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    _on_error: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl DreadnautManager {
    pub fn new(wakeup: impl Fn() + 'static) -> Self {
        Self {
            worker: None,
            task_start_time: None,
            is_computing: false,
            queue: VecDeque::new(),
            completed_jobs: Vec::new(),
            pending_responses: Rc::new(RefCell::new(Vec::new())),
            wakeup: Rc::new(wakeup),
            cur_result: Rc::new(RefCell::new(None)),
            _on_message: None,
            _on_error: None,
        }
    }

    pub fn init(&mut self) {
        if self.worker.is_some() {
            return;
        }
        let options = WorkerOptions::new();
        let _ = js_sys::Reflect::set(&options, &"type".into(), &"module".into());

        if let Ok(w) = Worker::new_with_options("./dreadnaut/dreadnaut-worker.js", &options) {
            let response_clone = self.pending_responses.clone();
            let cur_result_clone = self.cur_result.clone();
            let wakeup = self.wakeup.clone();
            let on_msg = Closure::wrap(Box::new(move |e: MessageEvent| {
                let mut pushed = 0;
                if let Ok(data) = e.data().dyn_into::<js_sys::Object>()
                    && let Ok(type_val) = js_sys::Reflect::get(&data, &"type".into())
                    && type_val.as_string().as_deref() == Some("output")
                    && let Ok(res_val) = js_sys::Reflect::get(&data, &"data".into())
                    && let Some(s) = res_val.as_string()
                {
                    for line in s.split('\n') {
                        if line.ends_with("START") {
                            cur_result_clone.borrow_mut().replace(String::new());
                        } else if let Some(mut cur_result) = cur_result_clone.take() {
                            if line.ends_with("END") {
                                response_clone.borrow_mut().push(cur_result);
                                pushed += 1;
                            } else {
                                let mut trimmed_line = line;
                                while trimmed_line.starts_with("> ") {
                                    trimmed_line = &trimmed_line[2..];
                                }
                                cur_result.push_str(trimmed_line);
                                cur_result.push('\n');
                                cur_result_clone.replace(Some(cur_result));
                            }
                        }
                    }
                }
                if pushed > 0 {
                    (wakeup.as_ref())();
                }
            }) as Box<dyn FnMut(_)>);
            w.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));

            let on_err = Closure::wrap(Box::new(move |_e: MessageEvent| {
                // Ignore for now
            }) as Box<dyn FnMut(_)>);
            w.set_onerror(Some(on_err.as_ref().unchecked_ref()));

            self._on_message = Some(on_msg);
            self._on_error = Some(on_err);
            self.worker = Some(w);
        }
    }

    pub fn enqueue_batch(&mut self, jobs: Vec<(usize, DreadnautJob)>) {
        // Enqueue a batch of scripts with unique IDs
        let mut full_script = String::new();

        for (request_id, mut job) in jobs {
            let (job, script) = match job.generate_script() {
                Ok(script) => (job, script),
                Err(e) => (DreadnautJob::Error(e), String::new()),
            };
            full_script.push_str(format!("\"START\\n\"\n{}\"END\\n\"\n", script).as_str());
            self.queue.push_back((request_id, job));
        }

        if !full_script.is_empty()
            && let Some(w) = &self.worker
        {
            let msg = js_sys::Object::new();
            js_sys::Reflect::set(&msg, &"type".into(), &"command".into()).unwrap();
            js_sys::Reflect::set(&msg, &"data".into(), &full_script.into()).unwrap();
            let _ = w.post_message(&msg);
            self.is_computing = true;
            self.task_start_time = Some(crate::time::now());
        }
    }

    pub fn process_responses(&mut self) {
        let mut new_responses = Vec::new();
        if let Ok(mut pending) = self.pending_responses.try_borrow_mut() {
            new_responses.extend(pending.drain(..));
        }

        if !new_responses.is_empty() {
            for res in new_responses {
                if let Some((request_id, mut job)) = self.queue.pop_front() {
                    let job = match job.process_script_result(&res) {
                        Ok(_) => job,
                        Err(e) => DreadnautJob::Error(e),
                    };
                    self.completed_jobs.push((request_id, job));
                }
            }
            if self.queue.is_empty() {
                self.is_computing = false;
                self.task_start_time = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::wrap_promise_in_timeout;

    use super::*;
    use puzzle_explorer_math::generator;
    use std::collections::HashSet;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    macro_rules! test_log {
        ($($arg:tt)*) => {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!($($arg)*)));
        };
    }

    /// Dev/debug test in browser
    /// Remove #[ignore] and run with:
    ///   wasm-pack test --chrome --headless . -- test_dev
    #[ignore]
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn test_dev() {
        let mut dreadnaut_test = DreadnautTest::new();
        {
            let canonizer = OrbitCanonizer::new(
                &generator::parse_gap_string(
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(5,6,10,9),(4,5,9,8)]",
                )
                .unwrap(),
            );
            dreadnaut_test.enqueue_job(DreadnautJob::Orbit(canonizer));
            let canonizer = dreadnaut_test.await_result().await.unwrap().unwrap_orbit();
            test_log!("{}", canonizer.get_canonical_graph_as_string());
            test_log!("{}", canonizer.get_hash());
        }
        {
            let canonizer = OrbitCanonizer::new(
                &generator::parse_gap_string(
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(5,6,10,9),(8,9,5,4)]",
                )
                .unwrap(),
            );
            dreadnaut_test.enqueue_job(DreadnautJob::Orbit(canonizer));
            let canonizer = dreadnaut_test.await_result().await.unwrap().unwrap_orbit();
            test_log!("{}", canonizer.get_canonical_graph_as_string());
            test_log!("{}", canonizer.get_hash());
        }
        panic!("force output");
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn test_canonized_orbits_consistency() {
        let test_pairs = vec![
            (
                "[(6,12,18)(24,28,26),(6,21,24)(8,29,12)]",
                "[o1 a7b74020 9bc23d37 7f67ff53]",
            ),
            (
                "[(14,26)(30,52)(38,47)(62,67),(4,52,62)(14,58,18)(26,38,30)]",
                "[o1 3ba5a1a4 f3f5cde7 15ecf125]",
            ),
            (
                "[(5,17)(35,46)(42,51)(61,66),(5,35,42)(13,57,17)(27,61,46)]",
                "[o1 3ba5a1a4 f3f5cde7 15ecf125]",
            ),
        ];
        let mut dreadnaut_test = DreadnautTest::new();

        for (generator, expected) in test_pairs {
            let canonizer = OrbitCanonizer::new(&generator::parse_gap_string(generator).unwrap());
            dreadnaut_test.enqueue_job(DreadnautJob::Orbit(canonizer));
            let canonizer = dreadnaut_test.await_result().await.unwrap().unwrap_orbit();
            assert_eq!(canonizer.get_hash(), expected);
        }
    }

    /// Check some direction and overlap cases
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn test_canonized_orbits_collision() {
        let mut dreadnaut_test = DreadnautTest::new();
        let tests = vec![
            (false, ["[(2,3)(5,6),(5,6)(7,8)]", "[(2,3)(5,6),(7,8)]"]),
            (true, ["[(1,2,3)]", "[(3,2,1)]"]),
            (true, ["[(1,2,3),(4,3,2)]", "[(3,2,1),(2,3,4)]"]),
            (true, ["[(1,2)]", "[(2,1),(1,2)]"]),
            (
                false, // Quaternion cube-like construction with one direction flipped
                [
                    "[(1,2,8)(6,5,4),(2,4,3)(7,6,8)]",
                    "[(2,1,8)(6,5,4),(2,4,3)(7,6,8)]",
                ],
            ),
            (
                false,
                [
                    // Magic cogra, flipping one direction
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(5,6,10,9)]",
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(9,10,6,5)]",
                ],
            ),
            (
                true,
                [
                    // Magic cogra extra cell, flipping middle cell is isomorphic
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(5,6,10,9),(4,5,9,8)]",
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(5,6,10,9),(8,9,5,4)]",
                ],
            ),
            (
                true,
                [
                    // Magic cogra, flipping two cells is isomorphic
                    "[(1,2,5,4)(8,9,12,11),(3,4,8,7)(5,6,10,9)]",
                    "[(1,2,5,4)(8,9,12,11),(7,8,4,3)(9,10,6,5)]",
                ],
            ),
        ];

        for (should_collide, generators) in tests {
            let mut results = Vec::new();
            for generator in generators {
                let canonizer =
                    OrbitCanonizer::new(&generator::parse_gap_string(generator).unwrap());
                dreadnaut_test.enqueue_job(DreadnautJob::Orbit(canonizer));
                let canonizer = dreadnaut_test.await_result().await.unwrap().unwrap_orbit();
                results.push(canonizer.get_hash());
            }

            let results_len = results.len();
            let results_unique = results.into_iter().collect::<HashSet<_>>().len();

            #[allow(clippy::collapsible_else_if)]
            if should_collide {
                if results_len == results_unique {
                    panic!("expected collision for {:?}", generators);
                }
            } else {
                if results_len != results_unique {
                    panic!("expected no collision for {:?}", generators);
                }
            }
        }
    }

    struct DreadnautTest {
        dreadnaut_manager: DreadnautManager,
        resolve_holder: Rc<RefCell<Option<js_sys::Function>>>,
        promise: Option<js_sys::Promise>,
    }

    impl DreadnautTest {
        fn new() -> Self {
            let resolve_holder: Rc<RefCell<Option<js_sys::Function>>> = Rc::new(RefCell::new(None));
            let promise = Self::new_promise(&resolve_holder);

            let resolve_holder_wakeup = resolve_holder.clone();
            let mut dreadnaut_manager = DreadnautManager::new(move || {
                if let Some(resolve) = resolve_holder_wakeup.borrow_mut().take() {
                    let _ = resolve.call0(&JsValue::NULL);
                }
            });
            dreadnaut_manager.init();
            Self {
                dreadnaut_manager,
                resolve_holder,
                promise: Some(promise),
            }
        }

        fn new_promise(resolve_holder: &Rc<RefCell<Option<js_sys::Function>>>) -> js_sys::Promise {
            let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                *resolve_holder.borrow_mut() = Some(resolve);
            });

            wrap_promise_in_timeout(1000, promise)
        }

        fn enqueue_job(&mut self, job: DreadnautJob) {
            self.dreadnaut_manager.enqueue_batch(vec![(0, job)]);
        }

        async fn await_result(&mut self) -> Result<DreadnautJob, String> {
            let promise = std::mem::take(&mut self.promise).unwrap();
            JsFuture::from(promise)
                .await
                .map_err(|_e| "worker failed to complete")?;
            self.dreadnaut_manager.process_responses();
            self.promise = Some(Self::new_promise(&self.resolve_holder));

            match (
                self.dreadnaut_manager.completed_jobs.len(),
                self.dreadnaut_manager.completed_jobs.first(),
            ) {
                (1, Some((0, _res))) => Ok(self.dreadnaut_manager.completed_jobs.remove(0).1),
                (n, _) => Err(format!("unexpected number of completed jobs {}", n)),
            }
        }
    }
}
