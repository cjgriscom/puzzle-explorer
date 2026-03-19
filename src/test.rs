//! Test utilities

pub fn wrap_promise_in_timeout(time_ms: i32, promise: js_sys::Promise) -> js_sys::Promise {
    use wasm_bindgen::closure::ScopedClosure;
    use wasm_bindgen::{JsCast, JsValue};

    // Promise that rejects after timeout
    let timeout_promise = js_sys::Promise::new(&mut |_, reject| {
        let reject = reject.clone();
        let closure = ScopedClosure::once(move || {
            let err = JsValue::from_str("timeout");
            let _ = reject.call1(&JsValue::NULL, &err);
        });
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                time_ms,
            )
            .expect("failed to set timeout");

        // Transfer ownership to JS so rust doesn't drop the closure
        closure.forget();
    });

    // Race the main promise vs timeout
    let race_array = js_sys::Array::new();
    race_array.push(&promise);
    race_array.push(&timeout_promise);

    js_sys::Promise::race(&race_array)
}
