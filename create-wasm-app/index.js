import * as wasm from "./pkg/hello_wgpu";
wasm.run().then(() => console.log("WASM Loaded"));
