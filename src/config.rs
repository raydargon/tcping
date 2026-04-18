// Configuration module for TCPing

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub ipv4: bool,
    pub ipv6: bool,
    pub retries: u32,
    pub count: u32,
    pub json: bool,
    pub pretty: bool,
    pub no_color: bool,
    pub debug: bool,
    pub csv: bool,
    pub verbose: bool,
    pub update: bool,
    pub interface: Option<String>,
    pub interval: Duration,
    pub timeout: Duration,
    pub database: Option<String>,
    pub show_source_address: bool,
    pub show_failures_only: bool,
    pub show_datetime: bool,
}

impl Config {
    pub fn from_cli(cli: &crate::Cli) -> Result<Self, String> {
        // Parse target (host:port format or separate host/port)
        let (host, port) = Self::parse_target(&cli.target)?;

        // Validate host and port format
        Self::validate_host_port(&host, port)?

        // Validate IPv4/IPv6 exclusivity
        if cli.ipv4 && cli.ipv6 {
            return Err("Cannot use both -4 and -6 flags simultaneously".to_string());
        }

        Ok(Config {
            host: host,
            port: port,
            ipv4: cli.ipv4,
            ipv6: cli.ipv6,
            retries: cli.retries,
            count: cli.count,
            json: cli.json,
            pretty: cli.pretty,
            no_color: cli.no_color,
            debug: cli.debug,
            csv: cli.csv,
            verbose: cli.verbose,
            update: cli.update,
            interface: cli.interface.clone(),
            interval: Duration::from_secs_f64(cli.interval),
            timeout: Duration::from_secs_f64(cli.timeout),
            database: cli.database.clone(),
            show_source_address: cli.show_source_address,
            show_failures_only: cli.show_failures_only,
            show_datetime: cli.show_datetime,
        })
    }

    fn parse_target(target: &str) -> Result<(String, u16), String> {
        // Check if target contains colon (host:port format)
        if let Some(colon_pos) = target.rfind(':') {
            let host_part = &target[..colon_pos];
            let port_part = &target[colon_pos + 1..];

            if host_part.is_empty() {
                return Err("Host cannot be empty".to_string());
            }

            match port_part.parse::<u16>() {
                Ok(port) if port > 0 => Ok((host_part.to_string(), port)),
                Ok(0) => Err("Port cannot be zero".to_string()),
                Ok(_) => Err("Port number must be between 1 and 65535".to_string()),
                Err(_) => Err(format!("Invalid port number: {}", port_part)),
            }
        } else {
            // No colon found, treat as host only (port must be provided separately)
            Err("Target must be in host:port format".to_string())
        }
    }

    fn validate_host_port(host: &str, port: u16) -> Result<(), String> {
        if host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }

        if port == 0 {
            return Err("Port cannot be zero".to_string());
        }

        if port > 65535 {
            return Err("Port number must be between 1 and 65535".to_string());
        }

        Ok(())
    }

    pub fn get_target_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn get_preferred_ip_version(&self) -> IpVersion {
        if self.ipv4 {
            IpVersion::V4
        } else if self.ipv6 {
            IpVersion::V6
        } else {
            IpVersion::Any
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IpVersion {
    V4,
    V6,
    Any,
}