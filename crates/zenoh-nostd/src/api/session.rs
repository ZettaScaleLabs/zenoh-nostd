use embassy_time::{Duration, Instant};
use zenoh_proto::EndPoint;

use crate::{
    api::driver::{Driver, DriverRx, DriverTx},
    io::{
        link::Link,
        transport::{Transport, TransportConfig, TransportMineConfig},
    },
    platform::ZPlatform,
};

pub(crate) mod driver;

mod put;
mod run;

#[macro_export]
macro_rules! zimport_types {
    (
        PLATFORM: $platform:ty,
        TX: $txbuf:ty,
        RX: $rxbuf:ty
    ) => {
        type Config = $crate::api::Config<$platform, $txbuf, $rxbuf>;
        type Resources<'a> = $crate::api::Resources<'a, $platform, $txbuf, $rxbuf>;
        type Session<'a> = $crate::api::Session<'a, $platform, $txbuf, $rxbuf>;
    };
}

pub struct Config<Platform, TxBuf, RxBuf> {
    platform: Platform,
    tx: TxBuf,
    rx: RxBuf,
}

impl<Platform, TxBuf, RxBuf> Config<Platform, TxBuf, RxBuf> {
    pub fn new(platform: Platform, tx_buf: TxBuf, rx_buf: RxBuf) -> Self {
        Self {
            platform,
            tx: tx_buf,
            rx: rx_buf,
        }
    }
}

pub struct Resources<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    transport: Option<Transport<Platform>>,
    driver: Option<Driver<'a, Platform, TxBuf, RxBuf>>,
}

impl<'a, Platform, TxBuf, RxBuf> Resources<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    pub fn new() -> Self {
        Self {
            transport: None,
            driver: None,
        }
    }

    pub(crate) fn init(
        &'a mut self,
        config: Config<Platform, TxBuf, RxBuf>,
        transport: Transport<Platform>,
        tconfig: TransportConfig,
    ) -> &'a Driver<'a, Platform, TxBuf, RxBuf> {
        let Self {
            transport: t,
            driver: d,
        } = self;

        *t = Some(transport);
        let (tx, rx) = t.as_mut().expect("Transport just set").split();
        let (tx, rx) = (
            DriverTx {
                tx_buf: config.tx,
                tx,
                sn: tconfig.negociated_config.mine_sn,
                next_keepalive: Instant::now(),
                config: tconfig.mine_config.clone(),
            },
            DriverRx {
                rx_buf: config.rx,
                rx,
                last_read: Instant::now(),
                config: tconfig.other_config.clone(),
            },
        );

        let driver = Driver::new(tx, rx);
        *d = Some(driver);
        d.as_ref().expect("Driver just set")
    }
}

pub struct Session<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) driver: &'a Driver<'a, Platform, TxBuf, RxBuf>,
}

impl<'a, Platform, TxBuf, RxBuf> Clone for Session<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    fn clone(&self) -> Self {
        Session {
            driver: self.driver,
        }
    }
}

pub async fn open<'a, Platform, TxBuf, RxBuf>(
    resources: &'a mut Resources<'a, Platform, TxBuf, RxBuf>,
    mut config: Config<Platform, TxBuf, RxBuf>,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<Session<'a, Platform, TxBuf, RxBuf>>
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

    Ok(Session {
        driver: resources.init(config, transport, tconfig),
    })
}
