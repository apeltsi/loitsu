cargo build --target wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/debug/loitsu-editor.wasm --target web --out-dir web/public/wasm
cd web
pnpm run build
cd ../server
cargo build
cd ..
