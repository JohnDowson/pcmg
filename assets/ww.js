importScripts('graph.js')

this.onmessage = async event => {
    const { wasm_thread_entry_point } = await wasm_bindgen(event.data[0], event.data[1]).catch(err => {
        setTimeout(() => {
            throw err;
        });
        throw err;
    });

    wasm_thread_entry_point(event.data[2]);
    // init.__wbindgen_thread_destroy();
    // close();
};
