use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use zenoh_sansio::{OpenedTransport, Transport};

use zenoh_proto::{
    exts::QoS,
    fields::{Reliability, WireExpr},
    keyexpr,
    msgs::*,
};
const BATCH_SIZE: usize = u16::MAX as usize;

fn open_listen(stream: &mut std::net::TcpStream) -> OpenedTransport<[u8; BATCH_SIZE]> {
    Transport::new([0u8; BATCH_SIZE])
        .streamed()
        .listen(
            stream,
            |stream, bytes| stream.read_exact(bytes).map(|_| bytes.len()),
            |stream, bytes| stream.write_all(bytes),
        )
        .finish()
        .expect("Error doing handshake")
}

fn open_connect(stream: &mut std::net::TcpStream) -> OpenedTransport<[u8; BATCH_SIZE]> {
    Transport::new([0u8; BATCH_SIZE])
        .streamed()
        .connect(
            stream,
            |stream, bytes| stream.read_exact(bytes).map(|_| bytes.len()),
            |stream, bytes| stream.write_all(bytes),
        )
        .expect("Couldn't send InitSyn")
        .finish()
        .expect("Error doing handshake")
}

fn handle_client(
    mut stream: std::net::TcpStream,
    mut transport: OpenedTransport<[u8; BATCH_SIZE]>,
) {
    let declare = NetworkMessage {
        reliability: Reliability::default(),
        qos: QoS::default(),
        body: NetworkBody::Declare(Declare {
            body: DeclareBody::DeclareSubscriber(DeclareSubscriber {
                id: 0,
                wire_expr: WireExpr::from(keyexpr::from_str_unchecked("test/thr/**")),
            }),
            ..Default::default()
        }),
    };

    transport.tx.encode(core::iter::once(declare));
    let bytes = transport.tx.flush().unwrap();
    stream.write_all(&bytes).unwrap();

    println!("Reading indefinitely from {:?}...", stream.peer_addr());
    let mut rx = [0u8; u16::MAX as usize];
    loop {
        let mut len = [0; 2];
        if stream.read_exact(&mut len).is_err() {
            break;
        }

        let l = u16::from_le_bytes(len) as usize;
        if stream.read_exact(&mut rx[..l]).is_err() {
            break;
        }
    }
}

fn main() {
    match std::env::args().nth(1) {
        None => {
            let listener = TcpListener::bind("127.0.0.1:7447").expect("Could not bind");
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let transport = open_listen(&mut stream);
                        handle_client(stream, transport)
                    }
                    Err(e) => {
                        panic!("Error accepting connection: {}", e);
                    }
                }
            }
        }
        Some(str) => match str.as_str() {
            "--listen" => {
                let listener = TcpListener::bind("127.0.0.1:7447").expect("Could not bind");
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            let transport = open_listen(&mut stream);
                            handle_client(stream, transport)
                        }
                        Err(e) => {
                            panic!("Error accepting connection: {}", e);
                        }
                    }
                }
            }
            "--connect" => {
                let mut stream = TcpStream::connect("127.0.0.1:7447").expect("Couldn't connect");
                let transport = open_connect(&mut stream);
                handle_client(stream, transport)
            }
            _ => {
                panic!("Invalid argument")
            }
        },
    }
}
