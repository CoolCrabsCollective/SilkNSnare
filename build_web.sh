# crate_name=cargo tree --depth 0 | awk '{print $1;}'
rm -rf ./out
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web target/wasm32-unknown-unknown/release/ld56.wasm
cp index.html ./out
mkdir ./out/assets/
cp -r ./assets/* ./out/assets/
zip -rq out.zip out
