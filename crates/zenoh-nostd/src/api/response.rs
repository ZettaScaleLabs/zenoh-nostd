use higher_kinded_types::ForLt;
use zenoh_proto::keyexpr;

pub(crate) type ResponseRef = ForLt!(<'a> = &'a Response<'a>);

pub struct Response<'a> {
    ok: bool,
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> Response<'a> {
    pub fn new(ok: bool, keyexpr: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self {
            ok,
            keyexpr,
            payload,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.ok
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.keyexpr
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }
}

pub struct HeaplessResponse<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    ok: bool,
    keyexpr: heapless::String<MAX_KEYEXPR>,
    payload: heapless::Vec<u8, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize>
    HeaplessResponse<MAX_KEYEXPR, MAX_PAYLOAD>
{
    pub fn new(
        ok: bool,
        keyexpr: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<Self, crate::CollectionError> {
        let mut ke_str = heapless::String::<MAX_KEYEXPR>::new();
        ke_str
            .push_str(keyexpr.as_str())
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        let mut pl_vec = heapless::Vec::<u8, MAX_PAYLOAD>::new();
        pl_vec
            .extend_from_slice(payload)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        Ok(Self {
            ok,
            keyexpr: ke_str,
            payload: pl_vec,
        })
    }

    pub fn is_ok(&self) -> bool {
        self.ok
    }

    pub fn keyexpr(&self) -> &keyexpr {
        // SAFETY: we ensure that the keyexpr is always valid
        unsafe { keyexpr::new(self.keyexpr.as_str()).unwrap_unchecked() }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<&Response<'_>>
    for HeaplessResponse<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::CollectionError;

    fn try_from(sample: &Response<'_>) -> core::result::Result<Self, Self::Error> {
        Self::new(sample.is_ok(), sample.keyexpr(), sample.payload())
    }
}
