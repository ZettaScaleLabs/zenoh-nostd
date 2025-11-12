clippy:
    cargo clippy --fix --lib --allow-dirty --allow-staged --tests -p zenoh-sansio
    cargo clippy --fix --lib --allow-dirty --allow-staged --tests -p zenoh-sansio-codec

test:
    cargo test --all

bench:
    cargo test bench --profile=release -- --nocapture --ignored

tokei:
    tokei --files --type Rust \
        zenoh-sansio-codec/src \
        zenoh-sansio/src/codec.rs \
        zenoh-sansio/src/ext.rs \
        zenoh-sansio/src/struct* \
        zenoh-sansio/src/protocol*
