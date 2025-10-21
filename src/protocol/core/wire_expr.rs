use core::{convert::TryInto, fmt};

use heapless::String;

use crate::{
    protocol::{
        ZCodecError, ZProtocolError,
        keyexpr::borrowed::keyexpr,
        network::Mapping,
        zcodec::{decode_str, decode_u16, encode_str, encode_u16},
    },
    result::ZResult,
    zbuf::{ZBufReader, ZBufWriter},
};

pub(crate) type ExprId = u16;
pub(crate) type ExprLen = u16;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct WireExpr<'a> {
    pub(crate) scope: ExprId,
    pub(crate) suffix: &'a str,
    pub(crate) mapping: Mapping,
}

impl<'a> WireExpr<'a> {
    pub(crate) fn is_empty(&self) -> bool {
        self.scope == 0 && self.suffix.is_empty()
    }

    pub(crate) fn try_as_id(&self) -> crate::result::ZResult<ExprId, ZProtocolError> {
        if self.has_suffix() {
            crate::zbail!(ZProtocolError::CouldNotParse);
        } else {
            Ok(self.scope)
        }
    }

    pub(crate) fn has_suffix(&self) -> bool {
        !self.suffix.is_empty()
    }

    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        encode_u16(writer, self.scope)?;

        if !self.suffix.is_empty() {
            encode_str(writer, true, self.suffix)?;
        }

        Ok(())
    }

    pub(crate) fn decode(
        condition: bool,
        reader: &mut ZBufReader<'a>,
    ) -> ZResult<Self, ZCodecError> {
        let scope = decode_u16(reader)?;

        let suffix = if condition {
            decode_str(reader, None)?
        } else {
            ""
        };

        Ok(WireExpr {
            scope,
            suffix,
            mapping: Mapping::DEFAULT,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use crate::zbuf::BufWriterExt;
        use rand::Rng;
        use rand::distributions::Alphanumeric;
        use rand::distributions::DistString;

        const MIN: usize = 2;
        const MAX: usize = 32;

        let mut rng = rand::thread_rng();

        let scope: ExprId = rng.gen_range(0..20);
        let suffix: &'a str = if rng.gen_bool(0.5) {
            let len = rng.gen_range(MIN..MAX);
            let proto = Alphanumeric.sample_string(&mut rng, len);
            zbuf.write_str_return(proto.as_str()).unwrap()
        } else {
            ""
        };

        WireExpr {
            scope,
            suffix: suffix.into(),
            mapping: Mapping::DEFAULT,
        }
    }
}

impl TryInto<ExprId> for WireExpr<'_> {
    type Error = ZProtocolError;
    fn try_into(self) -> crate::result::ZResult<ExprId, ZProtocolError> {
        self.try_as_id()
    }
}

impl From<ExprId> for WireExpr<'_> {
    fn from(scope: ExprId) -> Self {
        Self {
            scope,
            suffix: "",
            mapping: Mapping::Sender,
        }
    }
}

impl<'a> From<&'a keyexpr> for WireExpr<'a> {
    fn from(val: &'a keyexpr) -> Self {
        WireExpr {
            scope: 0,
            suffix: val.as_str(),
            mapping: Mapping::Sender,
        }
    }
}

impl fmt::Display for WireExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.scope == 0 {
            write!(f, "{}", self.suffix)
        } else {
            write!(f, "{}:{:?}:{}", self.scope, self.mapping, self.suffix)
        }
    }
}

impl<'a> From<&WireExpr<'a>> for WireExpr<'a> {
    #[inline]
    fn from(key: &WireExpr<'a>) -> WireExpr<'a> {
        key.clone()
    }
}

impl<'a> From<&'a str> for WireExpr<'a> {
    #[inline]
    fn from(name: &'a str) -> WireExpr<'a> {
        WireExpr {
            scope: 0,
            suffix: name,
            mapping: Mapping::Sender,
        }
    }
}

impl<'a, const N: usize> From<&'a String<N>> for WireExpr<'a> {
    #[inline]
    fn from(name: &'a String<N>) -> WireExpr<'a> {
        WireExpr {
            scope: 0,
            suffix: name.as_str(),
            mapping: Mapping::Sender,
        }
    }
}
