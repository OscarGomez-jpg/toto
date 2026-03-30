mod adapters;
mod domain;
mod ports;

use clap::{arg, command};
use std::sync::Arc;

use crate::adapters::storage::sqlite::SqliteRepository;
use crate::adapters::tui::runner::run_tui;
use crate::domain::service::TaskService;
use crate::ports::inbound::TaskServicePort;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .arg(arg!(-a --add <CONTENT> "Add a new todo").required(false))
        .arg(arg!(-l --list [LIMIT] "List todos").required(false))
        .arg(arg!(-r --remove <ID> "Remove a todo by ID").required(false))
        .arg(arg!(-c --done <ID> "Toggle completion status of a todo").required(false))
        .arg(arg!(-i --important <ID> "Toggle importance of a todo").required(false))
        .arg(arg!(-e --edit <ID_CONTENT> "Edit a todo (format: 'ID:New Content')").required(false))
        .arg(arg!(--start <DATE> "Start date (YYYY-MM-DD)").required(false))
        .arg(arg!(--end <DATE> "End date (YYYY-MM-DD)").required(false))
        .arg(
            arg!(--clear "Clear all completed todos")
                .required(false)
                .num_args(0),
        )
        .get_matches();

    let repository = Arc::new(SqliteRepository::new()?);
    let task_service: Arc<dyn TaskServicePort> = Arc::new(TaskService::new(repository));

    let mut performed_action = false;

    let start_date = matches.get_one::<String>("start").and_then(|s| {
        let full = if s.len() == 10 {
            format!("{}T00:00:00Z", s)
        } else {
            s.clone()
        };
        chrono::DateTime::parse_from_rfc3339(&full)
            .ok()
            .map(|d| d.with_timezone(&chrono::Utc))
    });

    let end_date = matches.get_one::<String>("end").and_then(|s| {
        let full = if s.len() == 10 {
            format!("{}T23:59:59Z", s)
        } else {
            s.clone()
        };
        chrono::DateTime::parse_from_rfc3339(&full)
            .ok()
            .map(|d| d.with_timezone(&chrono::Utc))
    });

    if let Some(content) = matches.get_one::<String>("add") {
        task_service.add_task(content.to_owned(), start_date, end_date)?;
        println!("Added: {}", content);
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("remove") {
        let msg = task_service.remove_task(id_str.to_owned())?;
        println!("{}", msg);
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("done") {
        task_service.toggle_completed(id_str.to_owned())?;
        println!("Toggled completion for task {}", id_str);
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("important") {
        task_service.toggle_important(id_str.to_owned())?;
        println!("Toggled importance for task {}", id_str);
        performed_action = true;
    }

    if let Some(id_content) = matches.get_one::<String>("edit") {
        if let Some((id_str, content)) = id_content.split_once(':') {
            task_service.update_task_content(
                id_str.trim().to_string(),
                content.trim().to_string(),
                start_date,
                end_date,
            )?;
            println!("Updated task {} content", id_str);
            performed_action = true;
        } else {
            println!("Invalid edit format. Use 'ID:New Content'");
            return Ok(());
        }
    }

    if matches.get_flag("clear") {
        let msg = task_service.clear_completed_tasks()?;
        println!("{}", msg);
        performed_action = true;
    }

    if matches.contains_id("list") {
        let items = task_service.get_all_tasks()?;
        let limit = matches
            .get_one::<String>("list")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(items.len());

        for item in items.iter().take(limit) {
            let status = if item.completed { "[X]" } else { "[ ]" };
            let important = if item.important { "!" } else { " " };
            let short_id = if item.id.len() > 4 {
                &item.id[..4]
            } else {
                &item.id
            };
            println!("{} {} {}: {}", important, status, short_id, item.content);
        }
        performed_action = true;
    }

    if performed_action {
        return Ok(());
    }

    run_tui(task_service)?;
    Ok(())
}
