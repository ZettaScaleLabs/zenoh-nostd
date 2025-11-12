use std::{net::UdpSocket, time::Instant};

use zenoh_sansio::ZResult;

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

    // Event loop: we receive data from the UDP socket and then we pass the received data to the session.
    // The `read` method may generate a response event, we will need to dispatch it back to the peer.
    //
    // This loop handles the end of the connection establishment as well as the keepalive messages.
    loop {
        let len = udp.recv(&mut rx).unwrap_or_default();
        let response = session.read(start.elapsed(), &rx[..len])?;

        session.dispatch(&mut tx, [response], |bytes: &[u8]| {
            udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
        })?;

        // Once the session is connected we can set the read timeout so that we don't block forever on recv.
        // This is useful in this example because the peer expects keepalive messages to be sent periodically.
        // The keepalive message will be sent automatically on a call to `dispatch` if needed.
        if session.connected() {
            udp.set_read_timeout(Some(session.lease())).unwrap();
        }

        if session.disconnected() {
            break;
        }
    }

    Ok(())
}
