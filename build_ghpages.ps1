# Build wasm binary
cargo build --lib --target wasm32-unknown-unknown --release

# Reduce binary size
wasm-gc "./target/wasm32-unknown-unknown/release/customasm.wasm" -o "./target/wasm32-unknown-unknown/release/customasm.gc.wasm"

# Copy to web folder
Copy-Item -Path "./target/wasm32-unknown-unknown/release/customasm.gc.wasm" -Destination "./web/customasm.gc.wasm"

# Commit to git
git add -A
git commit -m "build GitHub Pages"
git push -f origin main