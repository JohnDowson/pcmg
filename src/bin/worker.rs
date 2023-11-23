#![no_main]

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
// This function is here for `worker.js` to call.
pub fn entry_point(addr: u32) {
    console_error_panic_hook::set_once();
    panic!("From worker");
    // Interpret the address we were given as a pointer to a closure to call.
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn _start() {
    console_error_panic_hook::set_once();
    panic!("From worker main");
}
