#!/bin/sh -e
RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --no-typescript --no-pack --target=web --dev
python3 -m http.server 3000