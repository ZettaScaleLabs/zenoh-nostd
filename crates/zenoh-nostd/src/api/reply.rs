use crate::api::Sample;

pub enum ZReply<'a> {
    Ok(Sample<'a>),
    Err(Sample<'a>),
}
