use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Instant,
};

use zenoh_proto::{ZError, ZResult, keyexpr};
use zenoh_sansio::event::Event;

fn entry() -> ZResult<()> {
    env_logger::init();
    zenoh_proto::info!("zenoh-sansio z_put example");

    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];
    let mut tcp = TcpStream::connect("127.0.0.1:7447").map_err(|_| ZError::CouldNotConnect)?;

    let start = Instant::now();
    let (mut session, connect) = zenoh_sansio::open();
    session.dispatch(&mut tx, [connect].into_iter(), |bytes| {
        tcp_write(&mut tcp, bytes)
    })?;

    while !session.connected() {
        let n = tcp_read(&mut tcp, &mut rx).unwrap_or_default();
        if let Ok(response) = session.update(&rx[..n], start.elapsed(), [Event::EMPTY; 16])? {
            session.dispatch(&mut tx, response.into_iter(), |bytes| {
                tcp_write(&mut tcp, bytes)
            })?;
        }
    }

    let ke = keyexpr::new("demo/example")?;
    let payload = b"Hello, from sansio!";

    tcp.set_nonblocking(true)
        .map_err(|_| ZError::ConfigNotSupported)?;
    while !session.disconnected() {
        let put = session.put(ke, b"Hello, from sansio!");

        let n = tcp_read(&mut tcp, &mut rx).unwrap_or_default();
        if let Ok(response) = session.update(&rx[..n], start.elapsed(), [Event::EMPTY; 16])? {
            session.dispatch(&mut tx, response.chain([put]), |bytes| {
                tcp_write(&mut tcp, bytes)
            })?;
        }

        zenoh_proto::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

fn main() {
    match entry() {
        Ok(_) => {}
        Err(e) => {
            zenoh_proto::error!("Error: {:?}", e);
        }
    }
}

fn tcp_write(tcp: &mut TcpStream, buffer: &[u8]) -> ZResult<()> {
    tcp.write_all(&(buffer.len() as u16).to_le_bytes())
        .map_err(|_| ZError::TxError)
        .map(|_| ())?;

    tcp.write_all(buffer)
        .map_err(|_| ZError::TxError)
        .map(|_| ())
}

fn tcp_read(tcp: &mut TcpStream, buffer: &mut [u8]) -> ZResult<usize> {
    let mut len = [0u8; 2];
    tcp.read_exact(&mut len).map_err(|_| ZError::InvalidRx)?;
    let n = u16::from_le_bytes(len) as usize;
    tcp.read_exact(&mut buffer[..n])
        .map_err(|_| ZError::InvalidRx)
        .map(|_| n)
}
