mod adapters;
mod domain;
mod ports;

use clap::{arg, command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

use crate::adapters::storage::sqlite::SqliteRepository;
use crate::adapters::tui::app::{App, CurrentScreen, InputFocus};
use crate::adapters::tui::ui::ui;
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

fn run_tui(task_service: Arc<dyn TaskServicePort>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(task_service);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = std::time::Duration::from_millis(50);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('q') | KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            return Ok(())
                        }
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('a') => {
                            app.current_screen = CurrentScreen::Adding;
                            app.input.clear();
                            app.start_date_input.clear();
                            app.end_date_input.clear();
                            app.input_focus = InputFocus::Content;
                        }
                        KeyCode::Char('e') => {
                            let items = app.get_filtered_items();
                            if let Some(i) = app.list_state.selected() {
                                if i < items.len() {
                                    app.current_screen = CurrentScreen::Editing;
                                    app.editing_id = Some(items[i].id.clone());
                                    app.input = items[i].content.clone();
                                    app.start_date_input = items[i]
                                        .start_date
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_default();
                                    app.end_date_input = items[i]
                                        .end_date
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_default();
                                    app.input_focus = InputFocus::Content;
                                }
                            }
                        }
                        KeyCode::Char('x') => {
                            if app.list_state.selected().is_some() {
                                app.current_screen = CurrentScreen::ConfirmingDelete;
                            }
                        }
                        KeyCode::Char('d') => {
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                                app.page_down();
                            } else if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.remove_selected();
                            } else if app.list_state.selected().is_some() {
                                app.current_screen = CurrentScreen::ConfirmingDelete;
                            }
                        }
                        KeyCode::Char('l')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            let _ = app.task_service.clear_completed_tasks();
                        }
                        KeyCode::Char('/') => {
                            app.current_screen = CurrentScreen::Searching;
                        }
                        KeyCode::Char('c') | KeyCode::Enter => {
                            app.toggle_completed();
                        }
                        KeyCode::Char('i') => {
                            app.toggle_important();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.move_task_down();
                            } else {
                                app.next();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.move_task_up();
                            } else {
                                app.previous();
                            }
                        }
                        KeyCode::Char('g') => {
                            app.move_to_top();
                        }
                        KeyCode::Char('G') => {
                            app.move_to_bottom();
                        }
                        KeyCode::Char('u')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.page_up();
                        }
                        KeyCode::Home => app.move_to_top(),
                        KeyCode::End => app.move_to_bottom(),
                        KeyCode::PageUp => app.page_up(),
                        KeyCode::PageDown => app.page_down(),
                        KeyCode::Esc => {
                            app.search_query.clear();
                        }
                        _ => {}
                    },
                    CurrentScreen::Adding => match key.code {
                        KeyCode::Tab => {
                            app.next_field();
                        }
                        KeyCode::BackTab => {
                            app.next_field();
                            app.next_field();
                        }
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                let start = app.parse_start_date();
                                let end = app.parse_end_date();
                                let _ = app.task_service.add_task(app.input.clone(), start, end);
                                app.current_screen = CurrentScreen::Main;
                                app.input.clear();
                                app.start_date_input.clear();
                                app.end_date_input.clear();
                                app.input_focus = InputFocus::Content;
                            }
                        }
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                            app.start_date_input.clear();
                            app.end_date_input.clear();
                            app.input_focus = InputFocus::Content;
                        }
                        KeyCode::Char('u')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            match app.input_focus {
                                InputFocus::Content => app.input.clear(),
                                InputFocus::StartDate => app.start_date_input.clear(),
                                InputFocus::EndDate => app.end_date_input.clear(),
                            }
                        }
                        KeyCode::Char('w')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            let target = match app.input_focus {
                                InputFocus::Content => &mut app.input,
                                InputFocus::StartDate => &mut app.start_date_input,
                                InputFocus::EndDate => &mut app.end_date_input,
                            };
                            let mut graphemes = target.graphemes(true).collect::<Vec<&str>>();
                            while let Some(last) = graphemes.last() {
                                if last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            while let Some(last) = graphemes.last() {
                                if !last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            *target = graphemes.concat();
                        }
                        KeyCode::Char(c) => match app.input_focus {
                            InputFocus::Content => app.input.push(c),
                            InputFocus::StartDate => app.start_date_input.push(c),
                            InputFocus::EndDate => app.end_date_input.push(c),
                        },
                        KeyCode::Backspace if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            let target = match app.input_focus {
                                InputFocus::Content => &mut app.input,
                                InputFocus::StartDate => &mut app.start_date_input,
                                InputFocus::EndDate => &mut app.end_date_input,
                            };
                            let mut graphemes = target.graphemes(true).collect::<Vec<&str>>();
                            while let Some(last) = graphemes.last() {
                                if last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            while let Some(last) = graphemes.last() {
                                if !last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            *target = graphemes.concat();
                        }
                        KeyCode::Backspace => {
                            let target = match app.input_focus {
                                InputFocus::Content => &mut app.input,
                                InputFocus::StartDate => &mut app.start_date_input,
                                InputFocus::EndDate => &mut app.end_date_input,
                            };
                            let mut graphemes = target.graphemes(true).collect::<Vec<&str>>();
                            graphemes.pop();
                            *target = graphemes.concat();
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                            app.start_date_input.clear();
                            app.end_date_input.clear();
                            app.input_focus = InputFocus::Content;
                        }
                        _ => {}
                    },
                    CurrentScreen::Editing => match key.code {
                        KeyCode::Tab => {
                            app.next_field();
                        }
                        KeyCode::BackTab => {
                            app.next_field();
                            app.next_field();
                        }
                        KeyCode::Enter => {
                            if let Some(id) = &app.editing_id {
                                let start = app.parse_start_date();
                                let end = app.parse_end_date();
                                let _ = app.task_service.update_task_content(
                                    id.clone(),
                                    app.input.clone(),
                                    start,
                                    end,
                                );
                                app.current_screen = CurrentScreen::Main;
                                app.input.clear();
                                app.start_date_input.clear();
                                app.end_date_input.clear();
                                app.editing_id = None;
                                app.input_focus = InputFocus::Content;
                            }
                        }
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                            app.start_date_input.clear();
                            app.end_date_input.clear();
                            app.editing_id = None;
                            app.input_focus = InputFocus::Content;
                        }
                        KeyCode::Char('u')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            match app.input_focus {
                                InputFocus::Content => app.input.clear(),
                                InputFocus::StartDate => app.start_date_input.clear(),
                                InputFocus::EndDate => app.end_date_input.clear(),
                            }
                        }
                        KeyCode::Char('w')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            let target = match app.input_focus {
                                InputFocus::Content => &mut app.input,
                                InputFocus::StartDate => &mut app.start_date_input,
                                InputFocus::EndDate => &mut app.end_date_input,
                            };
                            let mut graphemes = target.graphemes(true).collect::<Vec<&str>>();
                            while let Some(last) = graphemes.last() {
                                if last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            while let Some(last) = graphemes.last() {
                                if !last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            *target = graphemes.concat();
                        }
                        KeyCode::Char(c) => match app.input_focus {
                            InputFocus::Content => app.input.push(c),
                            InputFocus::StartDate => app.start_date_input.push(c),
                            InputFocus::EndDate => app.end_date_input.push(c),
                        },
                        KeyCode::Backspace if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            let target = match app.input_focus {
                                InputFocus::Content => &mut app.input,
                                InputFocus::StartDate => &mut app.start_date_input,
                                InputFocus::EndDate => &mut app.end_date_input,
                            };
                            let mut graphemes = target.graphemes(true).collect::<Vec<&str>>();
                            while let Some(last) = graphemes.last() {
                                if last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            while let Some(last) = graphemes.last() {
                                if !last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            *target = graphemes.concat();
                        }
                        KeyCode::Backspace => {
                            let target = match app.input_focus {
                                InputFocus::Content => &mut app.input,
                                InputFocus::StartDate => &mut app.start_date_input,
                                InputFocus::EndDate => &mut app.end_date_input,
                            };
                            let mut graphemes = target.graphemes(true).collect::<Vec<&str>>();
                            graphemes.pop();
                            *target = graphemes.concat();
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                            app.start_date_input.clear();
                            app.end_date_input.clear();
                            app.editing_id = None;
                            app.input_focus = InputFocus::Content;
                        }
                        _ => {}
                    },
                    CurrentScreen::Searching => match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.list_state.select(Some(0));
                        }
                        KeyCode::Backspace
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            let mut graphemes =
                                app.search_query.graphemes(true).collect::<Vec<&str>>();
                            while let Some(last) = graphemes.last() {
                                if last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            while let Some(last) = graphemes.last() {
                                if !last.chars().all(|c| c.is_whitespace()) {
                                    graphemes.pop();
                                } else {
                                    break;
                                }
                            }
                            app.search_query = graphemes.concat();
                            app.list_state.select(Some(0));
                        }
                        KeyCode::Backspace => {
                            let mut graphemes =
                                app.search_query.graphemes(true).collect::<Vec<&str>>();
                            graphemes.pop();
                            app.search_query = graphemes.concat();
                            app.list_state.select(Some(0));
                        }
                        _ => {}
                    },
                    CurrentScreen::ConfirmingDelete => match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            app.remove_selected();
                            app.current_screen = CurrentScreen::Main;
                        }
                        KeyCode::Char('n')
                        | KeyCode::Esc
                        | KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.current_screen = CurrentScreen::Main;
                        }
                        _ => {
                            app.current_screen = CurrentScreen::Main;
                        }
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = std::time::Instant::now();
        }
    }
}
