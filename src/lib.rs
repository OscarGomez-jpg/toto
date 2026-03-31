//! # TOTO - A Hexagonal Terminal Task Manager
//! 
//! `toto` is a task management application built with a strict separation of concerns 
//! using Hexagonal Architecture (Ports and Adapters).
//! 
//! ## Architecture
//! 
//! - **Domain**: Core business logic and entities (`Task`). This layer has no external dependencies.
//! - **Ports**: Interfaces (`Traits`) that define how the core interacts with the outside world.
//!   - `TaskServicePort`: Inbound port for UI/CLI.
//!   - `TaskRepository`: Outbound port for persistence.
//! - **Adapters**: Implementation details.
//!   - `storage`: SQLite persistence.
//!   - `tui`: Ratatui-based user interface.
//!   - `jira`: External API integration.

pub mod adapters;
pub mod domain;
pub mod ports;
