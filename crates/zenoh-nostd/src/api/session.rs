use embassy_time::{Duration, Instant};
use zenoh_proto::EndPoint;

use crate::{
    io::{
        link::{Link, LinkRx, LinkTx},
        transport::{
            Transport, TransportConfig, TransportMineConfig, TransportRx, TransportTx, ZTransport,
        },
    },
    platform::ZPlatform,
};

use driver::ZDriver;

pub(crate) mod driver;
pub use driver::{Driver, DriverRx, DriverTx};

mod put;
mod run;

#[macro_export]
macro_rules! zimport_types {
    (
        PLATFORM: $platform:ty,
        TX_BUF: $tx_buf:ty,
        RX_BUF: $rx_buf:ty
    ) => {
        type Config = $crate::Config<$platform, $tx_buf, $rx_buf>;
        type Resources<'a> = $crate::SessionResources<'a, $platform, $tx_buf, $rx_buf>;
        type Session<'a> = $crate::Session<
            'a,
            $crate::Driver<
                $crate::DriverTx<$tx_buf, $crate::TransportTx<$crate::LinkTx<'a, $platform>>>,
                $crate::DriverRx<$rx_buf, $crate::TransportRx<$crate::LinkRx<'a, $platform>>>,
            >,
        >;
    };
}

pub struct Config<P: ZPlatform, TxBuf: AsMut<[u8]>, RxBuf: AsMut<[u8]>> {
    pub platform: P,
    pub tx_buf: TxBuf,
    pub rx_buf: RxBuf,
}

pub enum SessionResources<'a, P: ZPlatform, TxBuf: AsMut<[u8]>, RxBuf: AsMut<[u8]>> {
    Uninitialized,
    Initialized {
        transport: Transport<Link<P>>,
        driver: Option<
            Driver<
                DriverTx<TxBuf, TransportTx<LinkTx<'a, P>>>,
                DriverRx<RxBuf, TransportRx<LinkRx<'a, P>>>,
            >,
        >,
    },
}

impl<'a, P: ZPlatform, TxBuf: AsMut<[u8]>, RxBuf: AsMut<[u8]>>
    SessionResources<'a, P, TxBuf, RxBuf>
{
    pub(crate) fn init(
        &'a mut self,
        config: Config<P, TxBuf, RxBuf>,
        transport: Transport<Link<P>>,
        tconfig: TransportConfig,
    ) -> &'a Driver<
        DriverTx<TxBuf, TransportTx<LinkTx<'a, P>>>,
        DriverRx<RxBuf, TransportRx<LinkRx<'a, P>>>,
    > {
        *self = SessionResources::Initialized {
            transport,
            driver: None,
        };

        if let SessionResources::Initialized { transport, driver } = self {
            let (tx, rx) = transport.split();
            let (tx, rx) = (
                DriverTx {
                    tx_buf: config.tx_buf,
                    tx,
                    sn: tconfig.negociated_config.mine_sn,
                    next_keepalive: Instant::now(),
                    config: tconfig.mine_config.clone(),
                },
                DriverRx {
                    rx_buf: config.rx_buf,
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

pub struct Session<'a, T: ZDriver> {
    pub(crate) driver: &'a T,
}

impl<'a, T: ZDriver> Clone for Session<'a, T> {
    fn clone(&self) -> Self {
        Session {
            driver: self.driver,
        }
    }
}

pub async fn open<'a, P: ZPlatform, TxBuf: AsMut<[u8]>, RxBuf: AsMut<[u8]>>(
    resources: &'a mut SessionResources<'a, P, TxBuf, RxBuf>,
    mut config: Config<P, TxBuf, RxBuf>,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<
    Session<
        'a,
        Driver<
            DriverTx<TxBuf, TransportTx<LinkTx<'a, P>>>,
            DriverRx<RxBuf, TransportRx<LinkRx<'a, P>>>,
        >,
    >,
> {
    let link = Link::new(&config.platform, endpoint).await?;

    let (transport, tconfig) = Transport::open(
        link,
        TransportMineConfig {
            mine_zid: Default::default(),
            mine_lease: Duration::from_secs(20),
            keep_alive: 4,
            open_timeout: Duration::from_secs(5),
        },
        &mut config.tx_buf,
        &mut config.rx_buf,
    )
    .await?;

    let driver = resources.init(config, transport, tconfig);

    Ok(Session { driver })
}
