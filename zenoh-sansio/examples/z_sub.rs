use std::{net::UdpSocket, time::Instant};

use zenoh_sansio::{ZResult, ke::keyexpr};

fn callback(sample: &[u8]) {
    let _ = sample;
}

fn main() -> ZResult<()> {
    // To be system agnostic, the way to measure time is left to the user, the session
    // only requires a `Duration` representing the elapsed time since the session creation.
    //
    // Every call to `read` must be fed with the current elapsed time.
    let start = Instant::now();

    // Create a new Zenoh session, it also returns a "connect" event,
    // which represents the initial connection request to be sent to the other
    // peer.
    let (mut session, connect) = zenoh_sansio::open();

    // Create the RX and TX buffers. Here it's on the stack for simplicity, but
    // you may want to allocate them on the heap for larger sizes.
    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];

    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();

    // Dispatch all the events into by writing them into the `tx` buffer and then
    // sending them through the UDP socket.
    //
    // Design: This design has been thought for multiple connections, the idea is that
    // the callback passed to `dispatch` have the `bytes` as well as the destination (not yet).
    // In fact this design allows optimizations in case you need to send the same data to multiple peers,
    // you only need to serialize once the data and then send the slices to the different destinations.
    // However for now we only have one peer, so we ignore this aspect.
    session.dispatch(&mut tx, [connect], |bytes: &[u8]| {
        udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
    })?;

    while !session.connected() {
        let len = udp.recv(&mut rx).unwrap();
        let response = session.read(start.elapsed(), &rx[..len])?;

        session.dispatch(&mut tx, [response], |bytes: &[u8]| {
            udp.send_to(bytes, "127.0.0.1:7447").unwrap();
        })?;
    }

    // Session is now connected
    udp.set_read_timeout(Some(session.lease())).unwrap();

    let sub = session.declare_subscriber(keyexpr::new("demo/example")?, callback);
    session.dispatch(&mut tx, [sub], |bytes: &[u8]| {
        udp.send_to(bytes, "127.0.0.1:7447").unwrap();
    })?;

    loop {
        let len = udp.recv(&mut rx).unwrap_or_default();
        let response = session.read(start.elapsed(), &rx[..len])?;

        session.dispatch(&mut tx, [response], |bytes: &[u8]| {
            udp.send_to(bytes, "127.0.0.1:7447").unwrap();
        })?;

        if session.disconnected() {
            break;
        }
    }

    Ok(())
}
