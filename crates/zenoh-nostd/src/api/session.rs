use embassy_time::{Duration, Instant};
use zenoh_proto::EndPoint;

use crate::{
    api::driver::{Driver, DriverRx, DriverTx},
    io::{
        link::Link,
        transport::{Transport, TransportConfig, TransportMineConfig, TransportRx, TransportTx},
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
        type Resources<'a> = $crate::api::UserResources<'a, $platform, $txbuf, $rxbuf>;
        type Session<'a> = $crate::api::UserSession<'a, $platform, $txbuf, $rxbuf>;
    };
}

pub type UserSession<'a, Platform, TxBuf, RxBuf> = Session<
    'a,
    Driver<DriverTx<TxBuf, TransportTx<'a, Platform>>, DriverRx<RxBuf, TransportRx<'a, Platform>>>,
>;

pub type UserResources<'a, Platform, TxBuf, RxBuf> = SessionResources<
    Platform,
    Driver<DriverTx<TxBuf, TransportTx<'a, Platform>>, DriverRx<RxBuf, TransportRx<'a, Platform>>>,
>;

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

pub enum SessionResources<Platform, Driver>
where
    Platform: ZPlatform,
{
    Uninitialized,
    Initialized {
        transport: Transport<Platform>,
        driver: Option<Driver>,
    },
}

impl<'a, Platform, TxBuf, RxBuf>
    SessionResources<
        Platform,
        Driver<
            DriverTx<TxBuf, TransportTx<'a, Platform>>,
            DriverRx<RxBuf, TransportRx<'a, Platform>>,
        >,
    >
where
    Platform: ZPlatform,
{
    pub(crate) fn init(
        &'a mut self,
        config: Config<Platform, TxBuf, RxBuf>,
        transport: Transport<Platform>,
        tconfig: TransportConfig,
    ) -> &'a Driver<
        DriverTx<TxBuf, TransportTx<'a, Platform>>,
        DriverRx<RxBuf, TransportRx<'a, Platform>>,
    > {
        *self = SessionResources::Initialized {
            transport,
            driver: None,
        };

        if let SessionResources::Initialized { transport, driver } = self {
            let (tx, rx) = transport.split();
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

            let d = Driver::new(tx, rx);
            *driver = Some(d);
            driver.as_ref().expect("Driver just set")
        } else {
            unreachable!()
        }
    }
}

pub struct Session<'a, Driver> {
    pub(crate) driver: &'a Driver,
}

impl<'a, Driver> Clone for Session<'a, Driver> {
    fn clone(&self) -> Self {
        Session {
            driver: self.driver,
        }
    }
}

pub async fn open<'a, Platform, TxBuf, RxBuf>(
    resources: &'a mut SessionResources<
        Platform,
        Driver<
            DriverTx<TxBuf, TransportTx<'a, Platform>>,
            DriverRx<RxBuf, TransportRx<'a, Platform>>,
        >,
    >,
    mut config: Config<Platform, TxBuf, RxBuf>,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<
    Session<
        'a,
        Driver<
            DriverTx<TxBuf, TransportTx<'a, Platform>>,
            DriverRx<RxBuf, TransportRx<'a, Platform>>,
        >,
    >,
>
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
