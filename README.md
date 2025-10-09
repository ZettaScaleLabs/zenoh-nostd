<h1 align="center">zenoh-nostd</h1>
<p align="center"><strong>Zero Network Overhead. No std. No alloc. Pure Rust.</strong></p>
<p align="center">
  <code>bare-metal</code> • <code>no_std</code> • <code>zenoh</code>
</p>

---

## 📦 Overview

⚠️ This project is in early development.

**zenoh-nostd** is a Rust-native, `#![no_std]`, `no_alloc` library that provides a **zero-overhead network abstraction layer** for ultra-constrained and bare-metal environments.

> ⚡ Built on the <a href="https://github.com/eclipse-zenoh/zenoh">Zenoh protocol</a>, but stripped to the bone for minimalism and raw performance.

---

## ✨ Features

- **`#![no_std]`**: No reliance on the Rust standard library.
- **No dynamic allocation**: Fully `no_alloc`, ideal for bare-metal targets.
- **Deterministic**: No heap, no surprises.
- **Safe Rust**: Entirely memory-safe.
- **Testable**: Supports both embedded and native testing environments.

---

## 🚀 Use Cases

| Use Case               | Suitability     |
|------------------------|-----------------|
| IoT microcontrollers   | ✅ Perfect       |
| Space/Aerospace/Robotics | ✅ Safety-critical |
| Linux/macOS/Windows    | ✅ Ideal         |

---

## 🔧 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zenoh-nostd = { git = "https://github.com/ZettaScaleLabs/zenoh-nostd" }
```

> For embedded systems, ensure your crate uses `#![no_std]`:

```rust
#![no_std]
```

---

## 🔌 Integration

### Minimal Example

Here’s a simple example of sending a payload with `zenoh-nostd`:

```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut session = zenoh_nostd::open!(
        PlatformStd: (spawner, PlatformStd {}),
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
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

> 🛠️ **Minimum Supported Rust Version**: Currently `1.89.0`

---

## ⚠️ Limitations

* Uses `String<N>` from `heapless` for simplicity in some places. ([#10](https://github.com/ZettaScaleLabs/zenoh-nostd/issues/10))
* No serial support yet. ([#11](https://github.com/ZettaScaleLabs/zenoh-nostd/issues/11))
* No dedicated `Subscriber` struct yet. ([#12](https://github.com/ZettaScaleLabs/zenoh-nostd/issues/12))
* Limited support for key expression subscriptions. ([#13](https://github.com/ZettaScaleLabs/zenoh-nostd/issues/13))

---

## 🧪 Building and Testing

This project uses [`just`](https://github.com/casey/just) for task management. Use `just check` to verify the crate and examples and run the `codec` benchmark.

> 🔍 Pull requests that slow down the codec should be rejected.

### Testing Examples

Use the following command structure:

```bash
just <platform> <example> [args]
```

* **Platforms**: `std`, `wasm`, `esp32s3`
* **Examples**: `z_put`, `z_sub`

Set the `CONNECT=<endpoint>` environment variable to specify the endpoint (default is `tcp/127.0.0.1:7447`).

For `esp32s3`, you must also provide:

* `WIFI_SSID`
* `WIFI_PASSWORD`

See the ESP32 setup documentation for toolchain and target installation.

### Example: Local TCP

Run a Zenoh router with:

```bash
zenohd -l tcp/127.0.0.1:7447
```

In two terminals:

```bash
# Terminal 1
just std z_put

# Terminal 2
just std z_sub
```

### Example: WebSocket + WASM

Run a Zenoh router with:

```bash
zenohd -l tcp/127.0.0.1:7447 -l ws/127.0.0.1:7446
```

Then:

```bash
# Terminal 1 (WASM)
CONNECT=ws/127.0.0.1:7446 just wasm z_put

# Terminal 2 (STD)
just std z_sub
```

> 📦 Note: For WASM, ensure you have:
>
> * `wasm32-unknown-unknown` target
> * `wasm-bindgen-cli`
> * `basic-http-server` (or similar)

---

## 📁 Project Layout

```text
src/
├── keyexpr/       # Lightweight key expression parsing
├── protocol/      # Protocol definitions, encoding/decoding
├── platform/      # Platform abstraction (e.g., std support)
├── logging.rs     # Logging facade
├── result.rs      # Result and error types
├── zbuf.rs        # Byte buffer extension traits
└── lib.rs         # Library entry point

platforms/
├── zenoh-embassy  # Integration with Embassy-based devices
├── zenoh-wasm32   # WASM32 platform integration
```

---

## 📚 Documentation

The base project has been implemented in ([#6](https://github.com/ZettaScaleLabs/zenoh-nostd/pull/6))

> 📖 **Note**: Docs require `rustdoc` to be run with `--no-default-features`.

Build locally with:

```bash
cargo doc --no-deps --no-default-features --open
```

---

## 📄 License

Licensed under:

* Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
