// SQLite database output module

use rusqlite::{Connection, params, Result as SqlResult};
use std::path::Path;

use crate::printer::Printer;
use crate::statistics::Statistics;
use crate::tcp_probe::ProbeResult;
use crate::config::Config;

pub struct DatabasePrinter {
    config: Config,
    connection: Option<Connection>,
    database_path: String,
}

impl DatabasePrinter {
    pub fn new(config: Config, database_path: Option<String>) -> SqlResult<Self> {
        let database_path = database_path.unwrap_or_else(|| "tcping.db".to_string());

        // Create or open database
        let connection = Connection::open(&database_path)?;

        // Create tables if they don't exist
        connection.execute(
            "CREATE TABLE IF NOT EXISTS probes (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                target TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                rtt_ms REAL,
                source_addr TEXT,
                target_addr TEXT NOT NULL,
                error TEXT
            )",
            [],
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS statistics (
                id INTEGER PRIMARY KEY,
                session_start TEXT NOT NULL,
                session_end TEXT,
                target TEXT NOT NULL,
                total_probes INTEGER NOT NULL,
                successful_probes INTEGER NOT NULL,
                unsuccessful_probes INTEGER NOT NULL,
                packet_loss_percentage REAL NOT NULL,
                min_rtt_ms REAL,
                avg_rtt_ms REAL,
                max_rtt_ms REAL,
                total_duration_seconds REAL NOT NULL,
                longest_uptime_seconds REAL,
                longest_downtime_seconds REAL
            )",
            [],
        )?;

        Ok(Self {
            config,
            connection: Some(connection),
            database_path,
        })
    }

    fn insert_probe(&mut self, result: &ProbeResult) -> SqlResult<()> {
        if let Some(ref conn) = self.connection {
            conn.execute(
                "INSERT INTO probes (timestamp, target, success, rtt_ms, source_addr, target_addr, error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    chrono::Utc::now().to_rfc3339(),
                    self.config.get_target_address(),
                    result.success,
                    if result.success { Some(result.rtt) } else { None },
                    result.source_addr.map(|addr| addr.to_string()),
                    result.target_addr.to_string(),
                    result.error.clone()
                ],
            )?;
        }

        Ok(())
    }

    fn insert_statistics(&mut self, stats: &Statistics) -> SqlResult<()> {
        if let Some(ref conn) = self.connection {
            let total_probes = stats.total_successful_probes + stats.total_unsuccessful_probes;
            let packet_loss = stats.get_packet_loss_percentage();
            let (min_rtt, avg_rtt, max_rtt) = stats.get_rtt_statistics();

            conn.execute(
                "INSERT INTO statistics (
                    session_start, session_end, target, total_probes, successful_probes,
                    unsuccessful_probes, packet_loss_percentage, min_rtt_ms, avg_rtt_ms,
                    max_rtt_ms, total_duration_seconds, longest_uptime_seconds, longest_downtime_seconds
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    stats.start_time.to_string(),
                    stats.end_time.map(|t| t.to_string()),
                    self.config.get_target_address(),
                    total_probes,
                    stats.total_successful_probes,
                    stats.total_unsuccessful_probes,
                    packet_loss,
                    min_rtt,
                    avg_rtt,
                    max_rtt,
                    stats.get_total_duration().as_secs_f64(),
                    stats.longest_uptime.as_secs_f64(),
                    stats.longest_downtime.as_secs_f64()
                ],
            )?;
        }

        Ok(())
    }
}

impl Printer for DatabasePrinter {
    fn print_start(&mut self, hostname: &str, port: u16) {
        // Database output doesn't print start messages to stdout
    }

    fn print_probe_success(&mut self, source_addr: Option<String>, user_input: &str, streak: u32, rtt: f32) {
        let result = ProbeResult {
            success: true,
            rtt,
            source_addr: source_addr.and_then(|addr| addr.parse().ok()),
            target_addr: user_input.parse().unwrap(),
            error: None,
        };

        if let Err(e) = self.insert_probe(&result) {
            eprintln!("Failed to insert probe into database: {}", e);
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

        if let Err(e) = self.insert_probe(&result) {
            eprintln!("Failed to insert probe into database: {}", e);
        }
    }

    fn print_retrying_to_resolve(&mut self, hostname: &str) {
        // Database output doesn't print retry messages
    }

    fn print_total_downtime(&mut self, downtime: std::time::Duration) {
        // Downtime is handled in statistics
    }

    fn print_statistics(&mut self, stats: &Statistics) {
        if let Err(e) = self.insert_statistics(stats) {
            eprintln!("Failed to insert statistics into database: {}", e);
        }

        println!("Results saved to database: {}", self.database_path);
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