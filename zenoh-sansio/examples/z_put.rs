use std::{
    net::UdpSocket,
    time::{Duration, Instant},
};

use zenoh_sansio::{ZResult, ke::keyexpr};

fn main() -> ZResult<()> {
    let start = Instant::now();
    let (mut session, connect) = zenoh_sansio::open();
    let mut rx = [0u8; 1024];
    let mut tx = [0u8; 1024];

    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();

    session
        .dispatch(&mut tx, [connect], |bytes: &[u8]| {
            udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
        })
        .unwrap();

    while !session.connected() {
        let len = udp.recv(&mut rx).unwrap();
        let response = session.read(&rx[..len], start.elapsed())?;

        session.dispatch(&mut tx, [response], |bytes: &[u8]| {
            udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
        })?;
    }

    // Session is now connected
    let ke = keyexpr::new("demo/example")?;

    udp.set_nonblocking(true).unwrap();
    while session.connected() {
        let len = udp.recv(&mut rx).unwrap_or_default();
        let response = session.read(&rx[..len], start.elapsed())?;

        let put = session.put(ke, b"Hello, Zenoh!")?;

        session
            .dispatch(&mut tx, [response, put], |bytes: &[u8]| {
                udp.send_to(&bytes, "127.0.0.1:7447").unwrap();
                println!("Sent {} bytes", bytes.len());
            })
            .unwrap();

        std::thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

/*
Good:
25 f7 c2 db 71 7d 00 0c 64 65 6d 6f 2f
65 78 61 6d 70 6c 65 01 10 48 65 6c 6c
6f 2c 20 66 72 6f 6d 20 73 74 64 21

not Good:
05 00 a9 c7 86 7b 05 3d 00 0c 64 65 6d
6f 2f 65 78 61 6d 70 6c 65 01 0d 48 65
6c 6c 6f 2c 20 5a 65 6e 6f 68 21

*/
