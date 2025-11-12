// use std::net::UdpSocket;

fn main() {
    // let (mut session, connect) = zenoh_sansio::open();
    // let mut rx = [0u8; 1024];
    // let mut tx = [0u8; 1024];

    // let udp = UdpSocket::bind("127.0.0.1:0").unwrap();

    // let len = session.write(&mut tx, &[&connect]).unwrap();
    // udp.send_to(&tx[..len], "127.0.0.1:7447").unwrap();

    // while !session.connected() {
    //     let len = udp.recv(&mut rx).unwrap();
    //     let events = session.read(&rx[..len]);

    //     let len = session.write(&mut tx, &[&events]).unwrap();
    //     udp.send_to(&tx[..len], "127.0.0.1:7447").unwrap();
    // }

    // Session is now connected

    // let sub = session.subscribe("demo/example");

    // loop {
    //     let len = udp.recv(&mut rx).unwrap();
    //     let events = session.read(&rx[..len]);

    //     let len = session.write(&mut tx, &[&events, &sub]).unwrap();
    //     udp.send_to(&tx[..len], "127.0.0.1:7447").unwrap();
    // }
}
