use embassy_futures::select::{Either, select};
use embassy_time::Timer;
use zenoh_proto::msgs::NetworkMessage;

use crate::{api::ZConfig, io::ZTransportLinkRx};

impl<'res, Config> super::DriverRx<'res, Config>
where
    Config: ZConfig,
{
    pub async fn recv(
        &mut self,
    ) -> crate::ZResult<impl Iterator<Item = (NetworkMessage<'_>, &'_ [u8])>> {
        let read_lease = Timer::at(self.last_read + self.rx.transport.lease.try_into().unwrap());

        match select(read_lease, self.rx.recv()).await {
            Either::First(_) => {
                crate::warn!("Connection closed by peer");
                crate::zbail!(crate::TransportLinkError::LeaseTimeout);
            }
            Either::Second(msg) => match msg {
                Ok(msg) => {
                    self.last_read = embassy_time::Instant::now();
                    Ok(msg)
                }
                Err(e) => crate::zbail!(e),
            },
        }
    }
}
