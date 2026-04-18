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
}

impl Config {
    pub fn from_cli(cli: &crate::Cli) -> Result<Self, String> {
        // Validate host and port format
        Self::validate_host_port(&cli.host, cli.port)?;

        // Validate IPv4/IPv6 exclusivity
        if cli.ipv4 && cli.ipv6 {
            return Err("Cannot use both -4 and -6 flags simultaneously".to_string());
        }

        Ok(Config {
            host: cli.host.clone(),
            port: cli.port,
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
        })
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