test:
    cargo check
    cargo check --features=platform-std
    cargo check --features=log,
    cargo check --features=defmt
    cargo check --features=web_console

    cargo check --examples --features=platform-std,
    cargo check --examples --features=platform-std,log

    cargo test codec

bench:
    cargo bench

std example *args:
    RUST_LOG=info cargo run --example {{example}} --features=platform-std,log -- {{args}}

wasm example *args:
    cd platforms/zenoh-nostd-wasm && just wasm {{example}} {{args}}

esp32s3 example *args:
    cd platforms/zenoh-nostd-embassy && just esp32s3 {{example}} {{args}}
