cargo build --target wasm32-unknown-unknown --release
wasm-bindgen ./target/wasm32-unknown-unknown/release/loitsu-editor.wasm --target web --out-dir web/public/wasm
