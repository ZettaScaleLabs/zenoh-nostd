#![no_std]

pub mod tcp;

pub trait Platform {
    type PlatformTcpStream: tcp::PlatformTcpStream;
}
