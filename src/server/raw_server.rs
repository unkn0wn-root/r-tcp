use crate::{config::ServerConfig, error::Result};
use crate::handler::ConnectionHandler;
use log::{error, info};
use nix::sys::socket::{
    accept, bind, getpeername, listen, socket, setsockopt, sockopt, AddressFamily, SockFlag, SockType, SockaddrIn,
};
use nix::sys::time::TimeVal;
use std::os::unix::io::{FromRawFd, AsRawFd};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::os::fd::{AsFd, BorrowedFd};
use std::net::SocketAddrV4;
use std::io::{Error, ErrorKind};

pub struct RawServer {
    config: ServerConfig,
    active_connections: Arc<AtomicUsize>,
}

impl RawServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            active_connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn run(&self) -> Result<()> {
        let sock_fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None)?;

        // Convert IP address to SocketAddrV4
        let addr = match self.config.host {
            std::net::IpAddr::V4(ipv4) => SocketAddrV4::new(ipv4, self.config.port),
            std::net::IpAddr::V6(_) => {
                return Err(Error::new(ErrorKind::Unsupported, "IPv6 not supported").into())
            }
        };
        let sock_addr = SockaddrIn::from(addr);
        let sock_borrowed = unsafe { BorrowedFd::borrow_raw(sock_fd.as_raw_fd()) };

        bind(sock_fd.as_raw_fd(), &sock_addr)?;
        listen(&sock_fd, self.config.backlog as usize)?;

        info!(
            "Raw syscalls TCP server listening on {}:{}",
            self.config.host, self.config.port
        );

        self.set_socket_options(sock_borrowed)?;
        loop {
            if self.active_connections.load(Ordering::SeqCst) >= self.config.max_connections {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            match accept(sock_fd.as_raw_fd()) {
                Ok(client_fd) => {
                    self.active_connections.fetch_add(1, Ordering::SeqCst);

                    let client_addr = getpeername(client_fd)?;
                    let config = self.config.clone();
                    let active_connections = self.active_connections.clone();

                    std::thread::spawn(move || {
                        if let Err(e) = Self::handle_connection(client_fd, client_addr, config) {
                            error!("Error handling connection: {}", e);
                        }
                        active_connections.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(nix::errno::Errno::EAGAIN) => {
                    // no conn available
                    continue;
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                    continue;
                }
            }
        }
    }

    fn set_socket_options(&self, sock_fd: BorrowedFd) -> Result<()> {
        let read_timeout = TimeVal::new(
            (self.config.read_timeout_ms / 1000) as i64,
            ((self.config.read_timeout_ms % 1000) * 1000) as i32,
        );
        setsockopt(&sock_fd.as_fd(), sockopt::ReceiveTimeout, &read_timeout)?;

        let write_timeout = TimeVal::new(
            (self.config.write_timeout_ms / 1000) as i64,
            ((self.config.write_timeout_ms % 1000) * 1000) as i32,
        );
        setsockopt(&sock_fd.as_fd(), sockopt::SendTimeout, &write_timeout)?;
        setsockopt(&sock_fd.as_fd(), sockopt::ReuseAddr, &true)?;
        setsockopt(&sock_fd.as_fd(), sockopt::TcpNoDelay, &true)?;

        Ok(())
    }

    fn handle_connection(client_fd: i32, _client_addr: nix::sys::socket::SockaddrStorage, config: ServerConfig) -> Result<()> {
        // convert the raw file descriptor to a TcpStream
        let socket = unsafe { std::net::TcpStream::from_raw_fd(client_fd) };

        socket.set_nodelay(true)?;

        let peer_addr = socket.peer_addr()?;
        let mut handler = ConnectionHandler::new(socket, peer_addr, config.buffer_size);

        handler.handle_blocking()?;

        Ok(())
    }
}

impl Drop for RawServer {
    fn drop(&mut self) {
        while self.active_connections.load(Ordering::SeqCst) > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
