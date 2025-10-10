clippy:
    cargo clippy
    cargo clippy --features=platform-std
    cargo clippy --features=log,
    cargo clippy --features=defmt
    cargo clippy --features=web_console

    cargo clippy --examples --features=platform-std,
    cargo clippy --examples --features=platform-std,log

wasm-clippy:
    cd platforms/zenoh-nostd-wasm && just clippy

test:
    cargo test codec

bench:
    cargo bench

std example *args:
    RUST_LOG=info cargo run --example {{example}} --features=platform-std,log -- {{args}}

wasm example *args:
    cd platforms/zenoh-nostd-wasm && just wasm {{example}} {{args}}

esp32s3 example *args:
    cd platforms/zenoh-nostd-embassy && just esp32s3 {{example}} {{args}}
