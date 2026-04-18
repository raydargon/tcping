// JSON output format module

use serde::{Serialize, Deserialize};
use serde_json;

use crate::printer::Printer;
use crate::statistics::Statistics;
use crate::tcp_probe::ProbeResult;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput {
    pub target: String,
    pub probes: Vec<JsonProbe>,
    pub statistics: JsonStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonProbe {
    pub timestamp: String,
    pub success: bool,
    pub rtt: Option<f32>,
    pub source_addr: Option<String>,
    pub target_addr: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStatistics {
    pub total_probes: u32,
    pub successful_probes: u32,
    pub unsuccessful_probes: u32,
    pub packet_loss_percentage: f32,
    pub min_rtt: f32,
    pub avg_rtt: f32,
    pub max_rtt: f32,
    pub total_duration_seconds: f64,
    pub longest_uptime_seconds: f64,
    pub longest_downtime_seconds: f64,
}

pub struct JsonPrinter {
    config: Config,
    probes: Vec<JsonProbe>,
    pretty: bool,
}

impl JsonPrinter {
    pub fn new(config: Config, pretty: bool) -> Self {
        Self {
            config,
            probes: Vec::new(),
            pretty,
        }
    }

    pub fn add_probe(&mut self, result: &ProbeResult) {
        let probe = JsonProbe {
            timestamp: chrono::Utc::now().to_rfc3339(),
            success: result.success,
            rtt: if result.success { Some(result.rtt) } else { None },
            source_addr: result.source_addr.map(|addr| addr.to_string()),
            target_addr: result.target_addr.to_string(),
            error: result.error.clone(),
        };

        self.probes.push(probe);
    }

    pub fn generate_output(&self, stats: &Statistics) -> String {
        let json_stats = JsonStatistics {
            total_probes: stats.total_successful_probes + stats.total_unsuccessful_probes,
            successful_probes: stats.total_successful_probes,
            unsuccessful_probes: stats.total_unsuccessful_probes,
            packet_loss_percentage: stats.get_packet_loss_percentage(),
            min_rtt: stats.get_rtt_statistics().0,
            avg_rtt: stats.get_rtt_statistics().1,
            max_rtt: stats.get_rtt_statistics().2,
            total_duration_seconds: stats.get_total_duration().as_secs_f64(),
            longest_uptime_seconds: stats.longest_uptime.as_secs_f64(),
            longest_downtime_seconds: stats.longest_downtime.as_secs_f64(),
        };

        let output = JsonOutput {
            target: self.config.get_target_address(),
            probes: self.probes.clone(),
            statistics: json_stats,
        };

        if self.pretty {
            serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
        } else {
            serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

impl Printer for JsonPrinter {
    fn print_start(&mut self, hostname: &str, port: u16) {
        // JSON output doesn't print start messages to stdout
    }

    fn print_probe_success(&mut self, source_addr: Option<String>, _user_input: &str, _streak: u32, _rtt: f32) {
        // Probes are collected and output at the end in JSON format
    }

    fn print_probe_fail(&mut self, _user_input: &str, _streak: u32) {
        // Probes are collected and output at the end in JSON format
    }

    fn print_retrying_to_resolve(&mut self, _hostname: &str) {
        // JSON output doesn't print retry messages
    }

    fn print_total_downtime(&mut self, _downtime: std::time::Duration) {
        // Downtime is included in final statistics
    }

    fn print_statistics(&mut self, stats: &Statistics) {
        let output = self.generate_output(stats);
        println!("{}", output);
    }

    fn print_version(&mut self) {
        let version_info = serde_json::json!({
            "version": "0.1.0",
            "name": "tcping"
        });

        if self.pretty {
            println!("{}", serde_json::to_string_pretty(&version_info).unwrap());
        } else {
            println!("{}", serde_json::to_string(&version_info).unwrap());
        }
    }

    fn print_info(&mut self, format: &str, args: &[&dyn std::fmt::Display]) {
        let message = if args.is_empty() {
            format.to_string()
        } else {
            format!("{} {:?}", format, args)
        };

        let info_output = serde_json::json!({
            "type": "info",
            "message": message
        });

        if self.pretty {
            println!("{}", serde_json::to_string_pretty(&info_output).unwrap());
        } else {
            println!("{}", serde_json::to_string(&info_output).unwrap());
        }
    }

    fn print_error(&mut self, format: &str, args: &[&dyn std::fmt::Display]) {
        let message = if args.is_empty() {
            format.to_string()
        } else {
            format!("{} {:?}", format, args)
        };

        let error_output = serde_json::json!({
            "type": "error",
            "message": message
        });

        eprintln!("{}", serde_json::to_string(&error_output).unwrap());
    }
}