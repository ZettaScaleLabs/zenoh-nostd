use ryu_derive::ZStruct;

#[derive(ZStruct)]
pub struct Put<'a> {
    pub payload: &'a [u8],
}

impl Put<'_> {}
