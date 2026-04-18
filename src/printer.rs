// Printer module for output formatting

use colored::*;
use std::fmt;

use crate::config::Config;
use crate::statistics::Statistics;
use crate::tcp_probe::ProbeResult;

pub trait Printer {
    fn print_start(&mut self, hostname: &str, port: u16);
    fn print_probe_success(&mut self, source_addr: Option<String>, user_input: &str, streak: u32, rtt: f32);
    fn print_probe_fail(&mut self, user_input: &str, streak: u32);
    fn print_retrying_to_resolve(&mut self, hostname: &str);
    fn print_total_downtime(&mut self, downtime: std::time::Duration);
    fn print_statistics(&mut self, stats: &Statistics);
    fn print_version(&mut self);
    fn print_info(&mut self, format: &str, args: &[&dyn fmt::Display]);
    fn print_error(&mut self, format: &str, args: &[&dyn fmt::Display]);
}

pub struct ConsolePrinter {
    config: Config,
    use_color: bool,
}

impl ConsolePrinter {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            use_color: !config.no_color,
        }
    }

    fn format_duration(duration: std::time::Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m{}s", secs / 60, secs % 60)
        } else {
            format!("{}h{}m{}s", secs / 3600, (secs % 3600) / 60, secs % 60)
        }
    }
}

impl Printer for ConsolePrinter {
    fn print_start(&mut self, hostname: &str, port: u16) {
        let message = format!("TCPING {}:{}", hostname, port);
        if self.use_color {
            println!("{}", message.bold().blue());
        } else {
            println!("{}", message);
        }
    }

    fn print_probe_success(&mut self, source_addr: Option<String>, user_input: &str, streak: u32, rtt: f32) {
        let source_info = if let Some(addr) = source_addr {
            format!(" from {}", addr)
        } else {
            String::new()
        };

        let streak_info = if streak > 1 {
            format!(" (streak: {})", streak)
        } else {
            String::new()
        };

        let message = format!("{}: connected{} in {:.2}ms{}", user_input, source_info, rtt, streak_info);

        if self.use_color {
            println!("{}", message.green());
        } else {
            println!("{}", message);
        }
    }

    fn print_probe_fail(&mut self, user_input: &str, streak: u32) {
        let streak_info = if streak > 0 {
            format!(" (consecutive failures: {})", streak)
        } else {
            String::new()
        };

        let message = format!("{}: connection failed{}", user_input, streak_info);

        if self.use_color {
            println!("{}", message.red());
        } else {
            println!("{}", message);
        }
    }

    fn print_retrying_to_resolve(&mut self, hostname: &str) {
        let message = format!("Retrying to resolve: {}", hostname);
        if self.use_color {
            println!("{}", message.yellow());
        } else {
            println!("{}", message);
        }
    }

    fn print_total_downtime(&mut self, downtime: std::time::Duration) {
        let message = format!("Total downtime: {}", Self::format_duration(downtime));
        if self.use_color {
            println!("{}", message.yellow());
        } else {
            println!("{}", message);
        }
    }

    fn print_statistics(&mut self, stats: &Statistics) {
        let total_probes = stats.total_successful_probes + stats.total_unsuccessful_probes;
        let packet_loss = stats.get_packet_loss_percentage();
        let (min_rtt, avg_rtt, max_rtt) = stats.get_rtt_statistics();
        let total_duration = stats.get_total_duration();

        println!("\n--- {} statistics ---", self.config.get_target_address());
        println!("{} probes transmitted, {} received, {:.1}% packet loss",
                total_probes, stats.total_successful_probes, packet_loss);
        println!("time {}", Self::format_duration(total_duration));

        if stats.total_successful_probes > 0 {
            println!("rtt min/avg/max = {:.2}/{:.2}/{:.2} ms", min_rtt, avg_rtt, max_rtt);
        }

        if stats.longest_uptime > std::time::Duration::ZERO {
            println!("Longest uptime: {}", Self::format_duration(stats.longest_uptime));
        }

        if stats.longest_downtime > std::time::Duration::ZERO {
            println!("Longest downtime: {}", Self::format_duration(stats.longest_downtime));
        }
    }

    fn print_version(&mut self) {
        println!("tcping version 0.1.0");
    }

    fn print_info(&mut self, format: &str, args: &[&dyn fmt::Display]) {
        let message = if args.is_empty() {
            format.to_string()
        } else {
            // Simple formatting for demonstration
            // In a real implementation, this would use proper formatting
            format!("{} {:?}", format, args)
        };

        if self.use_color {
            println!("{}", message.cyan());
        } else {
            println!("{}", message);
        }
    }

    fn print_error(&mut self, format: &str, args: &[&dyn fmt::Display]) {
        let message = if args.is_empty() {
            format.to_string()
        } else {
            // Simple formatting for demonstration
            format!("{} {:?}", format, args)
        };

        eprintln!("Error: {}", message);
    }
}