#[derive(Debug)]
pub(crate) struct LogEntry {
    pub(crate) date: chrono::NaiveDate,
    pub(crate) time_began: chrono::NaiveTime,
    pub(crate) time_completed: chrono::NaiveTime,
    pub(crate) work_activity: String,
}

impl LogEntry {
    pub(crate) fn new(
        date: chrono::NaiveDate,
        time_began: chrono::NaiveTime,
        time_completed: chrono::NaiveTime,
        work_activity: String,
    ) -> Self {
        Self {
            date,
            time_began,
            time_completed,
            work_activity,
        }
    }

    pub(crate) fn hours(&self) -> f64 {
        let duration =
            chrono::NaiveTime::signed_duration_since(self.time_completed, self.time_began);
        duration.num_seconds() as f64 / 3600.0
    }
}
