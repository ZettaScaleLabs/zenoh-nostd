<h1 align="center">zenoh-nostd</h1>
<p align="center"><strong>Zero Network Overhead. No std. No alloc. Pure Rust.</strong></p>
<p align="center">
  <code>bare-metal</code> • <code>no_std</code> • <code>zenoh</code>
</p>

---

## 📦 Overview

**zenoh-nostd** is a Rust-native, `#![no_std]`, `heapless` library that implements a **zero-overhead network abstraction** layer for ultra-constrained and bare-metal environments.

> ⚡ Built on the <a href="https://github.com/eclipse-zenoh/zenoh">Zenoh protocol</a>, but stripped to the bone for minimalism and raw performance.

---

## ✨ Features

- **No_std**: No reliance on the standard library.
- **No allocation**: Fully `heapless`, suitable for `bare-metal` targets.
- **Deterministic**: Zero dynamic memory.
- **Safe Rust first**: Entirely memory-safe.
- **Testable**: Designed for embedded and native testing.

---

## 🚀 Use Cases

| Use Case                     | Suitability ✅ |
|-----------------------------|----------------|
| IoT microcontrollers        | ✅ Perfect      |
| Space/aero/autonomous       | ✅ Critical safe|
| Linux/server environments   | ✅ Ideal |

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

Coming soon!

---

## 🔬 MSRV

> 🛠️ **Minimum Supported Rust Version**: (currently) `1.90.0`

---

## ⚠️ Limitations

Coming soon!

---

## Building and Testing

Coming soon!

## 📁 Project Layout

```text
src/
├── keyexpr/       # Lightweight key expression parsing
├── protocol/      # Protocol definition, encoding and decoding
├── platform/      # Platform abstraction layer
├── logging.rs     # Logging facade
├── result.rs      # Result and Error types
├── zbuf.rs        # Ext traits for bytes buffers
└── lib.rs         # Entry point

platforms/
├── zenoh-embassy  # Embassy devices integration
├── zenoh-wasm32   # Wasm32 platforms integration
└── zenoh-std      # Std devices integration
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

---

> Zenoh-nostd is maintained with ❤️ by [ZettaScaleLabs].
