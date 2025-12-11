use elain::{Align, Alignment};
use zenoh_proto::keyexpr;

use crate::api::AsyncCallback;

pub struct Sample {
    keyexpr: *const keyexpr,
    payload: *const [u8],
}

impl Sample {
    pub(crate) fn new(keyexpr: &keyexpr, payload: &[u8]) -> Self {
        Self { keyexpr, payload }
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

pub type SamplePtr = *const Sample;

pub struct SampleRef<'a> {
    keyexpr: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> SampleRef<'a> {
    /// # Safety
    ///
    /// The caller must ensure that the pointers are valid for the lifetime of the Sample.
    pub unsafe fn new(sample: &SamplePtr) -> Self {
        unsafe {
            let sample = &**sample;

            Self {
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
        unsafe { Self::new(&sample) }
    }
}

impl From<SampleRef<'_>> for Sample {
    fn from(sample: SampleRef<'_>) -> Self {
        Self::new(sample.keyexpr(), sample.payload())
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

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> TryFrom<SamplePtr>
    for HeaplessSample<MAX_KEYEXPR, MAX_PAYLOAD>
{
    type Error = crate::CollectionError;

    fn try_from(sample: SamplePtr) -> core::result::Result<Self, Self::Error> {
        let sample = unsafe { SampleRef::new(&sample) };
        Self::new(sample.keyexpr(), sample.payload())
    }
}

impl<
    const CALLBACK_SIZE: usize,
    const CALLBACK_ALIGN: usize,
    const FUTURE_SIZE: usize,
    const FUTURE_ALIGN: usize,
> AsyncCallback<SamplePtr, (), CALLBACK_SIZE, CALLBACK_ALIGN, FUTURE_SIZE, FUTURE_ALIGN>
where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    pub fn new_sync_sub(f: impl Fn(&SampleRef<'_>) -> ()) -> Self {
        Self::new_sync(move |sample_ptr: SamplePtr| {
            let sample_ref = sample_ptr.into();
            f(&sample_ref)
        })
    }

    pub fn new_async_sub(f: impl AsyncFn(&SampleRef<'_>) -> ()) -> Self {
        Self::new_async(async move |sample_ptr: SamplePtr| {
            let sample_ref = sample_ptr.into();
            f(&sample_ref).await
        })
    }
}
