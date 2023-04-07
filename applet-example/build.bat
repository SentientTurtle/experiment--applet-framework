cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --no-typescript --weak-refs --out-dir "./js-out" --out-name "applet" "./target/wasm32-unknown-unknown/release/applet_example.wasm"
wasm-gc "./js-out/applet_bg.wasm" "./js-out/applet_bg.wasm"
