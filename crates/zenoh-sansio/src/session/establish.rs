use core::time::Duration;

use sha3::{
    Shake128,
    digest::{ExtendableOutput, Update, XofReader},
};
use zenoh_proto::{
    Bits, Field, Resolution, ZResult, ZenohIdProto,
    transport::{
        init::InitAck,
        open::{OpenAck, OpenExt, OpenSyn},
    },
};

use crate::{
    ZSessionError,
    event::{Event, EventInner},
    session::{MineConfig, NegotiatedConfig, OtherConfig, SessionState},
};

pub(super) fn negotiate_sn(
    mine_zid: &ZenohIdProto,
    other_zid: &ZenohIdProto,
    resolution: &Resolution,
) -> u32 {
    const RES_U8: u32 = (u8::MAX >> 1) as u32; // 1 byte max when encoded
    const RES_U16: u32 = (u16::MAX >> 2) as u32; // 2 bytes max when encoded
    const RES_U32: u32 = u32::MAX >> 4; // 4 bytes max when encoded
    const RES_U64: u32 = (u64::MAX >> 1) as u32; // 9 bytes max when encoded

    fn get_mask(resolution: Bits) -> u32 {
        match resolution {
            Bits::U8 => RES_U8,
            Bits::U16 => RES_U16,
            Bits::U32 => RES_U32,
            Bits::U64 => RES_U64,
        }
    }

    let mut hasher = Shake128::default();
    hasher.update(&mine_zid.as_le_bytes()[..mine_zid.size()]);
    hasher.update(&other_zid.as_le_bytes()[..other_zid.size()]);
    let mut array = 0_u32.to_le_bytes();
    hasher.finalize_xof().read(&mut array);
    u32::from_le_bytes(array) & get_mask(resolution.get(Field::FrameSN))
}

pub(super) fn negotiate_resolution(
    mine_resolution: &Resolution,
    other_resolution: &Resolution,
) -> ZResult<Resolution, ZSessionError> {
    let mut res = Resolution::default();

    let i_fsn_res = other_resolution.get(Field::FrameSN);
    let m_fsn_res = mine_resolution.get(Field::FrameSN);

    if i_fsn_res > m_fsn_res {
        return Err(ZSessionError::InvalidArgument);
    }

    res.set(Field::FrameSN, i_fsn_res);

    let i_rid_res = other_resolution.get(Field::RequestID);
    let m_rid_res = mine_resolution.get(Field::RequestID);

    if i_rid_res > m_rid_res {
        return Err(ZSessionError::InvalidArgument);
    }

    res.set(Field::RequestID, i_rid_res);

    Ok(res)
}

pub(super) fn negotiate_batch_size(
    mine_batch_size: u16,
    other_batch_size: u16,
) -> ZResult<u16, ZSessionError> {
    Ok(core::cmp::min(mine_batch_size, other_batch_size))
}

pub(super) fn handle_init_ack<'a>(
    mine: MineConfig,
    ack: InitAck<'a>,
) -> ZResult<(SessionState, Event<'a>), ZSessionError> {
    let other_zid = ack.identifier.zid.clone();
    let mine_lease = mine.mine_lease.clone();

    let negotiated_resolution =
        negotiate_resolution(&mine.mine_resolution, &ack.resolution.resolution)?;
    let negotiated_sn = negotiate_sn(&mine.mine_zid, &other_zid, &negotiated_resolution);
    let negotiated_batch_size =
        negotiate_batch_size(mine.mine_batch_size, ack.resolution.batch_size.0)?;

    let state = SessionState::Connecting {
        negotiated: NegotiatedConfig {
            negotiated_sn,
            _negotiated_resolution: negotiated_resolution,
            _negotiated_batch_size: negotiated_batch_size,
        },
        mine: mine.clone(),
        other_zid,
    };

    let event = Event {
        inner: EventInner::OpenSyn(OpenSyn {
            lease: mine_lease,
            sn: negotiated_sn,
            cookie: ack.cookie,
            ext: OpenExt::DEFAULT,
        }),
    };

    Ok((state, event))
}

pub(super) fn handle_open_ack(
    time: Duration,
    mine: MineConfig,
    negotiated: NegotiatedConfig,
    other_zid: ZenohIdProto,
    ack: OpenAck,
) -> SessionState {
    SessionState::Connected {
        next_recv_keepalive: time + ack.lease,
        next_send_keepalive: time + mine.mine_lease / 4,

        mine,
        other: OtherConfig {
            _other_zid: other_zid,
            _other_sn: ack.sn,
            other_lease: ack.lease,
        },
        negotiated,
    }
}
