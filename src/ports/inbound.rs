use crate::domain::task::Task;
use std::error::Error;

use chrono::{DateTime, Utc};

#[cfg_attr(test, mockall::automock)]
pub trait TaskServicePort: Send + Sync {
    fn add_task(
        &self,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>>;
    fn get_all_tasks(&self) -> Result<Vec<Task>, Box<dyn Error>>;
    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>>;
    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>>;
    fn update_task_content(
        &self,
        id: String,
        content: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>>;
    fn remove_task(&self, id: String) -> Result<String, Box<dyn Error>>;
    fn clear_completed_tasks(&self) -> Result<String, Box<dyn Error>>;
    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>>;
    fn sync_jira(
        &self,
        config: crate::adapters::tui::config::JiraConfig,
    ) -> Result<String, Box<dyn Error>>;
}
