use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};

use zenoh_proto::{
    BatchReader, BatchWriter, Message,
    msgs::{InitAck, OpenAck},
};

fn handle_client(mut stream: std::net::TcpStream) {
    let mut rx = [0; u16::MAX as usize];
    let mut tx = [0; u16::MAX as usize];

    let mut len = [0; 2];
    stream.read_exact(&mut len).expect("Could not read length");
    let l = u16::from_le_bytes(len) as usize;
    stream
        .read_exact(&mut rx[..l])
        .expect("Could not read InitSyn");
    let mut batch = BatchReader::new(&rx[..l]);
    let _ = loop {
        match batch.next() {
            Some(Message::InitSyn(i)) => break i,
            Some(_) => continue,
            None => panic!("Did not receive InitSyn"),
        }
    };

    let init_ack = InitAck::default();
    let mut batch = BatchWriter::new(&mut tx[2..], 0);
    batch.unframed(&init_ack).expect("Could not encode InitAck");
    let (_, payload_len) = batch.finalize();
    let len_bytes = (payload_len as u16).to_le_bytes();
    tx[..2].copy_from_slice(&len_bytes);
    stream
        .write_all(&tx[..payload_len + 2])
        .expect("Could not send InitAck");

    let mut len = [0; 2];
    stream.read_exact(&mut len).expect("Could not read length");
    let l = u16::from_le_bytes(len) as usize;
    stream
        .read_exact(&mut rx[..l])
        .expect("Could not read OpenSyn");
    let mut batch = BatchReader::new(&rx[..l]);
    let _ = loop {
        match batch.next() {
            Some(Message::OpenSyn(o)) => break o,
            Some(_) => continue,
            None => panic!("Did not receive OpenSyn"),
        }
    };

    let open_ack = OpenAck {
        lease: Duration::from_secs(60),
        ..Default::default()
    };
    let mut batch = BatchWriter::new(&mut tx[2..], 0);
    batch.unframed(&open_ack).expect("Could not encode OpenAck");
    let (_, payload_len) = batch.finalize();
    let len_bytes = (payload_len as u16).to_le_bytes();
    tx[..2].copy_from_slice(&len_bytes);
    stream
        .write_all(&tx[..payload_len + 2])
        .expect("Could not send OpenAck");

    // Just read messages indefinitely
    loop {
        let mut len = [0; 2];
        stream.read_exact(&mut len).expect("Could not read length");
        let l = u16::from_le_bytes(len) as usize;
        stream
            .read_exact(&mut rx[..l])
            .expect("Could not read message");
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7447").expect("Could not bind");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => {
                panic!("Error accepting connection: {}", e);
            }
        }
    }
}
