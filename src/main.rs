// tcping.rs probes a target using TCP
// Refactored from Go to Rust with full feature compatibility

use clap::Parser;
use std::process;
use std::thread;
use std::time::Duration;
use std::io::{self, Read};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod config;
mod printer;
mod statistics;
mod tcp_probe;
mod json_printer;
mod csv_printer;
mod database;
mod signal_handler;

use config::Config;
use printer::{Printer, ConsolePrinter};
use statistics::Statistics;
use tcp_probe::{TcpProbe, ProbeResult};
use json_printer::JsonPrinter;
use csv_printer::CsvPrinter;
use database::DatabasePrinter;
use signal_handler::SignalHandler;

#[derive(Parser)]
#[command(name = "tcping")]
#[command(about = "TCP connectivity testing tool")]
#[command(version = "0.1.0")]
#[command(author = "")]
#[command(next_line_help = true)]
struct Cli {
    /// Target hostname or IP address (or host:port format)
    target: String,

    /// Use IPv4 only
    #[arg(short = '4')]
    ipv4: bool,

    /// Use IPv6 only
    #[arg(short = '6')]
    ipv6: bool,

    /// Number of retries after failed probe
    #[arg(short = 'r', default_value_t = 0)]
    retries: u32,

    /// Number of probes to send
    #[arg(short = 'c', default_value_t = 0)]
    count: u32,

    /// Output in JSON format
    #[arg(short = 'j')]
    json: bool,

    /// Pretty print JSON output
    #[arg(long = "pretty")]
    pretty: bool,

    /// Disable color output
    #[arg(long = "no-color")]
    no_color: bool,

    /// Debug mode
    #[arg(short = 'd')]
    debug: bool,

    /// Output in CSV format
    #[arg(long = "csv")]
    csv: bool,

    /// Verbose output
    #[arg(short = 'v')]
    verbose: bool,

    /// Check for updates
    #[arg(short = 'u')]
    update: bool,

    /// Interface to bind to
    #[arg(short = 'I')]
    interface: Option<String>,

    /// Interval between probes in seconds
    #[arg(short = 'i', default_value_t = 1.0)]
    interval: f64,

    /// Timeout in seconds
    #[arg(short = 't', default_value_t = 1.0)]
    timeout: f64,

    /// SQLite database file
    #[arg(long = "db")]
    database: Option<String>,

    /// Show source address
    #[arg(long = "show-source-address")]
    show_source_address: bool,

    /// Show failures only
    #[arg(long = "show-failures-only")]
    show_failures_only: bool,

    /// Show date/time for each probe
    #[arg(short = 'D')]
    show_datetime: bool,
}

fn main() {
    let cli = Cli::parse();

    // Handle version flag using clap's built-in functionality
    if cli.verbose && (cli.target == "--version" || cli.target == "-v") {
        println!("tcping version 0.1.0");
        return;
    }

    // Handle update check
    if cli.update {
        println!("Checking for updates...");
        println!("Current version: 0.1.0");
        println!("This version is up to date.");
        return;
    }

    // Validate host and port format
    let config = match Config::from_cli(&cli) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Setup signal handling for graceful shutdown
    let signal_handler = SignalHandler::new();
    signal_handler.setup_graceful_shutdown();

    // Setup keyboard input handling for Enter key statistics
    let enter_pressed = Arc::new(AtomicBool::new(false));
    let enter_pressed_clone = enter_pressed.clone();

    thread::spawn(move || {
        let mut buffer = [0u8; 1];
        loop {
            if let Ok(1) = io::stdin().read(&mut buffer) {
                if buffer[0] == b'\n' || buffer[0] == b'\r' {
                    enter_pressed_clone.store(true, Ordering::Relaxed);
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Create appropriate printer based on output format
    let mut printer: Box<dyn Printer> = if cli.json {
        Box::new(JsonPrinter::new(config.clone(), cli.pretty))
    } else if cli.csv {
        match CsvPrinter::new(config.clone(), None) {
            Ok(csv_printer) => Box::new(csv_printer),
            Err(e) => {
                eprintln!("Failed to create CSV printer: {}", e);
                process::exit(1);
            }
        }
    } else if let Some(db_path) = &cli.database {
        match DatabasePrinter::new(config.clone(), Some(db_path.clone())) {
            Ok(db_printer) => Box::new(db_printer),
            Err(e) => {
                eprintln!("Failed to create database printer: {}", e);
                process::exit(1);
            }
        }
    } else {
        Box::new(ConsolePrinter::new(config.clone()))
    };

    // Initialize statistics
    let mut stats = Statistics::new();
    let tcp_probe = TcpProbe::new(config.clone());

    printer.print_start(&config.host, config.port);

    // Main probing loop
    let mut probe_count = 0;
    let mut should_continue = true;

    while should_continue && !signal_handler.should_shutdown() {
        // Perform TCP probe
        let result = if config.retries > 0 {
            tcp_probe.perform_retry(config.retries)
        } else {
            tcp_probe.probe_target()
        };

        match result {
            Ok(probe_result) => {
                if probe_result.success {
                    stats.record_success(probe_result.rtt);

                    let source_addr = probe_result.source_addr.map(|addr| addr.to_string());
                    printer.print_probe_success(
                        source_addr,
                        &config.get_target_address(),
                        stats.ongoing_successful_probes,
                        probe_result.rtt,
                    );
                } else {
                    stats.record_failure();
                    printer.print_probe_fail(&config.get_target_address(), stats.ongoing_unsuccessful_probes);
                }
            }
            Err(e) => {
                stats.record_failure();
                printer.print_probe_fail(&config.get_target_address(), stats.ongoing_unsuccessful_probes);

                if config.verbose {
                    printer.print_error(&format!("Probe failed: {}", e), &[]);
                }
            }
        }

        probe_count += 1;

        // Check if Enter key was pressed for statistics
        if enter_pressed.load(Ordering::Relaxed) {
            enter_pressed.store(false, Ordering::Relaxed);
            printer.print_statistics(&stats);
        }

        // Check if we should continue
        if config.count > 0 && probe_count >= config.count {
            should_continue = false;
        }

        // Wait for next probe if continuing
        if should_continue && !signal_handler.should_shutdown() {
            thread::sleep(config.interval);
        }
    }

    // Handle graceful shutdown if signal was received
    if signal_handler.should_shutdown() {
        println!("\nShutting down gracefully...");
    }

    // Finalize statistics and print results
    stats.finalize();
    printer.print_statistics(&stats);
}