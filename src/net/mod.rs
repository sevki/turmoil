//! This module contains the simulated TCP/UDP networking types.
//!
//! They mirror [tokio::net](https://docs.rs/tokio/latest/tokio/net/) to provide
//! a high fidelity implementation.

#[cfg(not(target_arch = "wasm32"))]
pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

#[cfg(target_arch = "wasm32")]
pub mod parser;
#[cfg(target_arch = "wasm32")]
pub use parser::*;

#[cfg(target_arch = "wasm32")]
pub mod ip_addr;
#[cfg(target_arch = "wasm32")]
pub use ip_addr::*;

#[cfg(target_arch = "wasm32")]
pub mod socket_addr;
#[cfg(target_arch = "wasm32")]
pub use socket_addr::*;

pub mod tcp;
pub use tcp::{listener::TcpListener, stream::TcpStream};

mod udp;
pub use udp::UdpSocket;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct SocketPair {
    pub(crate) local: SocketAddr,
    pub(crate) remote: SocketAddr,
}

impl SocketPair {
    pub(crate) fn new(local: SocketAddr, remote: SocketAddr) -> SocketPair {
        assert_ne!(local, remote);
        SocketPair { local, remote }
    }
}

impl std::fmt::Display for SocketPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}→{}", self.local, self.remote)
    }
}
use core::cmp;
use core::fmt::{self, Write};
use core::mem;

/// A partial reimplementation of `impl std::io::Write for &mut [u8]`.
///
/// There are probably simpler ways to do this, this was the first thing that
/// came to mind.
pub(crate) struct WriteHelper<'a>(&'a mut [u8]);

impl<'a> WriteHelper<'a> {
    pub(crate) fn new(inner: &'a mut [u8]) -> Self {
        Self(inner)
    }

    pub(crate) fn into_raw(self) -> &'a mut [u8] {
        self.0
    }
}

impl<'a> Write for WriteHelper<'a> {
    fn write_str(&mut self, data: &str) -> fmt::Result {
        let amt = cmp::min(data.len(), self.0.len());
        let (a, b) = mem::replace(&mut self.0, &mut []).split_at_mut(amt);
        a.copy_from_slice(&data[..amt].as_bytes());
        self.0 = b;
        Ok(())
    }
}
