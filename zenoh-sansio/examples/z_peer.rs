use std::{net::UdpSocket, time::Instant};

use zenoh_sansio::ZResult;

// This example doesn't work for now. It's only here to illustrate how to use the
// sans-io session in a multiple peer scenario.
//
// This example uses a `PeerTable` to manage multiple peers connecting. It just acts
// like a switch, internally the session will route the messages to the correct peer.
fn main_switch() -> ZResult<()> {
    let start = Instant::now();

    let mut session = zenoh_sansio::open();

    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];

    // Peer table to manage multiple peers, with a maximum of 16 peers in this example.
    let mut table = PeerTable::new::<16>();

    // In this scenario we assume multiple peers may connect to us through the same UDP socket for simplicity.
    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();

    loop {
        // Receive data from any peer. When updating the session, we also pass the peer information from
        // the peer table. When new peers connect, new entries will be created in the table and the session
        // will adapt.
        let (len, peer) = udp.recv_from(&mut rx).unwrap_or_default();
        let response = session.read(start.elapsed(), &rx[..len], table.entry(&peer))?;

        // Dispatch the response back to the correct peer. Here we use the same socket, but it's
        // possible to use different sockets (even streams or anything), by matching the peer information.
        //
        // Warning: in this context, an event generated from the session has more information than just the event
        // itself, it also has the peer information to know where to send it. (not yet implemented)
        session.dispatch(&mut tx, [response], |bytes: &[u8], peer| {
            udp.send_to(&bytes, table.addr(peer)).unwrap();
        })?;

        if session.connected() {
            udp.set_read_timeout(Some(session.lease())).unwrap();
        }

        if session.disconnected() {
            break;
        }
    }

    Ok(())
}

// Same as above.
//
// This example also publishes data to all connected peers.
fn main_put() -> ZResult<()> {
    let start = Instant::now();

    let mut session = zenoh_sansio::open();

    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];

    // Peer table to manage multiple peers, with a maximum of 16 peers in this example.
    let mut table = PeerTable::new::<16>();

    // In this scenario we assume multiple peers may connect to us through the same UDP socket for simplicity.
    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();

    let ke = keyexpr::new("demo/example")?;
    loop {
        // Receive data from any peer. When updating the session, we also pass the peer information from
        // the peer table. When new peers connect, new entries will be created in the table and the session
        // will adapt.
        let (len, peer) = udp.recv_from(&mut rx).unwrap_or_default();
        let response = session.read(start.elapsed(), &rx[..len], table.entry(&peer))?;

        let put = session.put(ke, b"Hello, from Rust!");
        session.dispatch(&mut tx, [response, put], |bytes: &[u8], peer| {
            udp.send_to(&bytes, table.addr(peer)).unwrap();
        })?;

        if session.connected() {
            udp.set_nonblocking(true).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        if session.disconnected() {
            break;
        }
    }

    Ok(())
}
