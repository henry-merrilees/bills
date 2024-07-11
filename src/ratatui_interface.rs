use chrono::{DateTime, Duration, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

use crate::{Session, Tag};

struct App {
    start_time: DateTime<Local>,
    hourly_rate: f64,
    tags: Vec<Tag>,
    input: String,
}

impl App {
    fn new(hourly_rate: f64, catch_up: Option<f64>) -> App {
        App {
            start_time: Local::now()
                .checked_sub_signed(Duration::minutes(catch_up.unwrap_or(0.0) as i64))
                .expect("Invalid time"),
            hourly_rate,
            tags: Vec::new(),
            input: String::new(),
        }
    }

    fn add_tag(&mut self) {
        if !self.input.is_empty() {
            self.tags.push(Tag {
                note: self.input.clone(),
                time: Local::now(),
            });
            self.input.clear();
        }
    }
}

pub async fn run(hourly_rate: f64, catch_up: Option<f64>) -> io::Result<Session> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::milliseconds((36000.0 / hourly_rate) as i64);

    // Create app and run it
    let app = App::new(hourly_rate, catch_up);
    let session = run_app(&mut terminal, app, tick_rate);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    session
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    // in minutes
) -> io::Result<Session> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if crossterm::event::poll(tick_rate.to_std().unwrap())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => app.add_tag(),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        return Ok(Session {
                            start: app.start_time,
                            end: Local::now(),
                            hourly_rate: app.hourly_rate,
                            tags: app.tags,
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let now = Local::now();
    let elapsed = now - app.start_time;
    let earned_money = app.hourly_rate * elapsed.num_seconds() as f64 / 3600.0;

    let time_money = Paragraph::new(Span::from(format!(
        "Time: {:02}:{:02}:{:02} Earned: ${:.2}",
        elapsed.num_hours(),
        elapsed.num_minutes() % 60,
        elapsed.num_seconds() % 60,
        earned_money
    )))
    .block(Block::default().borders(Borders::ALL).title("Session Info"));
    f.render_widget(time_money, chunks[0]);

    let tags: Vec<ListItem> = app
        .tags
        .iter()
        .map(|t| {
            ListItem::new(Span::from(format!(
                "{} -- {}",
                t.time.format("%H:%M:%S"),
                t.note
            )))
        })
        .collect();

    let tags = List::new(tags)
        .block(Block::default().borders(Borders::ALL).title("Tags"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_widget(tags, chunks[1]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);
    f.set_cursor(chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1)
}
