use crate::Session;
use chrono::{NaiveDate, NaiveTime};

#[derive(Debug)]
pub struct LogEntry {
    pub date: NaiveDate,
    pub time_began: NaiveTime,
    pub time_completed: NaiveTime,
    pub work_activity: Vec<String>,
}

impl LogEntry {
    pub fn new(
        date: NaiveDate,
        time_began: NaiveTime,
        time_completed: NaiveTime,
        work_activity: Vec<String>,
    ) -> Self {
        Self {
            date,
            time_began,
            time_completed,
            work_activity,
        }
    }

    pub fn hours(&self) -> f64 {
        let duration = self.time_completed.signed_duration_since(self.time_began);
        duration.num_seconds() as f64 / 3600.0
    }
}

pub fn sessions_to_log_entries(sessions: &[Session]) -> Vec<LogEntry> {
    sessions
        .iter()
        .map(|session| {
            LogEntry::new(
                session.start.date_naive(),
                session.start.time(),
                session.end.time(),
                session
                    .tags
                    .iter()
                    .map(|tag| tag.note.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}
