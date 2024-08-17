import * as wasm from "../create-wasm-app/pkg/hello_wgpu";
wasm.then(() => console.log("WASM Loaded"));
