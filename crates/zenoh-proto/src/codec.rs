pub mod ext;
pub mod r#struct;

pub use ext::*;
pub use r#struct::*;

use crate::{ZReadable, exts::*, fields::*, msgs::*};

fn decode<'a>(
    reader: &mut &'a [u8],
    reliability: &mut Option<Reliability>,
    qos: &mut Option<QoS>,
    sn: &mut u32,
    resolution: Resolution,
) -> Option<Message<'a>> {
    if !reader.can_read() {
        return None;
    }

    let header = reader
        .read_u8()
        .expect("reader should not be empty at this stage");

    macro_rules! decode {
        ($ty:ty) => {
            match <$ty as $crate::ZBodyDecode>::z_body_decode(reader, header) {
                Ok(msg) => msg,
                Err(e) => {
                    crate::error!(
                        "Failed to decode message of type {}: {}. Skipping the rest of the message - {}",
                        core::any::type_name::<$ty>(),
                        e,
                        crate::zctx!()
                    );

                    return None;
                }
            }
        };
    }

    let ack = header & 0b0010_0000 != 0;
    let net = reliability.is_some() && qos.is_some();
    let ifinal = header & 0b0110_0000 == 0;
    let id = header & 0b0001_1111;

    let body = match id {
        FrameHeader::ID => {
            let header = decode!(FrameHeader);

            // Check for missed messages regarding resolution
            let _ = resolution;
            if header.sn <= *sn && *sn != 0 {
                crate::error!(
                    "Inconsistent `SN` value {}, expected higher than {}",
                    header.sn,
                    sn
                );
                return None;
            } else if header.sn != *sn + 1 && *sn != 0 {
                crate::debug!("Transport missed {} messages", header.sn - *sn - 1);
            }

            reliability.replace(header.reliability);
            qos.replace(header.qos);
            *sn = header.sn;

            return decode(reader, reliability, qos, sn, resolution);
        }
        InitAck::ID if ack => Message::Transport(TransportMessage::InitAck(decode!(InitAck))),
        InitSyn::ID => Message::Transport(TransportMessage::InitSyn(decode!(InitSyn))),
        OpenAck::ID if ack => Message::Transport(TransportMessage::OpenAck(decode!(OpenAck))),
        OpenSyn::ID => Message::Transport(TransportMessage::OpenSyn(decode!(OpenSyn))),
        Close::ID => Message::Transport(TransportMessage::Close(decode!(Close))),
        KeepAlive::ID => Message::Transport(TransportMessage::KeepAlive(decode!(KeepAlive))),
        Push::ID if net => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::Push(decode!(Push)),
        }),
        Request::ID if net => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::Request(decode!(Request)),
        }),
        Response::ID if net => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::Response(decode!(Response)),
        }),
        ResponseFinal::ID if net => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::ResponseFinal(decode!(ResponseFinal)),
        }),
        InterestFinal::ID if net && ifinal => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::InterestFinal(decode!(InterestFinal)),
        }),
        Interest::ID if net => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::Interest(decode!(Interest)),
        }),
        Declare::ID if net => Message::Network(NetworkMessage {
            reliability: reliability.expect("Should be a frame. Something went wrong."),
            qos: qos.expect("Should be a frame. Something went wrong."),
            body: NetworkBody::Declare(decode!(Declare)),
        }),
        _ => {
            crate::error!(
                "Unrecognized message header: {:08b}. Skipping the rest of the message - {}",
                header,
                crate::zctx!()
            );
            return None;
        }
    };

    Some(body)
}

pub fn decoder<'a>(
    bytes: &'a [u8],
    sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = (Message<'a>, &'a [u8])> {
    let mut reader = &bytes[..];
    let mut reliability: Option<Reliability> = None;
    let mut qos: Option<QoS> = None;

    core::iter::from_fn(move || {
        let (data, start) = (reader.as_ptr(), reader.len());
        let msg = decode(&mut reader, &mut reliability, &mut qos, sn, resolution);
        let len = start - reader.len();
        msg.map(|msg| (msg, unsafe { core::slice::from_raw_parts(data, len) }))
    })
}

pub fn transport_decoder<'a>(
    bytes: &'a [u8],
    sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = (TransportMessage<'a>, &'a [u8])> {
    decoder(bytes, sn, resolution).filter_map(|m| match m.0 {
        Message::Transport(msg) => Some((msg, m.1)),
        _ => None,
    })
}

pub fn network_decoder<'a>(
    bytes: &'a [u8],
    sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = (NetworkMessage<'a>, &'a [u8])> {
    decoder(bytes, sn, resolution).filter_map(|m| match m.0 {
        Message::Network(msg) => Some((msg, m.1)),
        _ => None,
    })
}

