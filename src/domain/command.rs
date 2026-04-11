use std::error::Error;
use crate::domain::service::TaskService;
use crate::ports::inbound::TaskServicePort;
use chrono::{DateTime, Utc};

/// The result of a command execution.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// Simple success with no data.
    Success,
    /// Success returning a string (e.g., a new Task ID).
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

/// Command to toggle task completion.
pub struct ToggleCompletedCommand {
    pub id: String,
}

impl Command for ToggleCompletedCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
        service.toggle_completed(self.id.clone())?;
        Ok(CommandResult::Success)
    }
}

/// Command to add a tag.
pub struct AddTagCommand {
    pub id: String,
    pub tag: String,
}

impl Command for AddTagCommand {
    fn execute(&self, service: &TaskService) -> Result<CommandResult, Box<dyn Error>> {
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
        service.add_relation(
            self.source_id.clone(),
            self.target_id.clone(),
            self.relation_type.clone(),
        )?;
        Ok(CommandResult::Success)
    }
}
