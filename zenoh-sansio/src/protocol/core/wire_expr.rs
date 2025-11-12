use crate::{ZExt, network::Mapping};

#[cfg(test)]
use crate::ZWriterExt;
#[cfg(test)]
use rand::{
    Rng,
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

#[derive(ZExt, Debug, PartialEq)]
#[zenoh(header = "_:6|M|N|")]
pub struct WireExpr<'a> {
    pub scope: u16,

    #[zenoh(header = M)]
    pub mapping: Mapping,

    #[zenoh(presence = header(N), default = "", size = prefixed)]
    pub suffix: &'a str,
}

impl<'a> WireExpr<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let scope = thread_rng().r#gen();
        let mapping = Mapping::rand();

        let suffix = if thread_rng().gen_bool(0.5) {
            let suffix =
                Alphanumeric.sample_string(&mut thread_rng(), thread_rng().gen_range(1..16));
            w.write_str(&suffix).unwrap()
        } else {
            ""
        };

        Self {
            scope,
            mapping,
            suffix,
        }
    }
}
