use rand::distributions::{Alphanumeric, DistString};
use zenoh_buffer::ZBuf;
use zenoh_protocol::core::{
    encoding::{Encoding, EncodingId},
    locator::Locator,
};
use zenoh_result::ZResult;

pub trait RandomProtocol<'a> {
    fn rand(buffer: Option<&'a mut [u8]>) -> ZResult<Self>
    where
        Self: Sized;
}

impl<'a, const N: usize> RandomProtocol<'a> for Locator<N> {
    fn rand(_: Option<&'a mut [u8]>) -> ZResult<Self>
    where
        Self: Sized,
    {
        let mut rng = rand::thread_rng();

        let str1 = Alphanumeric.sample_string(&mut rng, N / 2 - 1);
        let str2 = Alphanumeric.sample_string(&mut rng, N / 2 - 1);

        Locator::new(str1, str2)
    }
}
impl<'a> RandomProtocol<'a> for Encoding<'a> {
    fn rand(buffer: Option<&'a mut [u8]>) -> ZResult<Self>
    where
        Self: Sized,
    {
        use rand::Rng;

        const MIN: usize = 2;
        const MAX: usize = 16;

        let mut rng = rand::thread_rng();

        let id: EncodingId = rng.gen();
        let schema = rng.gen_bool(0.5).then_some(rng.gen_range(MIN..=MAX));

        let schema = schema.and_then(|len| {
            buffer.and_then(|buf| {
                let _ = 3;

                rng.fill(&mut buf[..len]);

                Some(ZBuf(&buf[..len]))
            })
        });

        Ok(Encoding { id, schema })
    }
}
