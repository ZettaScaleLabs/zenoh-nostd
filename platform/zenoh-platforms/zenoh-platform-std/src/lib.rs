use zenoh_platform::Platform;

pub mod tcp;

pub struct PlatformStd;

impl Platform for PlatformStd {
    type PlatformTcpStream = tcp::PlatformStdTcpStream;
}
