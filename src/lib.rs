#![no_std]

use zenoh_nostd::api::ZConfig;

#[cfg(feature = "std")]
pub use zenoh_std::PlatformStd as Platform;

#[cfg(feature = "esp32s3")]
pub use zenoh_embassy::PlatformEmbassy as Platform;

#[cfg(feature = "wasm")]
pub use zenoh_wasm::PlatformWasm as Platform;

#[cfg(feature = "esp32s3")]
mod esp32s3_app {
    pub use embassy_net::{DhcpConfig, Runner, StackResources, udp::PacketMetadata};
    pub use embassy_time::{Duration, Timer};
    pub use esp_hal::{clock::CpuClock, rng::Rng, timer::systimer::SystemTimer};
    pub use esp_println as _;
    pub use esp_radio::{
        Controller,
        wifi::{ClientConfig, ModeConfig, WifiController, WifiDevice, WifiEvent, WifiStaState},
    };

    pub const SSID: Option<&str> = option_env!("WIFI_SSID");
    pub const PASSWORD: &str = env!("WIFI_PASSWORD");
}

#[cfg(feature = "esp32s3")]
use esp32s3_app::*;

pub const CONNECT: &str = match option_env!("CONNECT") {
    Some(v) => v,
    None => {
        if cfg!(feature = "wasm") {
            "ws/127.0.0.1:7446"
        } else {
            "tcp/127.0.0.1:7447"
        }
    }
};

#[cfg(feature = "esp32s3")]
const BUFF_SIZE: u16 = 512u16;
#[cfg(not(feature = "esp32s3"))]
const BUFF_SIZE: u16 = u16::MAX;

pub struct ExampleConfig {
    platform: Platform,
    tx: [u8; BUFF_SIZE as usize],
    rx: [u8; BUFF_SIZE as usize],
}

impl ZConfig for ExampleConfig {
    type Platform = Platform;

    type TxBuf = [u8; BUFF_SIZE as usize];
    type RxBuf = [u8; BUFF_SIZE as usize];

    fn platform(&self) -> &Self::Platform {
        &self.platform
    }

    fn txrx(&mut self) -> (&mut Self::TxBuf, &mut Self::RxBuf) {
        (&mut self.tx, &mut self.rx)
    }

    fn into_parts(self) -> (Self::Platform, Self::TxBuf, Self::RxBuf) {
        (self.platform, self.tx, self.rx)
    }
}

pub async fn init_example(spawner: &embassy_executor::Spawner) -> ExampleConfig {
    #[cfg(feature = "std")]
    {
        let _ = spawner;
        ExampleConfig {
            platform: Platform {},
            tx: [0; BUFF_SIZE as usize],
            rx: [0; BUFF_SIZE as usize],
        }
    }
    #[cfg(feature = "wasm")]
    {
        let _ = spawner;
        ExampleConfig {
            platform: Platform {},
            tx: [0; BUFF_SIZE as usize],
            rx: [0; BUFF_SIZE as usize],
        }
    }
    #[cfg(feature = "esp32s3")]
    {
        use static_cell::StaticCell;

        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        esp_alloc::heap_allocator!(size: 64 * 1024);

        let timer0 = SystemTimer::new(peripherals.SYSTIMER);
        esp_rtos::start(timer0.alarm0);

        zenoh_nostd::info!("Embassy initialized!");

        let rng = Rng::new();

        static RADIO_CTRL: StaticCell<Controller<'static>> = StaticCell::new();
        let radio_ctrl = esp_radio::init().expect("Failed to init radio");

        let (wifi_controller, interfaces) = esp_radio::wifi::new(
            RADIO_CTRL.init(radio_ctrl),
            peripherals.WIFI,
            Default::default(),
        )
        .expect("Failed to initialize WIFI controller");

        let wifi_interface = interfaces.sta;
        let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
        let dhcp_config = DhcpConfig::default();
        let config = embassy_net::Config::dhcpv4(dhcp_config);

        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            wifi_interface,
            config,
            RESOURCES.init(StackResources::new()),
            net_seed,
        );

        spawner.spawn(connection(wifi_controller)).ok();
        spawner.spawn(net_task(runner)).ok();

        zenoh_nostd::info!("Waiting for link to be up");
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        zenoh_nostd::info!("Waiting to get IP address...");
        let ip = loop {
            if let Some(config) = stack.config_v4() {
                zenoh_nostd::info!("Got IP: {}", config.address);
                break config.address;
            }
            Timer::after(Duration::from_millis(500)).await;
        };
        zenoh_nostd::info!("Network initialized with IP: {}", ip);

        /// This is a naive way, internally it always gives a ref to the same buffer
        /// so it's UB to create multiple links
        fn buffers() -> (&'static mut [u8], &'static mut [u8]) {
            static TX: StaticCell<[u8; BUFF_SIZE as usize]> = StaticCell::new();
            let tx = TX.init([0; BUFF_SIZE as usize]);

            static RX: StaticCell<[u8; BUFF_SIZE as usize]> = StaticCell::new();
            let rx = RX.init([0; BUFF_SIZE as usize]);

            (tx, rx)
        }

        fn metadatas() -> (&'static mut [PacketMetadata], &'static mut [PacketMetadata]) {
            static TX: StaticCell<[PacketMetadata; 16]> = StaticCell::new();
            let tx = TX.init([PacketMetadata::EMPTY; 16]);

            static RX: StaticCell<[PacketMetadata; 16]> = StaticCell::new();
            let rx = RX.init([PacketMetadata::EMPTY; 16]);

            (tx, rx)
        }

        ExampleConfig {
            platform: Platform {
                stack,
                buffers,
                metadatas,
            },
            tx: [0; BUFF_SIZE as usize],
            rx: [0; BUFF_SIZE as usize],
        }
    }
}

#[cfg(feature = "esp32s3")]
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    zenoh_nostd::info!("start connection task");
    zenoh_nostd::info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(SSID.unwrap_or("ZettaScale").into())
                    .with_password(PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            zenoh_nostd::info!("Starting wifi");
            controller.start_async().await.unwrap();
            zenoh_nostd::info!("Wifi started!");
        }
        zenoh_nostd::info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => zenoh_nostd::info!("Wifi connected!"),
            Err(e) => {
                zenoh_nostd::info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[cfg(feature = "esp32s3")]
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
