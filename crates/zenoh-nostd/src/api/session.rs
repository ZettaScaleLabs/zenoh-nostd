use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use embassy_time::{Duration, Instant};
use heapless::FnvIndexMap;
use zenoh_proto::{EndPoint, keyexpr};

use crate::{
    api::{
        Callback, ZOwnedSample, ZSample,
        driver::{Driver, DriverRx, DriverTx},
    },
    io::{
        link::Link,
        transport::{Transport, TransportConfig, TransportMineConfig},
    },
    platform::ZPlatform,
};

pub(crate) mod driver;
mod resources;
pub use resources::*;

mod put;
mod run;

#[macro_export]
macro_rules! zimport_types {
    (
        PLATFORM: $platform:ty,
        TX: $txbuf:ty,
        RX: $rxbuf:ty,

        MAX_KEYEXPR: $max_keyexpr:expr,
        MAX_QUEUED: $max_queued:expr,

        MAX_KEYEXPR_LEN: $max_keyexpr_len:expr,
        MAX_PARAMETERS_LEN: $max_parameters_len:expr,
        MAX_PAYLOAD_LEN: $max_payload_len:expr,

        MAX_SUBSCRIPTIONS: $max_subscriptions:expr,

        MAX_QUERIES: $max_queries:expr,
        MAX_QUERYABLES: $max_queryables:expr,
    ) => {
        type Config = $crate::api::PublicConfig<$platform, $txbuf, $rxbuf>;
        type Resources<'a> = $crate::api::PublicResources<
            'a,
            $platform,
            $txbuf,
            $rxbuf,
            $crate::api::SessionResources<
                $max_keyexpr,
                $max_queued,
                $max_keyexpr_len,
                $max_parameters_len,
                $max_payload_len,
                $max_subscriptions,
                $max_queries,
                $max_queryables,
            >,
        >;
        type Session<'a> = $crate::api::PublicSession<'a, $platform, $txbuf, $rxbuf>;
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

pub struct PublicSession<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) driver: &'a Driver<'a, Platform, TxBuf, RxBuf>,
}

impl<'a, Platform, TxBuf, RxBuf> Clone for PublicSession<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    fn clone(&self) -> Self {
        PublicSession {
            driver: self.driver,
        }
    }
}

pub async fn open<'a, Platform, TxBuf, RxBuf, SessionResources>(
    resources: &'a mut PublicResources<'a, Platform, TxBuf, RxBuf, SessionResources>,
    mut config: PublicConfig<Platform, TxBuf, RxBuf>,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<PublicSession<'a, Platform, TxBuf, RxBuf>>
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

    Ok(PublicSession {
        driver: resources.init(config, transport, tconfig),
    })
}
