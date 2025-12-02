use embassy_futures::select::{Either, select};
use embassy_time::Timer;

use crate::{
    io::transport::{TransportRx, ZTransportRx},
    platform::ZPlatform,
};

impl<RxBuf, Platform> super::DriverRx<RxBuf, TransportRx<'_, Platform>>
where
    RxBuf: AsMut<[u8]>,
    Platform: ZPlatform,
{
    pub async fn recv(&mut self) -> crate::ZResult<&[u8]> {
        let read_lease = Timer::at(self.last_read + self.config.other_lease);

        match select(read_lease, self.rx.recv(self.rx_buf.as_mut())).await {
            Either::First(_) => {
                crate::warn!("Connection closed by peer");
                return Err(crate::ZError::ConnectionClosed);
            }
            Either::Second(msg) => match msg {
                Ok(msg) => {
                    self.last_read = embassy_time::Instant::now();
                    Ok(msg)
                }
                Err(e) => {
                    crate::warn!("Could not read from connection: {:?}", e);
                    Err(crate::ZError::CouldNotRead)
                }
            },
        }
    }
}
