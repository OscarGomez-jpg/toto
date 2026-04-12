use crate::domain::service::TaskService;
use crate::ports::inbound::TaskServicePort;
use chrono::{DateTime, Utc};
use std::error::Error;

/// The result of a command execution.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// Simple success with no data.
    Success,
    /// Success returning a string (e.g., a new Task ID or message).
    Id(String),
    /// Success with a count (e.g., cleared tasks).
    Count(usize),
}

/// The Command trait for all domain operations.
pub trait Command: Send + Sync {
    /// Executes the command against the task service.
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>>;
}

/// Command to add a new task.
pub struct AddTaskCommand {
    pub title: String,
    pub description: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl Command for AddTaskCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        let id = service.add_task(
            self.title.clone(),
            self.description.clone(),
            self.start_date,
            self.end_date,
        )?;
        Ok(CommandResult::Id(id))
    }
}

/// Command to update an existing task.
pub struct UpdateTaskCommand {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl Command for UpdateTaskCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        service.update_task(
            self.id.clone(),
            self.title.clone(),
            self.description.clone(),
            self.start_date,
            self.end_date,
        )?;
        Ok(CommandResult::Success)
    }
}

/// Command to remove a task.
pub struct RemoveTaskCommand {
    pub id: String,
}

impl Command for RemoveTaskCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        let msg = service.remove_task(self.id.clone())?;
        Ok(CommandResult::Id(msg))
    }
}

/// Command to toggle task completion.
pub struct ToggleCompletedCommand {
    pub id: String,
}

impl Command for ToggleCompletedCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        service.toggle_completed(self.id.clone())?;
        Ok(CommandResult::Success)
    }
}

/// Command to toggle task importance.
pub struct ToggleImportantCommand {
    pub id: String,
}

impl Command for ToggleImportantCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        service.toggle_important(self.id.clone())?;
        Ok(CommandResult::Success)
    }
}

/// Command to clear completed tasks.
pub struct ClearCompletedCommand;

impl Command for ClearCompletedCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        let msg = service.clear_completed_tasks()?;
        Ok(CommandResult::Id(msg))
    }
}

/// Command to move a task.
pub struct MoveTaskCommand {
    pub id: String,
    pub delta: i32,
}

impl Command for MoveTaskCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        service.move_task(self.id.clone(), self.delta)?;
        Ok(CommandResult::Success)
    }
}

/// Command to sync with Jira.
pub struct SyncJiraCommand {
    pub config: crate::adapters::tui::config::JiraConfig,
}

impl Command for SyncJiraCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        let msg = service.sync_jira(self.config.clone())?;
        Ok(CommandResult::Id(msg))
    }
}

/// Command to add a tag.
pub struct AddTagCommand {
    pub id: String,
    pub tag: String,
}

impl Command for AddTagCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        service.add_tag(self.id.clone(), self.tag.clone())?;
        Ok(CommandResult::Success)
    }
}

/// Command to relate two tasks.
pub struct AddRelationCommand {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: crate::domain::task::RelationType,
}

impl Command for AddRelationCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        use crate::ports::inbound::TaskServicePort;
        service.add_relation(
            self.source_id.clone(),
            self.target_id.clone(),
            self.relation_type.clone(),
        )?;
        Ok(CommandResult::Success)
    }
}
