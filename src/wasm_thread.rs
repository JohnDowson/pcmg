use wasm_bindgen::JsValue;

pub fn spawn(f: impl FnOnce() + Send + 'static) -> Result<web_sys::Worker, JsValue> {
    let mut options = web_sys::WorkerOptions::new();
    options.type_(web_sys::WorkerType::Classic);
    let worker =
        web_sys::Worker::new_with_options("ww.js", &options).expect("Worker failed to start");
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));
    let msg: js_sys::Array = [
        &wasm_bindgen::module(),
        &wasm_bindgen::memory(),
        &JsValue::from(ptr as u32),
    ]
    .into_iter()
    .collect();
    if let Err(e) = worker.post_message(&msg) {
        let _ = unsafe { Box::from_raw(ptr) };
        Err(format!("Error initializing worker during post_message: {:?}", e).into())
    } else {
        Ok(worker)
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn wasm_thread_entry_point(addr: u32) {
    console_error_panic_hook::set_once();
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}
