use chrono::{DateTime, Utc};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// Represents the source of a task.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskSource {
    /// Task was created locally within the application.
    Local,
    /// Task was synchronized from an external Jira project.
    Jira,
}

/// Base trait for all task features (plugins).
pub trait TaskFeature: Send + Sync + Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn feature_type(&self) -> &'static str;
}

/// Feature: Basic metadata like title and description.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataFeature {
    pub title: String,
    pub description: String,
}

impl TaskFeature for MetadataFeature {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn feature_type(&self) -> &'static str { "metadata" }
}

/// Feature: Completion and importance status.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StatusFeature {
    pub completed: bool,
    pub important: bool,
}

impl TaskFeature for StatusFeature {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn feature_type(&self) -> &'static str { "status" }
}

/// Feature: Scheduling dates.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScheduleFeature {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl TaskFeature for ScheduleFeature {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn feature_type(&self) -> &'static str { "schedule" }
}

/// Feature: External integration data (e.g., Jira).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExternalFeature {
    pub external_id: Option<String>,
    pub source: TaskSource,
}

impl TaskFeature for ExternalFeature {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn feature_type(&self) -> &'static str { "external" }
}

/// Feature: Tags to relate and categorize tasks.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TagsFeature {
    pub tags: Vec<String>,
}

impl TaskFeature for TagsFeature {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn feature_type(&self) -> &'static str { "tags" }
}

/// Types of relationships between tasks.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RelationType {
    Blocks,
    BlockedBy,
    RelatedTo,
    Subtask,
    Parent,
}

/// A relationship to another task.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskRelation {
    pub target_id: String,
    pub relation_type: RelationType,
}

/// Feature: Relationships to other tasks.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelationsFeature {
    pub relations: Vec<TaskRelation>,
}

impl TaskFeature for RelationsFeature {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn feature_type(&self) -> &'static str { "relations" }
}

/// The core domain model for a single task, now a container for features.
pub struct Task {
    pub id: String,
    features: HashMap<&'static str, Box<dyn TaskFeature>>,
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("features", &self.features.keys())
            .finish()
    }
}

impl Task {
    pub fn get_feature<T: TaskFeature + 'static>(&self) -> Option<&T> {
        for feature in self.features.values() {
            if let Some(downcasted) = feature.as_any().downcast_ref::<T>() {
                return Some(downcasted);
            }
        }
        None
    }

    pub fn get_feature_mut<T: TaskFeature + 'static>(&mut self) -> Option<&mut T> {
        for feature in self.features.values_mut() {
            if let Some(downcasted) = feature.as_any_mut().downcast_mut::<T>() {
                return Some(downcasted);
            }
        }
        None
    }

    /// Helper for legacy compatibility: get title.
    pub fn title(&self) -> String {
        self.get_feature::<MetadataFeature>()
            .map(|f| f.title.clone())
            .unwrap_or_default()
    }

    /// Helper for legacy compatibility: get description.
    pub fn description(&self) -> String {
        self.get_feature::<MetadataFeature>()
            .map(|f| f.description.clone())
            .unwrap_or_default()
    }

    /// Helper for legacy compatibility: check if completed.
    pub fn is_completed(&self) -> bool {
        self.get_feature::<StatusFeature>()
            .map(|f| f.completed)
            .unwrap_or(false)
    }

    /// Helper for legacy compatibility: check if important.
    pub fn is_important(&self) -> bool {
        self.get_feature::<StatusFeature>()
            .map(|f| f.important)
            .unwrap_or(false)
    }

    /// Helper for legacy compatibility: get start date.
    pub fn start_date(&self) -> Option<DateTime<Utc>> {
        self.get_feature::<ScheduleFeature>().and_then(|f| f.start_date)
    }

    /// Helper for legacy compatibility: get end date.
    pub fn end_date(&self) -> Option<DateTime<Utc>> {
        self.get_feature::<ScheduleFeature>().and_then(|f| f.end_date)
    }

    /// Helper for legacy compatibility: get external id.
    pub fn external_id(&self) -> Option<String> {
        self.get_feature::<ExternalFeature>().and_then(|f| f.external_id.clone())
    }

    /// Helper for legacy compatibility: get source.
    pub fn source(&self) -> TaskSource {
        self.get_feature::<ExternalFeature>()
            .map(|f| f.source.clone())
            .unwrap_or(TaskSource::Local)
    }

    /// Get all tags associated with this task.
    pub fn tags(&self) -> Vec<String> {
        self.get_feature::<TagsFeature>()
            .map(|f| f.tags.clone())
            .unwrap_or_default()
    }

    /// Get all relations to other tasks.
    pub fn relations(&self) -> Vec<TaskRelation> {
        self.get_feature::<RelationsFeature>()
            .map(|f| f.relations.clone())
            .unwrap_or_default()
    }

    /// Validates that the start date is not after the end date.
    pub fn is_valid_range(&self) -> bool {
        match (self.start_date(), self.end_date()) {
            (Some(start), Some(end)) => start <= end,
            _ => true,
        }
    }
}

/// Builder for constructing tasks with specific features.
pub struct TaskBuilder {
    id: String,
    features: HashMap<&'static str, Box<dyn TaskFeature>>,
}

impl TaskBuilder {
    pub fn new(id: String) -> Self {
        Self {
            id,
            features: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, title: String, description: String) -> Self {
        self.features.insert("metadata", Box::new(MetadataFeature { title, description }));
        self
    }

    pub fn with_status(mut self, completed: bool, important: bool) -> Self {
        self.features.insert("status", Box::new(StatusFeature { completed, important }));
        self
    }

    pub fn with_schedule(mut self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Self {
        self.features.insert("schedule", Box::new(ScheduleFeature { start_date, end_date }));
        self
    }

    pub fn with_external(mut self, external_id: Option<String>, source: TaskSource) -> Self {
        self.features.insert("external", Box::new(ExternalFeature { external_id, source }));
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.features.insert("tags", Box::new(TagsFeature { tags }));
        self
    }

    pub fn with_relations(mut self, relations: Vec<TaskRelation>) -> Self {
        self.features.insert("relations", Box::new(RelationsFeature { relations }));
        self
    }

    pub fn build(self) -> Task {
        Task {
            id: self.id,
            features: self.features,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_task_builder() {
        let task = TaskBuilder::new("1".to_string())
            .with_metadata("title".to_string(), "desc".to_string())
            .with_status(false, true)
            .build();

        assert_eq!(task.id, "1");
        assert_eq!(task.title(), "title");
        assert_eq!(task.description(), "desc");
        assert!(!task.is_completed());
        assert!(task.is_important());
    }

    #[test]
    fn test_is_valid_range() {
        let now = Utc::now();
        let task = TaskBuilder::new("1".to_string())
            .with_schedule(Some(now), Some(now + Duration::days(1)))
            .build();
        assert!(task.is_valid_range());

        let task_invalid = TaskBuilder::new("2".to_string())
            .with_schedule(Some(now), Some(now - Duration::days(1)))
            .build();
        assert!(!task_invalid.is_valid_range());
    }
}
