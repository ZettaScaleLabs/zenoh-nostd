#![no_std]

#[cfg(feature = "std")]
pub use zenoh_std::PlatformStd as Platform;

#[cfg(feature = "esp32s3")]
pub use zenoh_embassy::PlatformEmbassy as Platform;

#[cfg(feature = "wasm")]
pub use zenoh_wasm::PlatformWasm as Platform;

#[cfg(feature = "esp32s3")]
mod esp32s3_app {
    pub use embassy_net::{DhcpConfig, Runner, StackResources};
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

pub async fn init_platform(spawner: &embassy_executor::Spawner) -> Platform {
    #[cfg(feature = "std")]
    {
        let _ = spawner;
        Platform {}
    }
    #[cfg(feature = "wasm")]
    {
        let _ = spawner;
        Platform {}
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

        Platform { stack }
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
