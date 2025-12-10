use zenoh_proto::keyexpr;

pub struct Sample {
    keyexpr: *const keyexpr,
    payload: *const [u8],
}

impl Sample {
    pub(crate) fn new(keyexpr: &keyexpr, payload: &[u8]) -> Sample {
        Sample { keyexpr, payload }
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

    /// # Safety
    ///
    /// The caller must ensure that the pointers are valid for the lifetime of the Sample.
    pub unsafe fn as_ref(&self) -> SampleRef<'_> {
        unsafe {
            SampleRef {
                keyexpr: self.keyexpr(),
                payload: self.payload(),
            }
        }
    }
}

pub type SamplePtr = *const Sample;

pub struct SampleRef<'a> {
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> SampleRef<'a> {
    /// # Safety
    ///
    /// The caller must ensure that the pointers are valid for the lifetime of the Sample.
    pub unsafe fn new(sample: &SamplePtr) -> SampleRef<'a> {
        unsafe {
            let sample = &**sample;

            SampleRef {
                keyexpr: sample.keyexpr(),
                payload: sample.payload(),
            }
        }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.keyexpr
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }
}

impl From<SamplePtr> for SampleRef<'_> {
    fn from(sample: SamplePtr) -> Self {
        unsafe { SampleRef::new(&sample) }
    }
}

impl From<SampleRef<'_>> for Sample {
    fn from(sample: SampleRef<'_>) -> Self {
        Sample::new(sample.keyexpr(), sample.payload())
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

        Ok(HeaplessSample {
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

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<SamplePtr>
    for HeaplessSample<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::CollectionError;

    fn try_from(sample: SamplePtr) -> core::result::Result<Self, Self::Error> {
        let sample = unsafe { SampleRef::new(&sample) };
        HeaplessSample::new(sample.keyexpr(), sample.payload())
    }
}
