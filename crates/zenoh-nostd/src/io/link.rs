use core::net::SocketAddr;

mod impls;

pub trait ZLinkManager: Sized {
    type Tcp<'ext>: ZLink;
    type Udp<'ext>: ZLink;
    type Ws<'ext>: ZLink;
    type Serial<'ext>: ZLink;

    fn connect_tcp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::LinkError>> {
        async move { unimplemented!("{addr}") }
    }

    fn listen_tcp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::LinkError>> {
        async move { unimplemented!("{addr}") }
    }

    fn connect_udp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::LinkError>> {
        async move { unimplemented!("{addr}") }
    }

    fn listen_udp(
        &self,
        addr: &SocketAddr,
    ) -> impl Future<Output = core::result::Result<Link<'_, Self>, zenoh_proto::LinkError>> {
        async move { unimplemented!("{addr}") }
    }
}

pub trait ZLinkInfo {
    fn mtu(&self) -> u16;
    fn is_streamed(&self) -> bool;
}

pub trait ZLinkTx: ZLinkInfo {
    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>>;
}

pub trait ZLinkRx: ZLinkInfo {
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
    type Tx<'link>: ZLinkTx + ZLinkInfo
    where
        Self: 'link;

    type Rx<'link>: ZLinkRx + ZLinkInfo
    where
        Self: 'link;

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
            type Tx<'link> = LinkTx<$lt1, 'link, LinkManager> where Self: 'link;
            type Rx<'link> = LinkRx<$lt1, 'link, LinkManager> where Self: 'link;

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
        pub enum Link<'ext, LinkManager: ZLinkManager> {
            $(
                $variant(LinkManager::$variant<'ext>),
            )*
        }

        impl_link_traits! { Link<'ext>: ZLinkInfo, $($variant<'ext>),* }
        impl_link_traits! { Link<'ext>: ZLinkTx, $($variant<'ext>),* }
        impl_link_traits! { Link<'ext>: ZLinkRx, $($variant<'ext>),* }
        impl_link_traits! { Link<'ext>: ZLink, $($variant<'ext>),* }

        pub enum LinkTx<'ext, 'link, LinkManager: ZLinkManager>
        where
            Self: 'link,
        {
            $(
                $variant(<LinkManager::$variant<'ext> as ZLink>::Tx<'link>),
            )*
        }

        impl_link_traits! { LinkTx<'ext, 'link>: ZLinkInfo, $($variant<'ext>),* }
        impl_link_traits! { LinkTx<'ext, 'link>: ZLinkTx, $($variant<'ext>),* }

        pub enum LinkRx<'ext, 'link, LinkManager: ZLinkManager>
        where
            Self: 'link,
        {
            $(
                $variant(<LinkManager::$variant<'ext> as ZLink>::Rx<'link>),
            )*
        }

        impl_link_traits! { LinkRx<'ext, 'link>: ZLinkInfo, $($variant<'ext>),* }
        impl_link_traits! { LinkRx<'ext, 'link>: ZLinkRx, $($variant<'ext>),* }
    };
}

define!(Tcp, Udp, Ws, Serial);
