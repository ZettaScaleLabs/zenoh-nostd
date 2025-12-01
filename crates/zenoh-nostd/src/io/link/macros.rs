macro_rules! impl_link_tx {
    ($name:ident, $trait_bound:path, $field:ident) => {
        impl<T: $trait_bound> ZLinkTx for $name<T> {
            async fn write(&mut self, buffer: &[u8]) -> crate::ZResult<usize, crate::ZLinkError> {
                self.$field.write(buffer).await
            }

            async fn write_all(&mut self, buffer: &[u8]) -> crate::ZResult<(), crate::ZLinkError> {
                self.$field.write_all(buffer).await
            }
        }
    };
}

macro_rules! impl_link_rx {
    ($name:ident, $trait_bound:path, $field:ident) => {
        impl<T: $trait_bound> ZLinkRx for $name<T> {
            async fn read(
                &mut self,
                buffer: &mut [u8],
            ) -> crate::ZResult<usize, crate::ZLinkError> {
                self.$field.read(buffer).await
            }

            async fn read_exact(
                &mut self,
                buffer: &mut [u8],
            ) -> crate::ZResult<(), crate::ZLinkError> {
                self.$field.read_exact(buffer).await
            }
        }
    };
}

pub(crate) use impl_link_rx;
pub(crate) use impl_link_tx;

macro_rules! define_link {
    ($name:ident, $trait_bound:path, $field:ident, $streamed:expr, tx) => {
        $crate::io::link::macros::define_link!(@base $name, $trait_bound, $field, $streamed);
        $crate::io::link::macros::impl_link_tx!($name, $trait_bound, $field);
    };

    ($name:ident, $trait_bound:path, $field:ident, $streamed:expr, rx) => {
        $crate::io::link::macros::define_link!(@base $name, $trait_bound, $field, $streamed);
        $crate::io::link::macros::impl_link_rx!($name, $trait_bound, $field);
    };

    ($name:ident, $trait_bound:path, $field:ident, $streamed:expr, both) => {
        $crate::io::link::macros::define_link!(@base $name, $trait_bound, $field, $streamed);
        $crate::io::link::macros::impl_link_tx!($name, $trait_bound, $field);
        $crate::io::link::macros::impl_link_rx!($name, $trait_bound, $field);
    };

    (@base $name:ident, $trait_bound:path, $field:ident, $streamed:expr) => {
        pub struct $name<T: $trait_bound> {
            $field: T,
            mtu: u16,
        }

        impl<T: $trait_bound> ZLinkInfo for $name<T> {
            fn mtu(&self) -> u16 { self.mtu }
            fn is_streamed(&self) -> bool { $streamed }
        }
    };
}

pub(crate) use define_link;
