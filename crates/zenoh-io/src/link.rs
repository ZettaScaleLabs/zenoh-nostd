use crate::ZLinkManager;

mod impls;

pub trait ZLinkInfo {
    fn mtu(&self) -> u16;
    fn is_streamed(&self) -> bool;
}

pub trait ZLinkTx {
    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>>;
}

pub trait ZLinkRx {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = core::result::Result<usize, zenoh_proto::LinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>>;
}

pub trait ZLink: ZLinkInfo + ZLinkTx + ZLinkRx {
    type Tx<'a>: ZLinkTx + ZLinkInfo
    where
        Self: 'a;

    type Rx<'a>: ZLinkRx + ZLinkInfo
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);
}

macro_rules! impl_link_traits {
    ($struct:ident<$($lt:lifetime),*>: ZLinkInfo, $($variant:ident<$($lt2:lifetime),*>),+) => {
        impl<$($lt,)* LinkManager: ZLinkManager> ZLinkInfo for $struct<$($lt,)* LinkManager>
        where
            $(LinkManager::$variant<$($lt2,)*>: ZLinkInfo,)+
        {
            fn mtu(&self) -> u16 {
                delegate_variants!($struct(self), mtu(), $($variant),+)
            }

            fn is_streamed(&self) -> bool {
                delegate_variants!($struct(self), is_streamed(), $($variant),+)
            }
        }
    };

    ($struct:ident<$($lt:lifetime),*>: ZLinkTx, $($variant:ident<$($lt2:lifetime),*>),+) => {
        impl<$($lt,)* LinkManager: ZLinkManager> ZLinkTx for $struct<$($lt,)* LinkManager>
        where
            $(LinkManager::$variant<$($lt2,)*>: ZLinkTx,)+
        {
            async fn write_all(&mut self, buffer: &[u8]) -> Result<(), zenoh_proto::LinkError> {
                delegate_variants!($struct(self), write_all(buffer).await, $($variant),+)
            }
        }
    };

    ($struct:ident<$($lt:lifetime),*>: ZLinkRx, $($variant:ident<$($lt2:lifetime),*>),+) => {
        impl<$($lt,)* LinkManager: ZLinkManager> ZLinkRx for $struct<$($lt,)* LinkManager>
        where
            $(LinkManager::$variant<$($lt2,)*>: ZLinkRx,)+
        {
            async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, zenoh_proto::LinkError> {
                delegate_variants!($struct(self), read(buffer).await, $($variant),+)
            }

            async fn read_exact(&mut self, buffer: &mut [u8]) -> Result<(), zenoh_proto::LinkError> {
                delegate_variants!($struct(self), read_exact(buffer).await, $($variant),+)
            }
        }
    };

    ($struct:ident<$lt1:lifetime>: ZLink, $($variant:ident<$lt2:lifetime>),+) => {
        impl<$lt1, LinkManager: ZLinkManager> ZLink for $struct<$lt1, LinkManager>
        where
            $(LinkManager::$variant<$lt2>: ZLink,)+
        {
            type Tx<'a> = LinkTx<$lt1, 'a, LinkManager> where Self: 'a;
            type Rx<'a> = LinkRx<$lt1, 'a, LinkManager> where Self: 'a;

            fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
                match self {
                    $(
                        $struct:: $variant (link) => {
                            let (tx, rx) = link.split();
                            (LinkTx:: $variant(tx), LinkRx:: $variant(rx))
                        },
                    )+
                }
            }
        }
    };
}

macro_rules! delegate_variants {
    ($struct:ident($self:ident), $method:ident($arg:ident).await, $($variant:ident),+) => {
        match $self {
            $($struct:: $variant (link) => link.$method($arg).await,)+
        }
    };

    ($struct:ident($self:ident), $method:ident(), $($variant:ident),+) => {
        match $self {
            $($struct:: $variant (link) => link.$method(),)+
        }
    };
}

macro_rules! define {
    ($($variant:ident),* $(,)?) => {
        pub enum Link<'p, LinkManager: ZLinkManager> {
            $(
                $variant(LinkManager::$variant<'p>),
            )*
        }

        impl_link_traits! { Link<'p>: ZLinkInfo, $($variant<'p>),* }
        impl_link_traits! { Link<'p>: ZLinkTx, $($variant<'p>),* }
        impl_link_traits! { Link<'p>: ZLinkRx, $($variant<'p>),* }
        impl_link_traits! { Link<'p>: ZLink, $($variant<'p>),* }

        pub enum LinkTx<'p, 'a, LinkManager: ZLinkManager>
        where
            Self: 'a,
        {
            $(
                $variant(<LinkManager::$variant<'p> as ZLink>::Tx<'a>),
            )*
        }

        impl_link_traits! { LinkTx<'p, 'a>: ZLinkInfo, $($variant<'p>),* }
        impl_link_traits! { LinkTx<'p, 'a>: ZLinkTx, $($variant<'p>),* }

        pub enum LinkRx<'p, 'a, LinkManager: ZLinkManager>
        where
            Self: 'a,
        {
            $(
                $variant(<LinkManager::$variant<'p> as ZLink>::Rx<'a>),
            )*
        }

        impl_link_traits! { LinkRx<'p, 'a>: ZLinkInfo, $($variant<'p>),* }
        impl_link_traits! { LinkRx<'p, 'a>: ZLinkRx, $($variant<'p>),* }
    };
}

define!(Tcp, Udp, Ws, Serial);

// pub enum Link<'p, LinkManager: ZLinkManager> {
//     Tcp(LinkManager::Tcp<'p>),
//     Udp(LinkManager::Udp<'p>),
//     Ws(LinkManager::Ws<'p>),
//     Serial(LinkManager::Serial<'p>),
// }

// impl_link_traits! { Link<'p>: ZLinkInfo, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }
// impl_link_traits! { Link<'p>: ZLinkTx, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }
// impl_link_traits! { Link<'p>: ZLinkRx, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }
// impl_link_traits! { Link<'p>: ZLink, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }

// pub enum LinkTx<'p, 'a, LinkManager: ZLinkManager>
// where
//     Self: 'a,
// {
//     Tcp(<LinkManager::Tcp<'p> as ZLink>::Tx<'a>),
//     Udp(<LinkManager::Udp<'p> as ZLink>::Tx<'a>),
//     Ws(<LinkManager::Ws<'p> as ZLink>::Tx<'a>),
//     Serial(<LinkManager::Serial<'p> as ZLink>::Tx<'a>),
// }

// impl_link_traits! { LinkTx<'p, 'a>: ZLinkInfo, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }
// impl_link_traits! { LinkTx<'p, 'a>: ZLinkTx, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }

// pub enum LinkRx<'p, 'a, LinkManager: ZLinkManager>
// where
//     Self: 'a,
// {
//     Tcp(<LinkManager::Tcp<'p> as ZLink>::Rx<'a>),
//     Udp(<LinkManager::Udp<'p> as ZLink>::Rx<'a>),
//     Ws(<LinkManager::Ws<'p> as ZLink>::Rx<'a>),
//     Serial(<LinkManager::Serial<'p> as ZLink>::Rx<'a>),
// }

// impl_link_traits! { LinkRx<'p, 'a>: ZLinkInfo, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }
// impl_link_traits! { LinkRx<'p, 'a>: ZLinkRx, Tcp<'p>, Udp<'p>, Ws<'p>, Serial<'p> }
