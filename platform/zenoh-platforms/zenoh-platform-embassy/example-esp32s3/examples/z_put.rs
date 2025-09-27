#![no_std]
#![no_main]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use esp_hal::rng::Rng;
use getrandom::{register_custom_getrandom, Error};

use core::num::NonZeroU32;
use core::str::FromStr;

use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use zenoh::{keyexpr, EndPoint};

use esp_wifi::{
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
    EspWifiController,
};
use static_cell::StaticCell;
use zenoh::api::session::SessionRunner;
use zenoh_platform_embassy::PlatformEmbassy;

static RNG: Mutex<CriticalSectionRawMutex, Option<Rng>> = Mutex::new(None);

register_custom_getrandom!(getrandom_custom);
const MY_CUSTOM_ERROR_CODE: u32 = Error::CUSTOM_START + 42;
pub fn getrandom_custom(bytes: &mut [u8]) -> Result<(), Error> {
    unsafe {
        RNG.lock_mut(|rng_opt| {
            let code = NonZeroU32::new(MY_CUSTOM_ERROR_CODE).unwrap();
            let rng = rng_opt.as_mut().ok_or(Error::from(code))?;
            rng.read(bytes);
            Ok(())
        })
    }
}

#[panic_handler]
fn panic(panic: &core::panic::PanicInfo) -> ! {
    defmt::error!("Panic: {}", panic);

    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

#[embassy_executor::task]
async fn session_task(mut runner: SessionRunner<'static, PlatformEmbassy>) {
    runner.run().await;
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let net_stack = init_esp32(spawner).await;
    defmt::info!("Network initialized!");

    static PLATFORM: StaticCell<PlatformEmbassy> = StaticCell::new();
    let platform = PLATFORM.init(PlatformEmbassy { stack: net_stack });

    let (mut session, runner) = zenoh::api::session::SingleLinkClientSession::open(
        platform,
        EndPoint::from_str("tcp/192.168.1.23:7447").unwrap(),
    )
    .await
    .unwrap();

    defmt::info!("Session initialized!");

    spawner.spawn(session_task(runner)).unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, world!";

    loop {
        session.try_read().unwrap();

        session.put(ke, payload).await.unwrap();

        defmt::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

async fn init_esp32(spawner: Spawner) -> Stack<'static> {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    defmt::info!("Embassy initialized!");

    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1: TimerGroup<'static, _> = TimerGroup::new(peripherals.TIMG0);

    unsafe {
        RNG.lock_mut(|rng_opt| {
            *rng_opt = Some(rng);
        });
    }

    static WIFI_INIT: StaticCell<EspWifiController<'static>> = StaticCell::new();

    let wifi_init =
        esp_wifi::init(timer1.timer0, rng).expect("Failed to initialize WIFI/BLE controller");

    let (wifi_controller, interfaces) =
        esp_wifi::wifi::new(WIFI_INIT.init(wifi_init), peripherals.WIFI)
            .expect("Failed to initialize WIFI controller");

    let wifi_interface = interfaces.sta;

    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let dhcp_config = DhcpConfig::default();

    let config = embassy_net::Config::dhcpv4(dhcp_config);
    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        RESOURCES.init(StackResources::new()),
        net_seed,
    );

    spawner.spawn(connection(wifi_controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    defmt::info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    defmt::info!("Waiting to get IP address...");
    let _ip = loop {
        if let Some(config) = stack.config_v4() {
            defmt::info!("Got IP: {}", config.address);
            break config.address;
        }
        Timer::after(Duration::from_millis(500)).await;
    };

    stack
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    defmt::info!("start connection task");
    defmt::info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            defmt::info!("Starting wifi");
            controller.start_async().await.unwrap();
            defmt::info!("Wifi started!");
        }
        defmt::info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => defmt::info!("Wifi connected!"),
            Err(e) => {
                defmt::info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
