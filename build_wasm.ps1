# Build wasm binary
cargo +nightly build --lib --target wasm32-unknown-unknown --release

# Reduce binary size
wasm-gc "./target/wasm32-unknown-unknown/release/customasm.wasm" -o "./target/wasm32-unknown-unknown/release/customasm.gc.wasm"

# Copy to web folder
Copy-Item -Path "./target/wasm32-unknown-unknown/release/customasm.gc.wasm" -Destination "./web/customasm.gc.wasm"

