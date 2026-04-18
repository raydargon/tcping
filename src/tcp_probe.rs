// TCP connectivity testing module

use std::net::{TcpStream, SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, Instant};
use std::io;

use crate::config::Config;

#[derive(Debug)]
pub struct TcpProbe {
    config: Config,
}

impl TcpProbe {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn probe_target(&self) -> Result<ProbeResult, ProbeError> {
        let start_time = Instant::now();

        // Resolve target address
        let socket_addr = self.resolve_target()?;

        // Bind to specific interface if requested
        let stream = if let Some(ref interface) = self.config.interface {
            self.connect_with_interface(&socket_addr, interface)?
        } else {
            TcpStream::connect_timeout(&socket_addr, self.config.timeout)?
        };

        // Attempt TCP connection with timeout
        match stream {
            Ok(stream) => {
                let rtt = start_time.elapsed().as_secs_f32() * 1000.0; // Convert to milliseconds

                // Get local address if requested
                let source_addr = if self.config.show_source_address {
                    Some(stream.local_addr().ok())
                } else {
                    None
                };

                Ok(ProbeResult {
                    success: true,
                    rtt,
                    source_addr: source_addr.flatten(),
                    target_addr: socket_addr,
                    error: None,
                })
            },
            Err(e) => {
                Err(ProbeError::ConnectionFailed(format!("Failed to connect to {}: {}", socket_addr, e)))
            }
        }
    }

    fn resolve_target(&self) -> Result<SocketAddr, ProbeError> {
        let target = format!("{}:{}", self.config.host, self.config.port);

        // Try to parse as IP address first
        if let Ok(addr) = target.parse::<SocketAddr>() {
            return Ok(addr);
        }

        // Try to resolve as hostname
        match std::net::ToSocketAddrs::to_socket_addrs(&target) {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    // Filter by IP version preference
                    let filtered_addr = self.filter_by_ip_version(addr);
                    if let Some(addr) = filtered_addr {
                        return Ok(addr);
                    }
                }
                Err(ProbeError::ResolutionFailed(format!("Could not resolve hostname: {}", self.config.host)))
            },
            Err(e) => {
                Err(ProbeError::ResolutionFailed(format!("Failed to resolve target {}: {}", target, e)))
            }
        }
    }

    fn filter_by_ip_version(&self, addr: SocketAddr) -> Option<SocketAddr> {
        match self.config.get_preferred_ip_version() {
            crate::config::IpVersion::V4 if addr.is_ipv4() => Some(addr),
            crate::config::IpVersion::V6 if addr.is_ipv6() => Some(addr),
            crate::config::IpVersion::Any => Some(addr),
            _ => None,
        }
    }

    fn connect_with_interface(&self, target: &SocketAddr, interface: &str) -> Result<TcpStream, ProbeError> {
        // Create a socket bound to the specified interface
        let socket = match target {
            SocketAddr::V4(_) => {
                let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
                socket.set_multicast_if_v4(&interface.parse::<Ipv4Addr>()?)?;
                socket
            },
            SocketAddr::V6(_) => {
                let socket = std::net::UdpSocket::bind("[::]:0")?;
                socket.set_multicast_if_v6(&interface.parse::<Ipv6Addr>()?)?;
                socket
            }
        };

        // Convert to TcpStream with timeout
        let stream = TcpStream::connect_timeout(target, self.config.timeout)?;
        Ok(stream)
    }

    pub fn perform_retry(&self, max_retries: u32) -> Result<ProbeResult, ProbeError> {
        for attempt in 0..=max_retries {
            match self.probe_target() {
                Ok(result) => return Ok(result),
                Err(e) if attempt == max_retries => return Err(e),
                Err(_) => {
                    // Wait before retry (exponential backoff could be added here)
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }

        unreachable!()
    }
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub success: bool,
    pub rtt: f32,
    pub source_addr: Option<SocketAddr>,
    pub target_addr: SocketAddr,
    pub error: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}