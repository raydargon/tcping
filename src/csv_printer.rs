// CSV output format module

use csv::Writer;
use std::fs::File;
use std::io;

use crate::printer::Printer;
use crate::statistics::Statistics;
use crate::tcp_probe::ProbeResult;
use crate::config::Config;

pub struct CsvPrinter {
    config: Config,
    writer: Option<Writer<File>>,
    file_path: String,
}

impl CsvPrinter {
    pub fn new(config: Config, file_path: Option<String>) -> Result<Self, io::Error> {
        let file_path = file_path.unwrap_or_else(|| "tcping_results.csv".to_string());

        let file = File::create(&file_path)?;
        let mut writer = Writer::from_writer(file);

        // Write CSV header
        writer.write_record(&[
            "timestamp",
            "target",
            "success",
            "rtt_ms",
            "source_addr",
            "target_addr",
            "error",
        ])?;

        writer.flush()?;

        Ok(Self {
            config,
            writer: Some(writer),
            file_path,
        })
    }

    fn write_probe_record(&mut self, result: &ProbeResult) -> Result<(), io::Error> {
        if let Some(ref mut writer) = self.writer {
            writer.write_record(&[
                &chrono::Utc::now().to_rfc3339(),
                &self.config.get_target_address(),
                &result.success.to_string(),
                &if result.success { format!("{:.2}", result.rtt) } else { "".to_string() },
                &result.source_addr.map(|addr| addr.to_string()).unwrap_or_default(),
                &result.target_addr.to_string(),
                &result.error.clone().unwrap_or_default(),
            ])?;

            writer.flush()?;
        }

        Ok(())
    }
}

impl Printer for CsvPrinter {
    fn print_start(&mut self, hostname: &str, port: u16) {
        // CSV output doesn't print start messages to stdout
        // File is already created with header
    }

    fn print_probe_success(&mut self, source_addr: Option<String>, user_input: &str, streak: u32, rtt: f32) {
        let result = ProbeResult {
            success: true,
            rtt,
            source_addr: source_addr.and_then(|addr| addr.parse().ok()),
            target_addr: user_input.parse().unwrap(),
            error: None,
        };

        if let Err(e) = self.write_probe_record(&result) {
            eprintln!("Failed to write CSV record: {}", e);
        }
    }

    fn print_probe_fail(&mut self, user_input: &str, streak: u32) {
        let result = ProbeResult {
            success: false,
            rtt: 0.0,
            source_addr: None,
            target_addr: user_input.parse().unwrap(),
            error: Some("Connection failed".to_string()),
        };

        if let Err(e) = self.write_probe_record(&result) {
            eprintln!("Failed to write CSV record: {}", e);
        }
    }

    fn print_retrying_to_resolve(&mut self, hostname: &str) {
        // CSV output doesn't print retry messages
    }

    fn print_total_downtime(&mut self, downtime: std::time::Duration) {
        // Downtime is handled in statistics
    }

    fn print_statistics(&mut self, stats: &Statistics) {
        // Write final statistics to a separate file or append to CSV
        let stats_file_path = format!("{}_stats.csv", self.file_path.trim_end_matches(".csv"));

        if let Ok(mut stats_writer) = Writer::from_path(&stats_file_path) {
            let _ = stats_writer.write_record(&[
                "statistic", "value"
            ]);

            let total_probes = stats.total_successful_probes + stats.total_unsuccessful_probes;
            let packet_loss = stats.get_packet_loss_percentage();
            let (min_rtt, avg_rtt, max_rtt) = stats.get_rtt_statistics();

            let _ = stats_writer.write_record(&[
                "total_probes", &total_probes.to_string()
            ]);
            let _ = stats_writer.write_record(&[
                "successful_probes", &stats.total_successful_probes.to_string()
            ]);
            let _ = stats_writer.write_record(&[
                "unsuccessful_probes", &stats.total_unsuccessful_probes.to_string()
            ]);
            let _ = stats_writer.write_record(&[
                "packet_loss_percentage", &format!("{:.1}", packet_loss)
            ]);
            let _ = stats_writer.write_record(&[
                "min_rtt_ms", &format!("{:.2}", min_rtt)
            ]);
            let _ = stats_writer.write_record(&[
                "avg_rtt_ms", &format!("{:.2}", avg_rtt)
            ]);
            let _ = stats_writer.write_record(&[
                "max_rtt_ms", &format!("{:.2}", max_rtt)
            ]);
            let _ = stats_writer.write_record(&[
                "total_duration_seconds", &stats.get_total_duration().as_secs_f64().to_string()
            ]);
        }

        println!("CSV results written to: {}", self.file_path);
        println!("Statistics written to: {}", stats_file_path);
    }

    fn print_version(&mut self) {
        println!("tcping version 0.1.0");
    }

    fn print_info(&mut self, format: &str, args: &[&dyn std::fmt::Display]) {
        println!("Info: {} {:?}", format, args);
    }

    fn print_error(&mut self, format: &str, args: &[&dyn std::fmt::Display]) {
        eprintln!("Error: {} {:?}", format, args);
    }
}