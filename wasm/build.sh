cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen ../target/wasm32-unknown-unknown/release/nes_rust_wasm.wasm --out-dir ./web/ --target web --no-typescript
wasm-bindgen ../target/wasm32-unknown-unknown/release/nes_rust_wasm.wasm --out-dir ./npm/nodejs --nodejs
wasm-bindgen ../target/wasm32-unknown-unknown/release/nes_rust_wasm.wasm --out-dir ./npm/pkg
