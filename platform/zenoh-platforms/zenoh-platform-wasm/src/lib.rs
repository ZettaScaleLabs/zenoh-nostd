use zenoh_platform::Platform;

pub mod ws;

pub struct PlatformWasm;

impl Platform for PlatformWasm {
    type PlatformTcpStream = zenoh_platform::tcp::DummyPlatformTcpStream;
    type PlatformWSStream = ws::PlatformWasmWSStream;
}
