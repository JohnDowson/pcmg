use wasm_bindgen::JsValue;

pub fn spawn(f: impl FnOnce() + Send + 'static) -> Result<web_sys::Worker, JsValue> {
    let worker = web_sys::Worker::new("./ww.js").expect("Worker failed to start");
    // Double-boxing because `dyn FnOnce` is unsized and so `Box<dyn FnOnce()>` is a fat pointer.
    // But `Box<Box<dyn FnOnce()>>` is just a plain pointer, and since wasm has 32-bit pointers,
    // we can cast it to a `u32` and back.
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));
    let msg = js_sys::Array::new();
    // Send the worker a reference to our memory chunk, so it can initialize a wasm module
    // using the same memory.
    msg.push(&wasm_bindgen::memory());
    // Also send the worker the address of the closure we want to execute.
    msg.push(&JsValue::from(ptr as u32));
    worker.post_message(&msg).expect("Worker failed to start");
    Ok(worker)
}

#[wasm_bindgen::prelude::wasm_bindgen]
// This function is here for `worker.js` to call.
pub fn wasm_thread_entry_point(addr: u32) {
    console_error_panic_hook::set_once();
    // Interpret the address we were given as a pointer to a closure to call.
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}
