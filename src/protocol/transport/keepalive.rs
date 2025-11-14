use crate::ZStruct;

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:3|ID:5=0x04")]
pub struct KeepAlive;

impl KeepAlive {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut crate::ZWriter) -> Self {
        Self {}
    }
}
