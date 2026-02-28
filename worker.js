import init, { worker_handle_msg } from './pkg/puzzle_explorer.js';

self.onmessage = async event => {
    try {
        await init();

        const result = worker_handle_msg(event.data);

        self.postMessage({ type: 'success', result });
    } catch (e) {
        console.error("Worker error:", e);
        self.postMessage({ type: 'error', error: e.toString() });
    }
};
