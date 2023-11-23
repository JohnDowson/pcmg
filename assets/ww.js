importScripts("graph.js")
this.onmessage = async event => {
    // event.data[0] should be the Memory object, and event.data[1] is the value to pass into entry_point
    const { wasm_thread_entry_point } = await wasm_bindgen(
        "graph_bg.wasm",
        event.data[0]
    );

    console.log("OnMessage: entry " + wasm_thread_entry_point + " reached");
    wasm_thread_entry_point(Number(event.data[1]));
    console.log("OnMessage: entry passed");
}
