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