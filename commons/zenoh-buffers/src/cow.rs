use heapless::Vec;

pub enum CowBytes<'a, const N: usize> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8, N>),
}
