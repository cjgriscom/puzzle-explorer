pub fn now() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;

        window().unwrap().performance().unwrap().now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}
