# Justfile for zenoh-nostd project

check:
    cargo clippy -p zenoh-nostd
    cargo clippy -p zenoh-nostd --features=log
    cargo clippy -p zenoh-nostd --features=defmt
    cargo clippy -p zenoh-nostd --features=web_console

    cd platforms/zenoh-std && just check
    cd platforms/zenoh-wasm && just check
    cd platforms/zenoh-embassy && just check

    cargo clippy --examples --features=std,log
    cargo clippy --examples --no-default-features --features=wasm,web_console --target wasm32-unknown-unknown
    WIFI_PASSWORD=* cargo +esp --config .cargo/config.esp32s3.toml check --examples --no-default-features --features=esp32s3,defmt

fix:
    cargo clippy -p zenoh-nostd --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=log --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=defmt --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=web_console --fix --lib --allow-dirty --allow-staged

    cd platforms/zenoh-std && just fix
    cd platforms/zenoh-wasm && just fix
    cd platforms/zenoh-embassy && just fix

    cargo clippy --examples --features=std,log --fix --lib --allow-dirty --allow-staged
    cargo clippy --examples --no-default-features --features=wasm,web_console --target wasm32-unknown-unknown --fix --lib --allow-dirty --allow-staged
    WIFI_PASSWORD=* cargo +esp --config .cargo/config.esp32s3.toml fix --examples --no-default-features --features=esp32s3,defmt --lib --allow-dirty --allow-staged

# Code statistics

loc-proto:
    tokei --files crates/zenoh-proto crates/zenoh-derive/src/codec* --exclude crates/zenoh-proto/src/tests*

# Tests and benches

test filter="":
    cargo test {{ filter }} -p zenoh-proto --features=alloc

bench filter="bench":
    cargo test -p zenoh-proto {{ filter }} --features=alloc --profile=release -- --nocapture --ignored --test-threads=1

# Special `std` examples

flood:
    cargo run -p zenoh-proto --release --features=std,log --example z_flood

drain:
    cargo run -p zenoh-proto --release --features=std,log --example z_drain

ping:
    RUST_LOG=trace cargo run --release --features=std,log --example z_ping

pong:
    RUST_LOG=trace cargo run --release --features=std,log --example z_pong

# Examples

std example:
    RUST_LOG=trace cargo run --example {{ example }} --features="std,log"

esp32s3 example:
    cargo +esp --config .cargo/config.esp32s3.toml run --example {{ example }} --no-default-features --features=esp32s3,defmt

sansio example:
    RUST_LOG=debug cargo run -p zenoh-sansio --example {{ example }} --features="log"

wasm example *args:
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --example {{ example }} --no-default-features --features="wasm,web_console" --target wasm32-unknown-unknown -- {{ args }}
    wasm-bindgen --target web --out-dir ./examples/web/ ./target/wasm32-unknown-unknown/debug/examples/{{ example }}.wasm --out-name z_example
    basic-http-server ./examples/web
