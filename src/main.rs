#![feature(trivial_bounds)]

// TODO move latex pdf to feature
use clap::{clap_derive::Subcommand, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

const NAME: &str = "Henry Merrilees";

mod output;

#[derive(clap::Parser)]
#[command(version, about, long_about)]
struct Cli {
    #[arg(env = "BILLS_PATH")]
    bills_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(Subcommand)]
enum Subcommand {
    Session {
        #[arg(env = "HOURLY_RATE")]
        hourly_rate: f64,
        /// Use to catch up on the time you missed (minutes)
        #[arg(long)]
        catch_up: Option<f64>,
    },
    NewPeriod,
    Output {
        #[arg(value_enum)]
        format: Output,
    },
}

#[allow(clippy::upper_case_acronyms)]
#[derive(ValueEnum, Clone)]
enum Output {
    PDF,
    CSV,
}

#[derive(Debug, Serialize, Deserialize)]
struct Log {
    periods: Vec<Period>,
}

impl Default for Log {
    fn default() -> Self {
        // Start with a period
        Self {
            periods: vec![Period::default()],
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Period {
    sessions: Vec<Session>,
}

impl Period {
    fn to_latex(&self) -> String {
        {
            let period = self;
            let start_date = period
                .sessions
                .first()
                .expect("Period must have at least one session")
                .start
                .date_naive();
            let end_date = period
                .sessions
                .last()
                .expect("Period must have at least one session")
                .end
                .date_naive();

            let entries = period
                .sessions
                .iter()
                .map(Session::to_log_entry)
                .collect::<Vec<_>>();

            format!(
                include_str!("template.txt"),
                start_date,
                end_date,
                period.earned(),
                entries
                    .iter()
                    .map(|entry| {
                        format!(
                            "{} & {} & {} & {} & {:.1} \\\\",
                            entry.date,
                            entry.time_began,
                            entry.time_completed,
                            entry.work_activity,
                            entry.hours()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    start: chrono::DateTime<chrono::Local>,
    end: chrono::DateTime<chrono::Local>,
    hourly_rate: f64,
    tags: Vec<Tag>,
}
impl Session {
    pub(crate) fn to_log_entry(&self) -> output::LogEntry {
        output::LogEntry::new(
            self.start.date_naive(),
            self.start.time(),
            self.end.time(),
            self.tags
                .iter()
                .map(|tag| tag.note.to_owned())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

// just wanted to see if this was possible. Wow this is disgusting.
trait Span {
    fn earned(&self) -> f64;
}

impl Span for Session {
    fn earned(&self) -> f64 {
        self.hourly_rate / 3600.0 * self.end.signed_duration_since(self.start).num_seconds() as f64
    }
}

impl Span for Period {
    fn earned(&self) -> f64 {
        self.sessions.iter().map(Span::earned).sum()
    }
}

impl Span for Log {
    fn earned(&self) -> f64 {
        self.periods.iter().map(Span::earned).sum()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Tag {
    note: String,
    time: chrono::DateTime<chrono::Local>,
}

fn main() {
    let cli = Cli::parse();

    let path = cli
        .bills_path
        .unwrap_or(dirs::home_dir().unwrap().join("bills.json"));

    let mut log = match std::fs::File::open(&path) {
        Ok(file) => serde_json::from_reader(file).unwrap(),
        Err(_) => Log::default(),
    };

    match cli.command {
        Subcommand::Session {
            hourly_rate,
            catch_up,
        } => {
            let period = log.periods.last_mut().expect("default populates periods");
            session(period, hourly_rate, catch_up.unwrap_or(0.0));
        }
        Subcommand::NewPeriod => {
            log.periods.push(Period::default());
        }
        Subcommand::Output {
            format: Output::PDF,
        } => {
            let period = log.periods.last().expect("default populates periods");
            let latex = period.to_latex();
            // write
            std::fs::write("output.tex", latex.clone()).unwrap();
            let pdf_data = tectonic::latex_to_pdf(latex).unwrap();
            std::fs::write("output.pdf", pdf_data).unwrap();
        }
        Subcommand::Output {
            format: Output::CSV,
        } => {
            unimplemented!()
        }
    }

    let new_contents = serde_json::to_string_pretty(&log).unwrap();
    std::fs::write(&path, new_contents).unwrap();
}

fn session(period: &mut Period, hourly_rate: f64, catch_up: f64) {
    let minimum_uptate_time = std::time::Duration::from_secs_f64(36.0 / hourly_rate); // seconds per cent
    let catch_up_duration = std::time::Duration::from_secs_f64(catch_up * 60.0); // minutes to seconds

    // TODO tag input mechanism

    let start = chrono::Local::now();
    let start = start - catch_up_duration;
    loop {
        let elapsed = start.signed_duration_since(chrono::Local::now());
        let seconds = elapsed.num_seconds();
        let (hours, minutes, seconds) = (seconds / 3600, (seconds % 3600) / 60, seconds % 60);

        let earned_money = hourly_rate / 3600.0 * elapsed.num_seconds() as f64;

        // clear last line
        let mut stdout = std::io::stdout();

        print!(
            "\rTime: {:02}:{:02}:{:02}. Earned: ${:.2}.",
            hours, minutes, seconds, earned_money
        );

        stdout.flush().unwrap();

        std::thread::sleep(minimum_uptate_time);
        // wait for input
        if std::io::stdin().read(&mut [0u8]).is_ok() {
            break;
        }
    }

    let end = chrono::Local::now();

    period.sessions.push(Session {
        start,
        end,
        hourly_rate,
        tags: Vec::new(),
    });
}
