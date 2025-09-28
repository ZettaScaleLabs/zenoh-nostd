version:
    @echo "zenoh-nostd version 1.5.1"

check-std:
    cargo check -p zenoh

check-wasm:
    cargo check -p zenoh --target wasm32-unknown-unknown

check-esp:
    cargo +esp check -Zbuild-std=core,alloc -p zenoh --target xtensa-esp32-none-elf

check: check-std check-wasm check-esp

build-std:
    cargo build -p zenoh

build-wasm:
    cargo build -p zenoh --target wasm32-unknown-unknown

build-esp:
    cargo +esp build -Zbuild-std=core,alloc -p zenoh --target xtensa-esp32-none-elf

build: build-std build-wasm build-esp

release-std:
    cargo build -p zenoh --release

release-wasm:
    cargo build -p zenoh --target wasm32-unknown-unknown --release

release-esp:
    cargo +esp build -Zbuild-std=core,alloc -p zenoh --target xtensa-esp32-none-elf --release

release: release-std release-wasm release-esp

z_put-std:
    cd platform/zenoh-platforms/zenoh-platform-std && cargo run --example z_put

z_put-wasm:
    cd platform/zenoh-platforms/zenoh-platform-wasm && just run_z_put

z_put-esp:
    cd platform/zenoh-platforms/zenoh-platform-embassy/example-esp32s3 && cargo run --example z_put
