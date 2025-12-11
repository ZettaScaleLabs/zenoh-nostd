use higher_kinded_types::ForLt;
use zenoh_proto::keyexpr;

pub(crate) type SampleRef = ForLt!(<'a> = &'a Sample<'a>);

pub struct Sample<'a> {
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> Sample<'a> {
    pub fn new(keyexpr: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self { keyexpr, payload }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.keyexpr
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }
}

pub struct HeaplessSample<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    keyexpr: heapless::String<MAX_KEYEXPR>,
    payload: heapless::Vec<u8, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> HeaplessSample<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub fn new(
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
            keyexpr: ke_str,
            payload: pl_vec,
        })
    }

    pub fn keyexpr(&self) -> &keyexpr {
        // SAFETY: we ensure that the keyexpr is always valid
        unsafe { keyexpr::new(self.keyexpr.as_str()).unwrap_unchecked() }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<&Sample<'_>>
    for HeaplessSample<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::CollectionError;

    fn try_from(sample: &Sample<'_>) -> core::result::Result<Self, Self::Error> {
        Self::new(sample.keyexpr(), sample.payload())
    }
}
