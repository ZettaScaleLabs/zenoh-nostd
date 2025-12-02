use embassy_futures::select::{Either, select};
use embassy_time::Timer;
use zenoh_proto::ZError;

use crate::{
    driver::{DriverRx, ZDriverRx},
    io::transport::ZTransportRecv,
};

impl<RxBuf: AsMut<[u8]>, Rx: ZTransportRecv> ZDriverRx for DriverRx<RxBuf, Rx> {
    async fn recv(&mut self) -> crate::ZResult<&[u8]> {
        let read_lease = Timer::at(self.last_read + self.config.other_lease);

        match select(read_lease, self.rx.recv(self.rx_buf.as_mut())).await {
            Either::First(_) => {
                zenoh_proto::warn!("Connection closed by peer");
                return Err(ZError::ConnectionClosed);
            }
            Either::Second(msg) => match msg {
                Ok(msg) => {
                    self.last_read = embassy_time::Instant::now();
                    Ok(msg)
                }
                Err(e) => {
                    zenoh_proto::warn!("Could not read from connection: {:?}", e);
                    Err(ZError::CouldNotRead)
                }
            },
        }
    }
}