fn encode<'a, 'b>(
    writer: &mut &'a mut [u8],
    msg: MessageRef<'b>,
    reliability: &mut Option<Reliability>,
    qos: &mut Option<QoS>,
    next_sn: &mut u32,
    resolution: Resolution,
) -> Option<usize> {
    let start = writer.len();

    match msg {
        MessageRef::Network(msg) => {
            let r = msg.reliability;
            let q = msg.qos;

            if reliability.as_ref() != Some(&r) || qos.as_ref() != Some(&q) {
                FrameHeader {
                    reliability: r,
                    sn: *next_sn,
                    qos: q,
                }
                .z_encode(writer)
                .ok()?;

                *reliability = Some(r);
                *qos = Some(q);
                // TODO: wrap with resolution
                let _ = resolution;
                *next_sn = next_sn.wrapping_add(1);
            }

            msg.body.z_encode(writer).ok()
        }
        MessageRef::Transport(msg) => msg.z_encode(writer).ok(),
    }?;

    Some(start - writer.len())
}

pub fn encoder<'a, 'b>(
    bytes: &'a mut [u8],
    mut msgs: impl Iterator<Item = Message<'b>>,
    next_sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = usize> {
    let mut writer = &mut bytes[..];
    let mut last_reliability: Option<Reliability> = None;
    let mut last_qos: Option<QoS> = None;
    core::iter::from_fn(move || {
        let msg = msgs.next()?;
        encode(
            &mut writer,
            msg.as_ref(),
            &mut last_reliability,
            &mut last_qos,
            next_sn,
            resolution,
        )
    })
}

pub fn encoder_ref<'a, 'b>(
    bytes: &'a mut [u8],
    mut msgs: impl Iterator<Item = MessageRef<'b>>,
    next_sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = usize> {
    let mut writer = &mut bytes[..];
    let mut last_reliability: Option<Reliability> = None;
    let mut last_qos: Option<QoS> = None;
    core::iter::from_fn(move || {
        let msg = msgs.next()?;
        encode(
            &mut writer,
            msg,
            &mut last_reliability,
            &mut last_qos,
            next_sn,
            resolution,
        )
    })
}

pub fn transport_encoder<'a, 'b>(
    bytes: &'a mut [u8],
    mut msgs: impl Iterator<Item = TransportMessage<'b>>,
) -> impl Iterator<Item = usize> {
    let mut writer = &mut bytes[..];
    core::iter::from_fn(move || {
        let msg = msgs.next()?;
        encode(
            &mut writer,
            MessageRef::Transport(&msg),
            &mut None,
            &mut None,
            &mut 0,
            Resolution::default(),
        )
    })
}

pub fn transport_encoder_ref<'a, 'b>(
    bytes: &'a mut [u8],
    mut msgs: impl Iterator<Item = &'b TransportMessage<'b>>,
) -> impl Iterator<Item = usize> {
    let mut writer = &mut bytes[..];
    core::iter::from_fn(move || {
        let msg = msgs.next()?;
        encode(
            &mut writer,
            MessageRef::Transport(msg),
            &mut None,
            &mut None,
            &mut 0,
            Resolution::default(),
        )
    })
}

pub fn network_encoder<'a, 'b>(
    bytes: &'a mut [u8],
    mut msgs: impl Iterator<Item = NetworkMessage<'b>>,
    next_sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = usize> {
    let mut writer = &mut bytes[..];
    let mut last_reliability: Option<Reliability> = None;
    let mut last_qos: Option<QoS> = None;
    core::iter::from_fn(move || {
        let msg = msgs.next()?;
        encode(
            &mut writer,
            MessageRef::Network(&msg),
            &mut last_reliability,
            &mut last_qos,
            next_sn,
            resolution,
        )
    })
}

pub fn network_encoder_ref<'a, 'b>(
    bytes: &'a mut [u8],
    mut msgs: impl Iterator<Item = &'b NetworkMessage<'b>>,
    next_sn: &mut u32,
    resolution: Resolution,
) -> impl Iterator<Item = usize> {
    let mut writer = &mut bytes[..];
    let mut last_reliability: Option<Reliability> = None;
    let mut last_qos: Option<QoS> = None;
    core::iter::from_fn(move || {
        let msg = msgs.next()?;
        encode(
            &mut writer,
            MessageRef::Network(msg),
            &mut last_reliability,
            &mut last_qos,
            next_sn,
            resolution,
        )
    })
}
