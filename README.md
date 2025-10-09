<h1 align="center">zenoh-nostd</h1>
<p align="center"><strong>Zero Network Overhead. No std. No alloc. Pure Rust.</strong></p>
<p align="center">
  <code>bare-metal</code> • <code>no_std</code> • <code>zenoh</code>
</p>

---

## 📦 Overview

> Warning: This project is in early development.

**zenoh-nostd** is a Rust-native, `#![no_std]`, `no alloc` library that implements a **zero-overhead network abstraction** layer for ultra-constrained and bare-metal environments.

> ⚡ Built on the <a href="https://github.com/eclipse-zenoh/zenoh">Zenoh protocol</a>, but stripped to the bone for minimalism and raw performance.

---

## ✨ Features

- **#![no_std]**: No reliance on the standard library.
- **No allocation**: Fully `no alloc`, suitable for `bare-metal` targets.
- **Deterministic**: Zero dynamic memory.
- **Safe Rust first**: Entirely memory-safe.
- **Testable**: Designed for embedded and native testing.

---

## 🚀 Use Cases

| Use Case                    | Suitability ✅  |
|-----------------------------|-----------------|
| IoT microcontrollers        | ✅ Perfect      |
| Space/aero/autonomous       | ✅ Critical safe|
| Linux/MacOS/Windows         | ✅ Ideal        |

---

## 🔧 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zenoh-nostd = { git = "https://github.com/ZettaScaleLabs/zenoh-nostd" }
````

> For embedded systems, make sure your crate is `#![no_std]`:

```rust
#![no_std]
```

---

## 🔌 Integration

### Minimal example

Here is an example of how to send payloads using `zenoh-nostd`:

```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut session = zenoh_nostd::open!(
        PlatformStd: (spawner, PlatformStd {}),
        EndPoint::<32>::from_str(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, from std!";

    let mut tx_zbuf = [0u8; 64];
    session
        .put(tx_zbuf.as_mut_slice(), ke, payload)
        .await
        .unwrap();

    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
}
```

---

## 🔬 MSRV

> 🛠️ **Minimum Supported Rust Version**: (currently) `1.90.0`

---

## ⚠️ Limitations

- Still uses `String<N>` from `heapless` for simplicity at some places. (#10)
- No Serial support yet. (#11)
- No real `Subscriber` struct yet. (#12)
- Limited support for keyexpr subscriptions. (#13)

---

## Building and Testing

This project uses `just` for running tasks. You can check for issues with `just check`. It will `cargo check` the crate and the examples, as well as running the `codec` benchmark.
PRs that result in a slower codec should be rejected.

To test examples, you can use `just platform example args`, available platforms are `std`, `wasm` and `esp32s3` and available examples are `z_put`, `z_sub`.

Additionnally, add the `CONNECT=<endpoint>` environment variable to specify the endpoint to connect to, default is `tcp/127.0.0.1:7447`.
For esp32s3, you also need to specify `WIFI_SSID` and `WIFI_PASSWORD`, also see esp32 instructions on how to install toolchains and targets.

### Concrete examples

- Run a `zenohd` router on `-l tcp/127.0.0.1:7447` and then run:

```bash
# in one terminal
just std z_put
# in another terminal
just std z_sub
```

- Run a `zenohd` router on `-l tcp/127.0.0.1:7447 -l ws/127.0.0.1:7446` and then run:

```bash
# in one terminal
CONNECT=ws/127.0.0.1:7446 just wasm z_put # Make sure to have wasm target installed, wasm-bindgen and basic-http-server
# in another terminal
just std z_sub
```

## 📁 Project Layout

```text
src/
├── keyexpr/       # Lightweight key expression parsing
├── protocol/      # Protocol definition, encoding and decoding
├── platform/      # Platform abstraction layer, with built-in std support
├── logging.rs     # Logging facade
├── result.rs      # Result and Error types
├── zbuf.rs        # Ext traits for bytes buffers
└── lib.rs         # Entry point

platforms/
├── zenoh-embassy  # Embassy devices integration
├── zenoh-wasm32   # Wasm32 platforms integration
```

---

## 📚 Documentation

> 📖 **NOTE**: Docs require `rustdoc` to be run with `--no-default-features`.

Build docs locally:

```bash
cargo doc --no-deps --no-default-features --open
```

---

## 📄 License

Licensed under:

* Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
