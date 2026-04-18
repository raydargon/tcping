// Statistics tracking module for TCPing

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Statistics {
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub start_of_uptime: Option<Instant>,
    pub start_of_downtime: Option<Instant>,
    pub last_successful_probe: Option<Instant>,
    pub last_unsuccessful_probe: Option<Instant>,
    pub rtt_measurements: Vec<f32>,
    pub ongoing_successful_probes: u32,
    pub ongoing_unsuccessful_probes: u32,
    pub total_successful_probes: u32,
    pub total_unsuccessful_probes: u32,
    pub total_downtime: Duration,
    pub total_uptime: Duration,
    pub longest_uptime: Duration,
    pub longest_downtime: Duration,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            end_time: None,
            start_of_uptime: None,
            start_of_downtime: None,
            last_successful_probe: None,
            last_unsuccessful_probe: None,
            rtt_measurements: Vec::new(),
            ongoing_successful_probes: 0,
            ongoing_unsuccessful_probes: 0,
            total_successful_probes: 0,
            total_unsuccessful_probes: 0,
            total_downtime: Duration::ZERO,
            total_uptime: Duration::ZERO,
            longest_uptime: Duration::ZERO,
            longest_downtime: Duration::ZERO,
        }
    }
}

impl Statistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&mut self, rtt: f32) {
        let now = Instant::now();

        // Update ongoing streak
        self.ongoing_successful_probes += 1;
        self.ongoing_unsuccessful_probes = 0;

        // Update total counts
        self.total_successful_probes += 1;

        // Record RTT
        self.rtt_measurements.push(rtt);

        // Update timing
        self.last_successful_probe = Some(now);

        // Handle uptime/downtime tracking
        if self.start_of_uptime.is_none() {
            self.start_of_uptime = Some(now);
        }

        // If we were in downtime, record it and start uptime
        if let Some(downtime_start) = self.start_of_downtime {
            let downtime_duration = now.duration_since(downtime_start);
            self.total_downtime += downtime_duration;

            if downtime_duration > self.longest_downtime {
                self.longest_downtime = downtime_duration;
            }

            self.start_of_downtime = None;
            self.start_of_uptime = Some(now);
        }
    }

    pub fn record_failure(&mut self) {
        let now = Instant::now();

        // Update ongoing streak
        self.ongoing_unsuccessful_probes += 1;
        self.ongoing_successful_probes = 0;

        // Update total counts
        self.total_unsuccessful_probes += 1;

        // Update timing
        self.last_unsuccessful_probe = Some(now);

        // Handle uptime/downtime tracking
        if self.start_of_downtime.is_none() {
            self.start_of_downtime = Some(now);
        }

        // If we were in uptime, record it and start downtime
        if let Some(uptime_start) = self.start_of_uptime {
            let uptime_duration = now.duration_since(uptime_start);
            self.total_uptime += uptime_duration;

            if uptime_duration > self.longest_uptime {
                self.longest_uptime = uptime_duration;
            }

            self.start_of_uptime = None;
            self.start_of_downtime = Some(now);
        }
    }

    pub fn get_packet_loss_percentage(&self) -> f32 {
        let total_probes = self.total_successful_probes + self.total_unsuccessful_probes;
        if total_probes == 0 {
            return 0.0;
        }

        (self.total_unsuccessful_probes as f32 / total_probes as f32) * 100.0
    }

    pub fn get_rtt_statistics(&self) -> (f32, f32, f32) {
        if self.rtt_measurements.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let min = *self.rtt_measurements.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = *self.rtt_measurements.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let avg = self.rtt_measurements.iter().sum::<f32>() / self.rtt_measurements.len() as f32;

        (min, avg, max)
    }

    pub fn finalize(&mut self) {
        self.end_time = Some(Instant::now());

        // Finalize any ongoing uptime/downtime periods
        let now = self.end_time.unwrap();

        if let Some(uptime_start) = self.start_of_uptime {
            let uptime_duration = now.duration_since(uptime_start);
            self.total_uptime += uptime_duration;
            if uptime_duration > self.longest_uptime {
                self.longest_uptime = uptime_duration;
            }
        }

        if let Some(downtime_start) = self.start_of_downtime {
            let downtime_duration = now.duration_since(downtime_start);
            self.total_downtime += downtime_duration;
            if downtime_duration > self.longest_downtime {
                self.longest_downtime = downtime_duration;
            }
        }
    }

    pub fn get_total_duration(&self) -> Duration {
        self.end_time.unwrap_or_else(Instant::now).duration_since(self.start_time)
    }
}