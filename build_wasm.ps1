# Build wasm binary
cargo build --lib --target wasm32-unknown-unknown --release

# Copy to web folder
Copy-Item -Path "./target/wasm32-unknown-unknown/release/customasm.wasm" -Destination "./web/customasm.wasm"

