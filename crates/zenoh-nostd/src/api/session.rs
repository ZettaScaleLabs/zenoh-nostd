use crate::{
    api::driver::Driver,
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
    platform::ZPlatform,
};
use embassy_time::Duration;

pub(crate) mod driver;
mod resources;
pub use resources::*;
use zenoh_proto::EndPoint;

mod put;
mod run;
mod sub;

#[macro_export]
macro_rules! zimport_types {
    (
        PLATFORM: $platform:ty,
        TX: $txbuf:ty,
        RX: $rxbuf:ty,

        MAX_KEYEXPR_LEN: $max_keyexpr_len:expr,
        MAX_PARAMETERS_LEN: $max_parameters_len:expr,
        MAX_PAYLOAD_LEN: $max_payload_len:expr,

        MAX_QUEUED: $max_queued:expr,
        MAX_CALLBACKS: $max_callbacks:expr,

        MAX_SUBSCRIBERS: $max_subscribers:expr,
    ) => {
        type Config = $crate::api::PublicConfig<$platform, $txbuf, $rxbuf>;

        type Resources<'a> = $crate::api::PublicResources<
            'a,
            $platform,
            $txbuf,
            $rxbuf,
            $crate::api::SessionResources<
                $max_keyexpr_len,
                $max_parameters_len,
                $max_payload_len,
                $max_queued,
                $max_callbacks,
                $max_subscribers,
            >,
        >;

        type Session<'a> = $crate::api::PublicSession<
            'a,
            $platform,
            $txbuf,
            $rxbuf,
            $crate::api::SessionResources<
                $max_keyexpr_len,
                $max_parameters_len,
                $max_payload_len,
                $max_queued,
                $max_callbacks,
                $max_subscribers,
            >,
        >;
    };
}

pub struct PublicConfig<Platform, TxBuf, RxBuf> {
    platform: Platform,
    tx: TxBuf,
    rx: RxBuf,
}

impl<Platform, TxBuf, RxBuf> PublicConfig<Platform, TxBuf, RxBuf> {
    pub fn new(platform: Platform, tx_buf: TxBuf, rx_buf: RxBuf) -> Self {
        Self {
            platform,
            tx: tx_buf,
            rx: rx_buf,
        }
    }
}

pub struct PublicSession<'a, Platform, TxBuf, RxBuf, Resources>
where
    Platform: ZPlatform,
{
    pub(crate) driver: &'a Driver<'a, Platform, TxBuf, RxBuf>,
    pub(crate) resources: &'a Resources,
}

impl<'a, Platform, TxBuf, RxBuf, Resources> Clone
    for PublicSession<'a, Platform, TxBuf, RxBuf, Resources>
where
    Platform: ZPlatform,
{
    fn clone(&self) -> Self {
        PublicSession {
            driver: self.driver,
            resources: self.resources,
        }
    }
}

pub async fn open<'a, Platform, TxBuf, RxBuf, SessionResources>(
    resources: &'a mut PublicResources<'a, Platform, TxBuf, RxBuf, SessionResources>,
    mut config: PublicConfig<Platform, TxBuf, RxBuf>,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<PublicSession<'a, Platform, TxBuf, RxBuf, SessionResources>>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
    RxBuf: AsMut<[u8]>,
{
    let link = Link::new(&config.platform, endpoint).await?;

    let (transport, tconfig) = Transport::open(
        link,
        TransportMineConfig {
            mine_zid: Default::default(),
            mine_lease: Duration::from_secs(20),
            keep_alive: 4,
            open_timeout: Duration::from_secs(5),
        },
        &mut config.tx,
        &mut config.rx,
    )
    .await?;

    let (driver, resources) = resources.init(config, transport, tconfig);

    Ok(PublicSession { driver, resources })
}
