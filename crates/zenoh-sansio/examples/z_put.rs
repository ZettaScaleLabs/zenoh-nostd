use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Instant,
};

use zenoh_proto::{Error, ZResult, keyexpr};
use zenoh_sansio::event::Event;

fn entry() -> crate::ZResult<()> {
    env_logger::init();

    zenoh_proto::info!("zenoh-sansio z_put example");

    // Create the RX and TX buffers. Here it's on the stack for simplicity, but
    // you may want to allocate them on the heap for larger sizes.
    // Also create a TCP link.
    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];
    let mut tcp = TcpStream::connect("127.0.0.1:7447").map_err(|_| Error::CouldNotConnect)?;

    // To be system agnostic, the way to measure time is left to the user, the session
    // only requires a `Duration` representing the elapsed time since the session creation.
    //
    // Every call to `update` must be fed with the current elapsed time.
    let start = Instant::now();
    // Create a new Zenoh session, it also returns a "connect" event,
    // which represents the initial connection request to be sent to the other
    // peer (InitSyn message).
    let (mut session, connect) = zenoh_sansio::open();
    // Dispatch all events (currently only the "connect" event) to the TCP link.
    // It encodes all events into the `tx` buffer and uses the provided closure to send
    // the bytes.
    //
    // This function may seem useless, but it could handle fragmentation, optimizations of one-to-many
    // messages, etc. As for now, we only have one-to-one messages without fragmentation.
    session.dispatch(&mut tx, [connect].into_iter(), |bytes| {
        tcp_write(&mut tcp, bytes)
    })?;

    // Handshake loop: keep calling `update` until the session is connected.
    //
    // The `update` function takes the received bytes and the elapsed time since
    // the session creation. It also takes a Buffer to store the potential reponse events: here it uses
    // a fixed-size array of empty events, but it could be any data structure implementing
    // `AsEventAccumulator`, such as `alloc::vec::Vec`.
    //
    // `session.update` returns a `ZResult` containing either an error, indicating that the session failed to
    // compute the bytes, or a `ZResult` representing either:
    // - Ok(Iterator) -> the buffer was large enough to store all response events,
    // - Err(Iterator) -> the buffer was not large enough, and only a part of the response events are returned.
    // In this case, the user should provide a larger buffer.
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

    // Main loop: send a PUT request every second. Set the TCP stream to non-blocking.
    tcp.set_nonblocking(true)
        .map_err(|_| Error::ConfigNotSupported)?;
    while !session.disconnected() {
        // This creates a PUT event.
        let put = session.put(ke, payload);

        // Again, read over the TCP link and feed the received bytes to the session.
        let n = tcp_read(&mut tcp, &mut rx).unwrap_or_default();
        if let Ok(response) = session.update(&rx[..n], start.elapsed(), [Event::EMPTY; 16])? {
            // Dispatch any response events along with the PUT event.
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

fn tcp_write(tcp: &mut TcpStream, buffer: &[u8]) -> crate::ZResult<()> {
    tcp.write_all(&(buffer.len() as u16).to_le_bytes())
        .map_err(|_| Error::TxError)
        .map(|_| ())?;

    tcp.write_all(buffer)
        .map_err(|_| Error::TxError)
        .map(|_| ())
}

fn tcp_read(tcp: &mut TcpStream, buffer: &mut [u8]) -> crate::ZResult<usize> {
    let mut len = [0u8; 2];
    tcp.read_exact(&mut len).map_err(|_| Error::InvalidRx)?;
    let n = u16::from_le_bytes(len) as usize;
    tcp.read_exact(&mut buffer[..n])
        .map_err(|_| Error::InvalidRx)
        .map(|_| n)
}
