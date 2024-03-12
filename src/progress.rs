use std::time::SystemTime;

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
