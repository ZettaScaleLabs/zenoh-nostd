check:
    cargo clippy --fix --lib --allow-dirty --allow-staged
    cargo clippy --features=zenoh-proto/log --fix --lib --allow-dirty --allow-staged
    cargo clippy --features=zenoh-proto/defmt --fix --lib --allow-dirty --allow-staged

    cargo clippy --examples --fix --lib --allow-dirty --allow-staged
    cargo clippy --examples --features=zenoh-proto/log --fix --lib --allow-dirty --allow-staged
    cargo clippy --examples --features=zenoh-proto/defmt --fix --lib --allow-dirty --allow-staged

    cd platforms/zenoh-std && just check

test:
    cargo test --all
