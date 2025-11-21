use crate::ZSample;

pub enum ZReply<'a> {
    Ok(ZSample<'a>),
    Err(ZSample<'a>),
}
