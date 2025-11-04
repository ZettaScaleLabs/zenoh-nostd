use ryu_derive::ZExt;

#[derive(ZExt, Debug, PartialEq)]
pub struct Patch {
    pub patch: u64,
}

impl Patch {
    #[cfg(test)]
    pub(crate) fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Patch { patch: rng.r#gen() }
    }
}
