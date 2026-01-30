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
    cargo clippy --target wasm32-unknown-unknown --examples --no-default-features --features=wasm,web_console
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
    cargo test {{ filter }} -p zenoh-proto -p zenoh-sansio

bench filter="bench":
    cargo test -p zenoh-proto {{ filter }} --profile=release -- --nocapture --ignored --test-threads=1

# Special `std` examples

flood *args:
    cargo run -p zenoh-sansio --release --no-default-features --features=std,log --example z_flood -- {{ args }}

drain *args:
    cargo run -p zenoh-sansio --release --no-default-features --features=std,log --example z_drain -- {{ args }}

ping:
    RUST_LOG=trace cargo run --release --no-default-features --features=std,log --example z_ping

pong:
    RUST_LOG=trace cargo run --release --no-default-features --features=std,log --example z_pong

pub_thr:
    RUST_LOG=trace cargo run --release --no-default-features --features=std,log --example z_pub_thr

sub_thr:
    RUST_LOG=trace cargo run --release --no-default-features --features=std,log --example z_sub_thr

# Examples

esp32s3 example:
    cargo +esp --config .cargo/config.esp32s3.toml run --release --no-default-features --features=esp32s3,defmt --example {{ example }}

std example *args:
    RUST_LOG=info cargo run --no-default-features --features="std,log" --features={{ args }} --example {{ example }}

wasm example *args:
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --no-default-features --features="wasm,web_console" --target wasm32-unknown-unknown -- {{ args }} --example {{ example }}
    wasm-bindgen --target web --out-dir ./examples/web/ ./target/wasm32-unknown-unknown/debug/examples/{{ example }}.wasm --out-name z_example
    basic-http-server ./examples/web
