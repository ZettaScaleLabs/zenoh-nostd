check:
    cargo clippy -p zenoh-nostd --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=log --fix --lib --allow-dirty --allow-staged
    cargo clippy -p zenoh-nostd --features=defmt --fix --lib --allow-dirty --allow-staged

    cd platforms/zenoh-std && just check
    cd platforms/zenoh-embassy && just check

    cargo clippy --examples --features=std,log --fix --lib --allow-dirty --allow-staged

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
