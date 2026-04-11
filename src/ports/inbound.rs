use crate::domain::task::Task;
use std::error::Error;

use chrono::{DateTime, Utc};

/// Defines the primary interface for interacting with the task service.
/// 
/// This port is used by inbound adapters (like the TUI or CLI) to perform
/// actions on the domain.
#[cfg_attr(test, mockall::automock)]
pub trait TaskServicePort: Send + Sync {
    /// Adds a new task with optional start and end dates.
    fn add_task(
        &self,
        title: String,
        description: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>>;

    /// Retrieves all tasks, applying any active filters (e.g., search).
    fn get_all_tasks(&self) -> Result<Vec<Task>, Box<dyn Error>>;

    /// Toggles the completion status of a task by ID.
    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>>;

    /// Toggles the important/prioritized status of a task by ID.
    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>>;

    /// Updates the content and dates of an existing task.
    fn update_task(
        &self,
        id: String,
        title: String,
        description: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>>;

    /// Removes a task by ID.
    fn remove_task(&self, id: String) -> Result<String, Box<dyn Error>>;

    /// Deletes all tasks currently marked as completed.
    fn clear_completed_tasks(&self) -> Result<String, Box<dyn Error>>;

    /// Moves a task up or down in the list order.
    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>>;

    /// Synchronizes tasks from an external Jira project based on configuration.
    fn sync_jira(&self, config: crate::adapters::tui::config::JiraConfig) -> Result<String, Box<dyn Error>>;
}
