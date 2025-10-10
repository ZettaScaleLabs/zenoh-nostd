use crate::{protocol::core::wire_expr::WireExpr, zbuf::ZBuf};

pub struct ZSample<'a> {
    keyexpr: WireExpr<'a>,
    payload: ZBuf<'a>,
}

impl<'a> ZSample<'a> {
    pub fn new(keyexpr: WireExpr<'a>, payload: ZBuf<'a>) -> Self {
        Self { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &WireExpr<'a> {
        &self.keyexpr
    }

    pub fn payload(&self) -> &ZBuf<'a> {
        &self.payload
    }
}
