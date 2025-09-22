//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use std::{cell::UnsafeCell, fmt, net::SocketAddr, sync::Arc};

use async_net::TcpStream;
use async_trait::async_trait;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use zenoh_link_commons::{LinkAuthId, LinkUnicast, LinkUnicastTrait};
use zenoh_protocol::{
    core::{EndPoint, Locator},
    transport::BatchSize,
};
use zenoh_result::{zerror, ZResult};

use crate::{TcpLinkConfig, TcpSocketConfig, IS_RELIABLE, TCP_DEFAULT_MTU, TCP_LOCATOR_PREFIX};

pub struct LinkUnicastTcp {
    // The underlying socket as returned from the tokio library
    socket: UnsafeCell<TcpStream>,
    // The source socket address of this link (address used on the local host)
    src_addr: SocketAddr,
    src_locator: Locator,
    // The destination socket address of this link (address used on the remote host)
    dst_addr: SocketAddr,
    dst_locator: Locator,
    // The computed mtu
    mtu: BatchSize,
}

unsafe impl Sync for LinkUnicastTcp {}

impl LinkUnicastTcp {
    pub fn new(
        socket: TcpStream,
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
    ) -> ZResult<LinkUnicastTcp> {
        // Set the TCP nodelay option
        socket.set_nodelay(true).map_err(|e| {
            zerror!(
                "Cannot set TCP_NODELAY option on TCP link {} => {}: {:?}",
                src_addr,
                dst_addr,
                e
            )
        })?;

        // Compute the MTU
        // See IETF RFC6691: https://datatracker.ietf.org/doc/rfc6691/
        let header = match src_addr.ip() {
            std::net::IpAddr::V4(_) => 40,
            std::net::IpAddr::V6(_) => 60,
        };
        #[allow(unused_mut)] // mut is not needed when target_family != unix
        let mut mtu = TCP_DEFAULT_MTU - header;

        // Build the Tcp object
        Ok(LinkUnicastTcp {
            socket: UnsafeCell::new(socket),
            src_addr,
            src_locator: Locator::new(TCP_LOCATOR_PREFIX, src_addr.to_string(), "").unwrap(),
            dst_addr,
            dst_locator: Locator::new(TCP_LOCATOR_PREFIX, dst_addr.to_string(), "").unwrap(),
            mtu,
        })
    }

    pub async fn new_link(endpoint: &EndPoint) -> ZResult<LinkUnicast> {
        let config = endpoint.config();

        let socket_config = TcpSocketConfig::from(TcpLinkConfig::new(&config).await?);

        let da = endpoint.address().to_string();
        let dst_addr: SocketAddr = da.parse().map_err(|e| {
            zerror!(
                "Cannot parse TCP address '{}' for endpoint {}: {}",
                da,
                endpoint,
                e
            )
        })?;
        match socket_config.new_link(&dst_addr).await {
            Ok((stream, src_addr, dst_addr)) => {
                let link = Arc::new(LinkUnicastTcp::new(stream, src_addr, dst_addr)?);

                Ok(LinkUnicast(link))
            }
            Err(_) => Err(zerror!(
                "Cannot connect to TCP address '{}' for endpoint {}",
                da,
                endpoint
            )
            .into()),
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn get_mut_socket(&self) -> &mut TcpStream {
        unsafe { &mut *self.socket.get() }
    }
}

#[async_trait]
impl LinkUnicastTrait for LinkUnicastTcp {
    async fn close(&self) -> ZResult<()> {
        self.get_mut_socket()
            .shutdown(std::net::Shutdown::Both)
            .map_err(|e| zerror!("TCP link shutdown {}: {:?}", self, e).into())
    }

    async fn write(&self, buffer: &[u8]) -> ZResult<usize> {
        self.get_mut_socket()
            .write(buffer)
            .await
            .map_err(|e| zerror!("Write error on TCP link {}: {}", self, e).into())
    }

    async fn write_all(&self, buffer: &[u8]) -> ZResult<()> {
        self.get_mut_socket()
            .write_all(buffer)
            .await
            .map_err(|e| zerror!("Write error on TCP link {}: {}", self, e).into())
    }

    async fn read(&self, buffer: &mut [u8]) -> ZResult<usize> {
        self.get_mut_socket()
            .read(buffer)
            .await
            .map_err(|e| zerror!("Read error on TCP link {}: {}", self, e).into())
    }

    async fn read_exact(&self, buffer: &mut [u8]) -> ZResult<()> {
        let _ = self
            .get_mut_socket()
            .read_exact(buffer)
            .await
            .map_err(|e| zerror!("Read error on TCP link {}: {}", self, e))?;
        Ok(())
    }

    #[inline(always)]
    fn get_src(&self) -> &Locator {
        &self.src_locator
    }

    #[inline(always)]
    fn get_dst(&self) -> &Locator {
        &self.dst_locator
    }

    #[inline(always)]
    fn get_mtu(&self) -> BatchSize {
        self.mtu
    }

    #[inline(always)]
    fn is_reliable(&self) -> bool {
        IS_RELIABLE
    }

    #[inline(always)]
    fn is_streamed(&self) -> bool {
        true
    }

    #[inline(always)]
    fn get_auth_id(&self) -> &LinkAuthId {
        &LinkAuthId::Tcp
    }
}

impl fmt::Display for LinkUnicastTcp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.src_addr, self.dst_addr)?;
        Ok(())
    }
}

impl fmt::Debug for LinkUnicastTcp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tcp")
            .field("src", &self.src_addr)
            .field("dst", &self.dst_addr)
            .field("mtu", &self.get_mtu())
            .finish()
    }
}
