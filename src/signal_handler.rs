// Signal handling module for graceful shutdown

use tokio::signal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn setup_graceful_shutdown(&self) {
        let flag = self.shutdown_flag.clone();

        // Handle SIGINT (Ctrl+C)
        let flag_int = flag.clone();
        tokio::spawn(async move {
            signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            flag_int.store(true, Ordering::SeqCst);
            println!("\nReceived SIGINT, shutting down gracefully...");
        });

        // Handle SIGTERM
        let flag_term = flag.clone();
        tokio::spawn(async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).expect("Failed to listen for SIGTERM");
                sigterm.recv().await;
                flag_term.store(true, Ordering::SeqCst);
                println!("\nReceived SIGTERM, shutting down gracefully...");
            }
            #[cfg(not(unix))]
            {
                // On non-Unix systems, SIGTERM might not be available
                // We'll rely on SIGINT for graceful shutdown
            }
        });

        // Handle Enter key press for statistics display
        let flag_enter = flag.clone();
        thread::spawn(move || {
            let mut input = String::new();
            loop {
                if std::io::stdin().read_line(&mut input).is_ok() {
                    if input.trim().is_empty() {
                        println!("\nStatistics requested...");
                        // This could trigger statistics display without shutting down
                        // For now, we'll just log the request
                    }
                    input.clear();
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    pub fn wait_for_shutdown(&self) {
        while !self.should_shutdown() {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}