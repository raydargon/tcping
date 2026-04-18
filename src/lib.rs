// Library module for testing

pub mod config;
pub mod printer;
pub mod statistics;
pub mod tcp_probe;
pub mod json_printer;
pub mod csv_printer;
pub mod database;
pub mod signal_handler;

// Re-export main types for testing
pub use config::Config;
pub use statistics::Statistics;
pub use tcp_probe::{TcpProbe, ProbeResult};

// Define the Cli struct here so it's accessible to all modules
use clap::Parser;

#[derive(Parser, Clone)]
#[command(name = "tcping")]
#[command(about = "TCP connectivity testing tool")]
#[command(version = "0.1.0")]
#[command(author = "")]
#[command(next_line_help = true)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Target hostname or IP address (or host:port format)
    pub target: String,

    /// Use IPv4 only
    #[arg(short = '4')]
    pub ipv4: bool,

    /// Use IPv6 only
    #[arg(short = '6')]
    pub ipv6: bool,

    /// Number of retries after failed probe
    #[arg(short = 'r', default_value_t = 0)]
    pub retries: u32,

    /// Number of probes to send
    #[arg(short = 'c', default_value_t = 0)]
    pub count: u32,

    /// Output in JSON format
    #[arg(short = 'j')]
    pub json: bool,

    /// Pretty print JSON output
    #[arg(long = "pretty")]
    pub pretty: bool,

    /// Disable color output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Debug mode
    #[arg(short = 'd')]
    pub debug: bool,

    /// Output in CSV format
    #[arg(long = "csv")]
    pub csv: bool,

    /// Verbose output
    #[arg(short = 'v')]
    pub verbose: bool,

    /// Check for updates
    #[arg(short = 'u')]
    pub update: bool,

    /// Interface to bind to
    #[arg(short = 'I')]
    pub interface: Option<String>,

    /// Interval between probes in seconds
    #[arg(short = 'i', default_value_t = 1.0)]
    pub interval: f64,

    /// Timeout in seconds
    #[arg(short = 't', default_value_t = 1.0)]
    pub timeout: f64,

    /// SQLite database file
    #[arg(long = "db")]
    pub database: Option<String>,

    /// Show source address
    #[arg(long = "show-source-address")]
    pub show_source_address: bool,

    /// Show failures only
    #[arg(long = "show-failures-only")]
    pub show_failures_only: bool,

    /// Show date/time for each probe
    #[arg(short = 'D')]
    pub show_datetime: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        // Test valid configuration
        let config = Config {
            host: "example.com".to_string(),
            port: 80,
            ipv4: false,
            ipv6: false,
            retries: 0,
            count: 0,
            json: false,
            pretty: false,
            no_color: false,
            debug: false,
            csv: false,
            verbose: false,
            update: false,
            interface: None,
            interval: std::time::Duration::from_secs(1),
            timeout: std::time::Duration::from_secs(5),
            database: None,
            show_source_address: false,
            show_failures_only: false,
        };

        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 80);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut stats = Statistics::new();

        // Record some successes
        stats.record_success(10.5);
        stats.record_success(15.2);
        stats.record_success(12.8);

        // Record a failure
        stats.record_failure();

        let (min, avg, max) = stats.get_rtt_statistics();
        assert_eq!(min, 10.5);
        assert_eq!(max, 15.2);
        assert!((avg - 12.83).abs() < 0.1); // Allow small floating point error

        assert_eq!(stats.total_successful_probes, 3);
        assert_eq!(stats.total_unsuccessful_probes, 1);
        assert_eq!(stats.get_packet_loss_percentage(), 25.0);
    }

    #[test]
    fn test_tcp_probe_structure() {
        let probe_result = ProbeResult {
            success: true,
            rtt: 15.5,
            source_addr: None,
            target_addr: "127.0.0.1:80".parse().unwrap(),
            error: None,
        };

        assert!(probe_result.success);
        assert_eq!(probe_result.rtt, 15.5);
    }
}