// TODO fix output to csv and pdf, fix time formatting to elide decimals on report,
// todo, prevent overlapping intevals
// todo, session management, delete old periods, etc.
// Git import commit to work activity command
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

mod output;
mod ratatui_interface;

#[derive(Parser)]
#[command(version, about, long_about)]
struct Cli {
    #[arg(env = "BILLS_PATH")]
    path: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        format: OutputFormat,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Generate CSV output
    Csv,
    /// Generate PDF output
    Pdf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Period {
    sessions: Vec<Session>,
}

impl Period {
    fn hours(&self) -> f64 {
        self.sessions
            .iter()
            .map(|session| {
                let duration = session.end - session.start;
                duration.num_seconds() as f64 / 3600.0
            })
            .sum()
    }

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

            let entries = output::sessions_to_log_entries(&period.sessions);

            format!(
                include_str!("template.txt"),
                start_date,
                end_date,
                (period.hours() * 10.0).round() / 10.0,
                entries
                    .iter()
                    .map(|entry| {
                        let taglist = if entry.work_activity.is_empty() {
                            "".to_string()
                        } else {
                            format!(
                                "\\begin{{itemize}}{}\\end{{itemize}}",
                                entry
                                    .work_activity
                                    .iter()
                                    .map(|tag| format!("\\item {}", tag))
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            )
                        };

                        format!(
                            "{} & {} & {} & {} & {:.1} \\\\",
                            entry.date,
                            entry.time_began.format("%I:%M %p"),
                            entry.time_completed.format("%I:%M %p"),
                            taglist,
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
    start: DateTime<Local>,
    end: DateTime<Local>,
    hourly_rate: f64,
    tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tag {
    note: String,
    time: DateTime<Local>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let Some(path) = cli.path else {
        panic!("No path provided, either specify a path using the --bills-path flag or set the BILLS_PATH environment variable");
    };
    // Ensure the directory exists
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    let data_path = path.join("bills.json");

    match cli.command {
        Commands::Session {
            hourly_rate,
            catch_up,
        } => {
            // Read existing sessions from file
            let mut periods = read_period_from_file(&data_path)?;
            let last_period = match periods.last_mut() {
                Some(period) => period,
                None => {
                    periods.push(Period {
                        sessions: Vec::new(),
                    });
                    periods.last_mut().unwrap()
                }
            };

            // Run the ratatui interface
            let new_session = ratatui_interface::run(hourly_rate, catch_up).await?;

            // Add the new session and write all sessions back to file
            last_period.sessions.push(new_session);
            write_periods(&data_path, &periods)?;
        }
        Commands::Output { format } => {
            // Read sessions from file
            let periods = read_period_from_file(&data_path)?;
            let last_period = match periods.last() {
                Some(period) => period,
                None => {
                    eprintln!("No sessions found");
                    return Ok(());
                }
            };

            match format {
                OutputFormat::Csv => {}
                OutputFormat::Pdf => {
                    let latex = last_period.to_latex();
                    let output_folder = path.join("output");

                    if !output_folder.exists() {
                        fs::create_dir_all(&output_folder)?;
                    }

                    let timestamp = Local::now().format("%+");

                    let tex_file = output_folder.join(format!("output-{}.tex", timestamp));
                    std::fs::write(tex_file, latex.clone()).unwrap();

                    let pdf_data = tectonic::latex_to_pdf(latex).unwrap();
                    let pdf_file = output_folder.join(format!("output-{}.pdf", timestamp));
                    std::fs::write(pdf_file, pdf_data).unwrap();
                }
            }
        }
        Commands::NewPeriod => {
            let mut sessions = read_period_from_file(&data_path)?;
            sessions.push(Period {
                sessions: Vec::new(),
            });
            write_periods(&data_path, &sessions)?;
        }
    };

    Ok(())
}

fn read_period_from_file(path: &PathBuf) -> Result<Vec<Period>, Box<dyn std::error::Error>> {
    if path.exists() {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(Vec::new())
    }
}

fn write_periods(path: &PathBuf, sessions: &[Period]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(sessions)?;
    std::fs::write(path, json)?;
    Ok(())
}
