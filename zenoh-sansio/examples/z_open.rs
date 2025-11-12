use std::{net::UdpSocket, time::Instant};

use zenoh_sansio::ZResult;

fn main() -> ZResult<()> {
    let start = Instant::now();
    let (mut session, connect) = zenoh_sansio::open();
    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];

    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
    session.dispatch(&mut tx, [connect], |bytes: &[u8]| {
        udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
    })?;

    while !session.connected() {
        let len = udp.recv(&mut rx).unwrap();
        let response = session.read(&rx[..len], start.elapsed())?;

        session.dispatch(&mut tx, [response], |bytes: &[u8]| {
            udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
        })?;
    }

    // Session is now connected
    udp.set_read_timeout(Some(session.lease())).unwrap();
    while session.connected() {
        let len = udp.recv(&mut rx).unwrap_or_default();
        let response = session.read(&rx[..len], start.elapsed())?;

        session.dispatch(&mut tx, [response], |bytes: &[u8]| {
            udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
        })?;
    }

    Ok(())
}
