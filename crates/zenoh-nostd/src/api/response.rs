use zenoh_proto::keyexpr;

pub struct Response {
    ok: bool,
    keyexpr: *const keyexpr,
    payload: *const [u8],
}

impl Response {
    pub(crate) fn new(ok: bool, keyexpr: &keyexpr, payload: &[u8]) -> Self {
        Self {
            ok,
            keyexpr,
            payload,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// # Safety
    ///
    /// The caller must ensure that the pointers are valid for the lifetime of the Sample.
    pub unsafe fn keyexpr(&self) -> &keyexpr {
        unsafe { &*self.keyexpr }
    }

    /// # Safety
    ///
    /// The caller must ensure that the pointers are valid for the lifetime of the Sample.
    pub unsafe fn payload(&self) -> &[u8] {
        unsafe { &*self.payload }
    }
}

pub type ResponsePtr = *const Response;

pub struct ResponseRef<'a> {
    ok: bool,
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> ResponseRef<'a> {
    /// # Safety
    ///
    /// The caller must ensure that the pointers are valid for the lifetime of the Sample.
    pub unsafe fn new(sample: &ResponsePtr) -> Self {
        unsafe {
            let sample = &**sample;

            Self {
                ok: sample.is_ok(),
                keyexpr: sample.keyexpr(),
                payload: sample.payload(),
            }
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

impl From<ResponsePtr> for ResponseRef<'_> {
    fn from(sample: ResponsePtr) -> Self {
        unsafe { Self::new(&sample) }
    }
}

impl From<ResponseRef<'_>> for Response {
    fn from(sample: ResponseRef<'_>) -> Self {
        Self::new(sample.is_ok(), sample.keyexpr(), sample.payload())
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

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<ResponsePtr>
    for HeaplessResponse<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::CollectionError;

    fn try_from(sample: ResponsePtr) -> core::result::Result<Self, Self::Error> {
        let sample = unsafe { ResponseRef::new(&sample) };
        Self::new(sample.is_ok(), sample.keyexpr(), sample.payload())
    }
}
