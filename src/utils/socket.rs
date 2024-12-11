use crate::error::{Result, ServerError};
use nix::sys::socket::{self, setsockopt, sockopt, SockaddrIn};
use std::net::{IpAddr, SocketAddr};
use std::os::unix::io::RawFd;
use std::time::Duration;
use nix::sys::time::TimeVal;

pub struct SocketUtils;

impl SocketUtils {
    pub fn create_socket(nonblocking: bool) -> Result<RawFd> {
        let mut flags = socket::SockFlag::SOCK_CLOEXEC;
        if nonblocking {
            flags |= socket::SockFlag::SOCK_NONBLOCK;
        }

        socket::socket(
            socket::AddressFamily::Inet,
            socket::SockType::Stream,
            flags,
            None,
        )
        .map_err(ServerError::from)
    }

    pub fn set_socket_opts(fd: RawFd, read_timeout: Duration, write_timeout: Duration) -> Result<()> {
        let read_tv = TimeVal::new(
            read_timeout.as_secs() as i64,
            read_timeout.subsec_micros() as i64,
        );

        let write_tv = TimeVal::new(
            write_timeout.as_secs() as i64,
            write_timeout.subsec_micros() as i64,
        );

        setsockopt(fd, sockopt::ReceiveTimeout, &read_tv)?;
        setsockopt(fd, sockopt::SendTimeout, &write_tv)?;
        setsockopt(fd, sockopt::TcpNoDelay, &true)?;
        setsockopt(fd, sockopt::ReuseAddr, &true)?;

        Ok(())
    }

    pub fn sockaddr_to_std(addr: &SockaddrIn) -> Result<SocketAddr> {
        let ip = match addr.ip() {
            Ok(ip) => IpAddr::V4(ip),
            Err(_) => return Err(ServerError::Connection("Invalid IP address".into())),
        };
        let port = addr.port();
        Ok(SocketAddr::new(ip, port))
    }

    pub fn std_to_sockaddr(addr: SocketAddr) -> Result<SockaddrIn> {
        match addr.ip() {
            IpAddr::V4(ipv4) => Ok(SockaddrIn::new(ipv4.to_string().as_str(), addr.port())),
            IpAddr::V6(_) => Err(ServerError::Connection("IPv6 not supported".into())),
        }
    }
}
