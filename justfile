check:
    cargo clippy -p zenoh-nostd --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=log --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=defmt --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=web_console --fix --lib --allow-dirty --allow-staged

    cd platforms/zenoh-std && just check
    cd platforms/zenoh-embassy && just check
    cd platforms/zenoh-wasm && just check

    cargo clippy --examples --features=std,log --fix --lib --allow-dirty --allow-staged
    cargo clippy --examples --no-default-features --features=wasm,web_console --target wasm32-unknown-unknown

test:
    cargo test -p zenoh-proto

bench:
    cargo test -p zenoh-proto bench --profile=release -- --nocapture --ignored

std example:
    RUST_LOG=debug cargo run --example {{example}} --features="std,log"

esp32s3 example:
    cargo +esp --config .cargo/config.esp32s3.toml run --example {{example}} --no-default-features --features="esp32s3,defmt"

sansio example:
    RUST_LOG=debug cargo run -p zenoh-sansio --example {{example}} --features="log"

wasm example *args:
    RUST_LOG='debug' RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --example {{example}} --no-default-features --features=wasm --target wasm32-unknown-unknown -- {{args}}
    wasm-bindgen --target web --out-dir ./examples/web/ ./target/wasm32-unknown-unknown/debug/examples/{{example}}.wasm --out-name z_example
    basic-http-server ./examples/web
