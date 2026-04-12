use clap::{arg, command};
use directories::ProjectDirs;
use log::{info, LevelFilter};
use simplelog::{Config as LogConfig, WriteLogger};
use std::fs::File;
use std::sync::Arc;

use toto::adapters::storage::sqlite::SqliteRepository;
use toto::adapters::tui::runner::run_tui;
use toto::domain::service::TaskService;
use toto::ports::inbound::TaskServicePort;
use toto::domain::command::{AddTaskCommand, RemoveTaskCommand, ToggleCompletedCommand, ToggleImportantCommand, UpdateTaskCommand, ClearCompletedCommand};

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "toto") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;
        let log_path = data_dir.join("toto.log");
        let file = File::create(log_path)?;
        WriteLogger::init(LevelFilter::Info, LogConfig::default(), file)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = init_logger();
    info!("Starting toto...");

    let matches = command!()
        .arg(arg!(-a --add <TITLE> "Add a new todo title").required(false))
        .arg(arg!(-d --desc <DESCRIPTION> "Task description").required(false))
        .arg(arg!(-l --list [LIMIT] "List todos").required(false))
        .arg(arg!(-r --remove <ID> "Remove a todo by ID").required(false))
        .arg(arg!(-c --done <ID> "Toggle completion status of a todo").required(false))
        .arg(arg!(-i --important <ID> "Toggle importance of a todo").required(false))
        .arg(
            arg!(-e --edit <ID_TITLE> "Edit a todo title (format: 'ID:New Title')").required(false),
        )
        .arg(arg!(--start <DATE> "Start date (YYYY-MM-DD)").required(false))
        .arg(arg!(--end <DATE> "End date (YYYY-MM-DD)").required(false))
        .arg(
            arg!(--clear "Clear all completed todos")
                .required(false)
                .num_args(0),
        )
        .arg(
            arg!(--"reset-config" "Reset configuration to defaults")
                .required(false)
                .num_args(0),
        )
        .get_matches();

    // 1. Initialize Adapters (Infrastructure)
    let repository = Arc::new(SqliteRepository::new()?);

    // 2. Initialize Core (Domain) with Ports
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

    if let Some(title) = matches.get_one::<String>("add") {
        let description = matches
            .get_one::<String>("desc")
            .cloned()
            .unwrap_or_default();
        let cmd = Box::new(AddTaskCommand {
            title: title.to_owned(),
            description,
            start_date,
            end_date,
        });
        task_service.execute_command(cmd)?;
        println!("Added: {}", title);
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("remove") {
        let cmd = Box::new(RemoveTaskCommand { id: id_str.to_owned() });
        let res = task_service.execute_command(cmd)?;
        if let toto::domain::command::CommandResult::Id(msg) = res {
            println!("{}", msg);
        }
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("done") {
        let cmd = Box::new(ToggleCompletedCommand { id: id_str.to_owned() });
        task_service.execute_command(cmd)?;
        println!("Toggled completion for task {}", id_str);
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("important") {
        let cmd = Box::new(ToggleImportantCommand { id: id_str.to_owned() });
        task_service.execute_command(cmd)?;
        println!("Toggled importance for task {}", id_str);
        performed_action = true;
    }

    if let Some(id_title) = matches.get_one::<String>("edit") {
        if let Some((id_str, title)) = id_title.split_once(':') {
            let description = matches
                .get_one::<String>("desc")
                .cloned()
                .unwrap_or_default();
            let cmd = Box::new(UpdateTaskCommand {
                id: id_str.trim().to_string(),
                title: title.trim().to_string(),
                description,
                start_date,
                end_date,
            });
            task_service.execute_command(cmd)?;
            println!("Updated task {} title", id_str);
            performed_action = true;
        } else {
            println!("Invalid edit format. Use 'ID:New Title'");
            return Ok(());
        }
    }

    if matches.get_flag("clear") {
        let cmd = Box::new(ClearCompletedCommand);
        let res = task_service.execute_command(cmd)?;
        if let toto::domain::command::CommandResult::Id(msg) = res {
            println!("{}", msg);
        }
        performed_action = true;
    }

    if matches.get_flag("reset-config") {
        let config = toto::adapters::tui::config::Config::default();
        config.save()?;
        println!("Configuration reset to defaults.");
        performed_action = true;
    }

    if matches.contains_id("list") {
        let items = task_service.get_all_tasks()?;
        let limit = matches
            .get_one::<String>("list")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(items.len());

        for item in items.iter().take(limit) {
            let status = if item.is_completed() { "[X]" } else { "[ ]" };
            let important = if item.is_important() { "!" } else { " " };
            let short_id = if item.id.len() > 4 {
                &item.id[..4]
            } else {
                &item.id
            };
            println!(
                "{} {} {}: {} - {}",
                important,
                status,
                short_id,
                item.title(),
                item.description()
            );
        }
        performed_action = true;
    }

    if performed_action {
        return Ok(());
    }

    run_tui(task_service)?;
    Ok(())
}
