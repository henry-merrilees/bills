use clap::Parser;
use std::io::Write;

#[derive(clap::Parser)]
#[command(version, about, long_about)]
struct Cli {
    #[arg(env = "HOURLY_RATE")]
    hourly_rate: f64,
    /// Use to catch up on the time you missed (minutes)
    #[arg(long)]
    catch_up: Option<f64>,
}

fn main() {
    let cli = Cli::parse();
    let hourly_rate = cli.hourly_rate;

    let minimum_uptate_time = std::time::Duration::from_secs_f64(36.0 / hourly_rate); // seconds per cent
    let catch_up_duration = std::time::Duration::from_secs_f64(cli.catch_up.unwrap_or(0.0) * 60.0);

    let start = std::time::Instant::now();
    let start = start - catch_up_duration;
    loop {
        let elapsed = start.elapsed();
        let seconds = elapsed.as_secs();
        let (hours, minutes, seconds) = (seconds / 3600, (seconds % 3600) / 60, seconds % 60);

        let earned_money = hourly_rate / 3600.0 * elapsed.as_secs_f64();

        // clear last line
        let mut stdout = std::io::stdout();

        print!(
            "\rTime: {:02}:{:02}:{:02}. Earned: ${:.2}.",
            hours, minutes, seconds, earned_money
        );

        stdout.flush().unwrap();

        std::thread::sleep(minimum_uptate_time);
    }
}
