# Project

clippy:
    cargo clippy --fix --lib --allow-dirty --allow-staged
    cargo clippy --features=platform-std --fix --lib --allow-dirty --allow-staged
    cargo clippy --features=log --fix --lib --allow-dirty --allow-staged
    cargo clippy --features=defmt --fix --lib --allow-dirty --allow-staged
    cargo clippy --features=web_console --fix --lib --allow-dirty --allow-staged

    cargo clippy --examples --features=platform-std, --fix --lib --allow-dirty --allow-staged
    cargo clippy --examples --features=platform-std,log --fix --lib --allow-dirty --allow-staged

test:
    cargo test codec

bench:
    cargo test --profile=release -- --nocapture bench --

# Ping

ping:
    RUST_LOG=info cargo run --example z_ping --features=platform-std,log --release

pong:
    RUST_LOG=info cargo run --example z_pong --features=platform-std,log --release

# Examples

std example *args:
    RUST_LOG=trace cargo run --example {{example}} --features=platform-std,log -- {{args}}

esp32s3 example *args:
    cd platforms/zenoh-nostd-embassy && just esp32s3 {{example}} {{args}}
