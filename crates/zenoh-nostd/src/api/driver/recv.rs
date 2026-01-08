use embassy_futures::select::{Either, select};
use embassy_time::Timer;

use crate::{api::ZConfig, io::transport::ZTransportRx};

impl<Config> super::DriverRx<'_, Config>
where
    Config: ZConfig,
{
    pub async fn recv(&mut self) -> crate::ZResult<&[u8]> {
        let read_lease = Timer::at(self.last_read + self.config.other_lease);

        match select(read_lease, self.rx.recv(self.rx_buf.as_mut())).await {
            Either::First(_) => {
                crate::warn!("Connection closed by peer");
                crate::zbail!(crate::TransportError::LeaseTimeout);
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
