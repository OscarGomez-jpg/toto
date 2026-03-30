use crate::domain::task::Task;
use std::error::Error;

use chrono::{DateTime, Utc};

#[cfg_attr(test, mockall::automock)]
pub trait TaskRepository: Send + Sync {
    fn add(
        &self,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>>;
    fn get_all(&self) -> Result<Vec<Task>, Box<dyn Error>>;
    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>>;
    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>>;
    fn update_content(
        &self,
        id: String,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>>;
    fn remove(&self, id: String) -> Result<bool, Box<dyn Error>>;
    fn clear_completed(&self) -> Result<usize, Box<dyn Error>>;
    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>>;
    fn upsert_from_external(&self, task: Task) -> Result<(), Box<dyn Error>>;
}
