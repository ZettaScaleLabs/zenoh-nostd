use crate::{Transport, transport::establishment::State};
use core::{cell::RefCell, time::Duration};
use zenoh_proto::{fields::*, msgs::*};

#[test]
fn transport_state_handshake() {
    let a_zid = ZenohIdProto::default();
    let mut a = State::WaitingInitSyn {
        mine_zid: a_zid,
        mine_batch_size: 512,
        mine_resolution: Resolution::default(),
        mine_lease: Duration::from_secs(30),
    };

    let b_zid = ZenohIdProto::default();
    let mut b = State::WaitingInitAck {
        mine_zid: b_zid,
        mine_batch_size: 1025,
        mine_resolution: Resolution::default(),
        mine_lease: Duration::from_secs(37),
    };

    let init = TransportMessage::InitSyn(InitSyn {
        identifier: InitIdentifier {
            zid: b_zid,
            ..Default::default()
        },
        resolution: InitResolution {
            resolution: Resolution::default(),
            batch_size: BatchSize(1025),
        },
        ..Default::default()
    });

    let mut buff = [0u8; 128];

    macro_rules! buff {
        ($msg:expr) => {{
            let len: usize =
                zenoh_proto::transport_encoder_ref(&mut buff, core::iter::once($msg)).sum();

            &buff[..len]
        }};
    }

    let mut buff = buff!(&init);
    let mut next = Some(init);
    let mut desc = None;
    let mut current = &mut a;
    let mut other = &mut b;

    for _ in 0..4 {
        if let Some(response) = next {
            (next, desc) = current.poll((response, buff));
            core::mem::swap(&mut current, &mut other);

            buff = &[];
        }
    }

    assert!(desc.is_some());
    assert!(a.description().is_some() && b.description().is_some());
    assert_eq!(desc.unwrap().batch_size, 512);
    assert_eq!(desc.unwrap().resolution, Resolution::default());
}

#[test]
fn transport_handshake() {
    let socket = ([0u8; 512], 0usize);
    let socket_ref = RefCell::new(socket);

    let a = Transport::new([0u8; 512]);
    let b = Transport::new([0u8; 512]);

    extern crate std;

    let read = |socket: &mut &RefCell<([u8; 512], usize)>,
                bytes: &mut [u8]|
     -> core::result::Result<usize, i32> {
        let borrow = socket.borrow();
        let slice = &borrow.0[..borrow.1];

        bytes[..slice.len()].copy_from_slice(slice);
        Ok(slice.len())
    };

    let write = |socket: &mut &RefCell<([u8; 512], usize)>,
                 bytes: &[u8]|
     -> core::result::Result<(), i32> {
        std::println!("Writing message of size {}", bytes.len());

        let mut borrow_mut = socket.borrow_mut();
        borrow_mut.0[..bytes.len()].copy_from_slice(bytes);
        borrow_mut.1 = bytes.len();
        Ok(())
    };

    let mut ha = a.listen(&socket_ref, &read, &write);
    let mut hb = b
        .connect(&socket_ref, &read, &write)
        .expect("Couldn't write InitSyn");

    for _ in 0..2 {
        ha.poll().unwrap();
        hb.poll().unwrap();
    }

    ha.poll()
        .expect("Unexpected Error")
        .expect("Transport A is not opened yet")
        .open();

    hb.poll()
        .expect("Unexpected Error")
        .expect("Transport B is not opened yet")
        .open();
}

#[test]
fn transport_handshake_streamed() {
    let socket = ([0u8; 512], 0usize);
    let socket_ref = RefCell::new(socket);

    let a = Transport::new([0u8; 512]).streamed();
    let b = Transport::new([0u8; 512]).streamed();

    extern crate std;

    let read = |socket: &mut &RefCell<([u8; 512], usize)>,
                bytes: &mut [u8]|
     -> core::result::Result<usize, i32> {
        let borrow = socket.borrow();

        std::println!("Reading message of size {}/{}", borrow.1, bytes.len());
        if bytes.len() == 2 {
            bytes.copy_from_slice(&borrow.0[..2]);
            return Ok(2);
        }

        let slice = &borrow.0[2..borrow.1];

        bytes[..slice.len()].copy_from_slice(slice);
        Ok(slice.len())
    };

    let write = |socket: &mut &RefCell<([u8; 512], usize)>,
                 bytes: &[u8]|
     -> core::result::Result<(), i32> {
        std::println!("Writing message of size {}", bytes.len());

        let mut borrow_mut = socket.borrow_mut();
        borrow_mut.0[..bytes.len()].copy_from_slice(bytes);
        borrow_mut.1 = bytes.len();
        Ok(())
    };

    let mut ha = a.listen(&socket_ref, &read, &write);
    let mut hb = b
        .connect(&socket_ref, &read, &write)
        .expect("Couldn't write InitSyn");

    for _ in 0..2 {
        ha.poll().unwrap();
        hb.poll().unwrap();
    }

    ha.poll()
        .expect("Unexpected Error")
        .expect("Transport A is not opened yet")
        .open();

    hb.poll()
        .expect("Unexpected Error")
        .expect("Transport B is not opened yet")
        .open();
}
