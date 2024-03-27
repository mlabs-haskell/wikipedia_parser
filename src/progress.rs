use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Progress {
    pub total: u64,
    pub rate_divider: f64,
    pub rate_unit: &'static str,
    pub start: SystemTime,
    pub window_length: Duration,
    pub window_start: SystemTime,
    pub window_count: u64,
}

impl Progress {
    pub fn progress(&mut self, count: u64, now: SystemTime) -> String {
        let percent = 100.0 * (count as f64) / (self.total as f64);

        let window_elapsed = now.duration_since(self.window_start).unwrap();
        let rate = (count - self.window_count) as f64 / window_elapsed.as_secs_f64();

        let elapsed = now.duration_since(self.start).unwrap();
        let rate_avg = count as f64 / elapsed.as_secs_f64();

        let eta_remaining_secs = (self.total - count) as f64 / rate;
        let eta_remaining_mins_part = (eta_remaining_secs / 60.0).floor();
        let eta_remaining_secs_part = (eta_remaining_secs % 60.0).floor();

        let eta_total_secs =
            eta_remaining_secs + now.duration_since(self.start).unwrap().as_secs_f64();
        let eta_total_mins_part = (eta_total_secs / 60.0).floor();
        let eta_total_secs_part = (eta_total_secs % 60.0).floor();

        let rate = rate / self.rate_divider;
        let rate_avg = rate_avg / self.rate_divider;

        let ret = format!(
            "{:.2}% {}/{} | {:.2} {} ({:.2} {}) | ETA {:02.}:{:02.} mins ({:02.}:{:02.} mins total)",
            percent,
            count,
            self.total,
            rate,
            self.rate_unit,
            rate_avg,
            self.rate_unit,
            eta_remaining_mins_part,
            eta_remaining_secs_part,
            eta_total_mins_part,
            eta_total_secs_part,
        );

        if window_elapsed > self.window_length {
            self.window_start = now;
            self.window_count = count;
        }

        ret
    }
}

pub fn progress(
    count: u64,
    total: u64,
    rate_divider: f64,
    rate_unit: &str,
    start: SystemTime,
    now: SystemTime,
) -> String {
    let percent = 100.0 * (count as f64) / (total as f64);

    let elapsed = now.duration_since(start).unwrap();
    let rate = count as f64 / elapsed.as_secs_f64();

    let eta_remaining_secs = (total - count) as f64 / rate;
    let eta_remaining_mins_part = (eta_remaining_secs / 60.0).floor();
    let eta_remaining_secs_part = (eta_remaining_secs % 60.0).floor();

    let eta_total_secs = total as f64 / rate;
    let eta_total_mins_part = (eta_total_secs / 60.0).floor();
    let eta_total_secs_part = (eta_total_secs % 60.0).floor();

    let rate = rate / rate_divider;

    format!(
        "{:.2}% {}/{} | {:.2} {} | ETA {:02.}:{:02.} mins ({:02.}:{:02.} mins total)",
        percent,
        count,
        total,
        rate,
        rate_unit,
        eta_remaining_mins_part,
        eta_remaining_secs_part,
        eta_total_mins_part,
        eta_total_secs_part,
    )
}
