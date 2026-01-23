use zenoh_embassy::EmbassyLinkManager;
use zenoh_nostd::session::*;

struct Config<'ext> {
    transports: TransportLinkManager<EmbassyLinkManager<'ext, 512, 1>>,
}

impl<'ext> ZSessionConfig for Config<'ext> {
    type Buff = [u8; 512];
    type LinkManager = EmbassyLinkManager<'ext, 512, 1>;

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager> {
        &self.transports
    }

    fn buff(&self) -> Self::Buff {
        [0u8; 512]
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config: Config<'_> = unsafe { core::mem::transmute_copy(&()) };
    let mut resources = SessionResources::new();
    let session = zenoh::connect(&mut resources, &config, Endpoint::try_from("").unwrap()).await;
}
