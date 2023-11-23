importScripts("worker.js")
this.onmessage = async event => {
    console.log("OnMessage");
    // event.data[0] should be the Memory object, and event.data[1] is the value to pass into entry_point
    const { entry_point } = await wasm_bindgen(
        "worker_bg.wasm",
        event.data[0]
    );

    console.log("OnMessage: entry reached");
    entry_point(Number(event.data[1]));
    console.log("OnMessage: entry passed");
}
