use crate::adapters::tui::app::{Action, App, CurrentScreen, InputFocus};
use crate::adapters::tui::config::Config;
use crate::adapters::tui::ui::ui;
use crate::ports::inbound::TaskServicePort;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, error, info};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

//TODO: Implement the command pattern for the behavior
pub fn run_tui(task_service: Arc<dyn TaskServicePort>) -> io::Result<()> {
    info!("Initializing TUI...");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = Config::load();
    let mut app = App::new(task_service);
    let res = run_app(&mut terminal, &mut app, &config);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(ref err) = res {
        error!("TUI run_app error: {:?}", err);
        println!("{:?}", err)
    }

    info!("TUI session ended.");
    Ok(())
}

fn handle_action(app: &mut App, action: Action, config: &Config) -> io::Result<bool> {
    debug!("Handling action: {:?}", action);
    match action {
        Action::Quit => {
            info!("User requested quit.");
            return Ok(true);
        }
        Action::Add => {
            app.current_screen = CurrentScreen::Adding;
            app.title_input.clear();
            app.description_input.clear();
            app.start_date_input.clear();
            app.end_date_input.clear();
            app.input_focus = InputFocus::Title;
            app.sync_selected_date();
        }
        Action::Edit => {
            let items = app.get_filtered_items();
            if let Some(i) = app.list_state.selected() {
                if i < items.len() {
                    app.current_screen = CurrentScreen::Editing;
                    app.editing_id = Some(items[i].id.clone());
                    app.title_input = items[i].title.clone();
                    app.description_input = items[i].description.clone();
                    app.start_date_input = items[i]
                        .start_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default();
                    app.end_date_input = items[i]
                        .end_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default();
                    app.input_focus = InputFocus::Title;
                    app.sync_selected_date();
                }
            }
        }
        Action::Delete => {
            if app.list_state.selected().is_some() {
                app.current_screen = CurrentScreen::ConfirmingDelete;
            }
        }
        Action::ConfirmDelete => {
            info!("User confirmed deletion of task.");
            app.remove_selected();
            app.current_screen = CurrentScreen::Main;
        }
        Action::ToggleCompleted => {
            app.toggle_completed();
        }
        Action::ToggleImportant => {
            app.toggle_important();
        }
        Action::ToggleGantt => {
            if app.current_screen == CurrentScreen::Gantt {
                app.current_screen = CurrentScreen::Main;
            } else {
                app.current_screen = CurrentScreen::Gantt;
            }
        }
        Action::ToggleHelp => {
            if app.current_screen == CurrentScreen::Help {
                app.current_screen = CurrentScreen::Main;
            } else {
                app.current_screen = CurrentScreen::Help;
            }
        }
        Action::MoveUp => {
            app.previous();
        }
        Action::MoveDown => {
            app.next();
        }
        Action::MoveTaskUp => {
            app.move_task_up();
        }
        Action::MoveTaskDown => {
            app.move_task_down();
        }
        Action::MoveToTop => {
            app.move_to_top();
        }
        Action::MoveToBottom => {
            app.move_to_bottom();
        }
        Action::PageUp => {
            app.page_up();
        }
        Action::PageDown => {
            app.page_down();
        }
        Action::Search => {
            app.current_screen = CurrentScreen::Searching;
        }
        Action::SyncJira => {
            if !config.jira.enabled
                || config.jira.domain.is_empty()
                || config.jira.email.is_empty()
                || config.jira.api_token.is_empty()
            {
                info!("Triggering Jira configuration screen");
                app.current_screen = CurrentScreen::JiraConfiguring;
                app.jira_domain_input = config.jira.domain.clone();
                app.jira_email_input = config.jira.email.clone();
                app.jira_api_token_input = config.jira.api_token.clone();
                app.jira_projects_input = config.jira.projects.join(", ");
                app.jira_labels_input = config.jira.labels.join(", ");
                app.input_focus = InputFocus::JiraDomain;
            } else {
                info!("Initiating Jira synchronization");
                if let Err(e) = app.task_service.sync_jira(config.jira.clone()) {
                    error!("Jira sync failed: {:?}", e);
                }
            }
        }
        Action::ClearCompleted => {
            info!("Clearing completed tasks.");
            let _ = app.task_service.clear_completed_tasks();
        }
        Action::Esc => match app.current_screen {
            CurrentScreen::Main => app.search_query.clear(),
            CurrentScreen::JiraConfiguring | CurrentScreen::Help => {
                app.current_screen = CurrentScreen::Main;
            }
            _ => {
                app.current_screen = CurrentScreen::Main;
                app.title_input.clear();
                app.description_input.clear();
                app.start_date_input.clear();
                app.end_date_input.clear();
                app.editing_id = None;
                app.input_focus = InputFocus::Title;
            }
        },
        Action::Enter => match app.current_screen {
            CurrentScreen::JiraConfiguring => {
                info!("Saving Jira configuration and syncing");
                let mut new_config = config.clone();
                new_config.jira.enabled = true;
                new_config.jira.domain = app.jira_domain_input.clone();
                new_config.jira.email = app.jira_email_input.clone();
                new_config.jira.api_token = app.jira_api_token_input.clone();
                new_config.jira.projects = app
                    .jira_projects_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                new_config.jira.labels = app
                    .jira_labels_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if let Ok(_) = new_config.save() {
                    info!("Jira configuration saved successfully");
                    if let Err(e) = app.task_service.sync_jira(new_config.jira.clone()) {
                        error!("Jira sync failed after save: {:?}", e);
                    }
                    app.current_screen = CurrentScreen::Main;
                } else {
                    error!("Failed to save Jira configuration");
                }
            }
            CurrentScreen::Adding => {
                if !app.title_input.is_empty() {
                    info!("Adding new task: {}", app.title_input);
                    let start = app.parse_start_date();
                    let end = app.parse_end_date();
                    if let Err(e) = app.task_service.add_task(
                        app.title_input.clone(),
                        app.description_input.clone(),
                        start,
                        end,
                    ) {
                        error!("Failed to add task: {:?}", e);
                    }
                    app.current_screen = CurrentScreen::Main;
                    app.title_input.clear();
                    app.description_input.clear();
                    app.start_date_input.clear();
                    app.end_date_input.clear();
                    app.input_focus = InputFocus::Title;
                }
            }
            CurrentScreen::Editing => {
                if let Some(id) = &app.editing_id {
                    info!("Updating task: {}", id);
                    let start = app.parse_start_date();
                    let end = app.parse_end_date();
                    if let Err(e) = app.task_service.update_task(
                        id.clone(),
                        app.title_input.clone(),
                        app.description_input.clone(),
                        start,
                        end,
                    ) {
                        error!("Failed to update task: {:?}", e);
                    }
                    app.current_screen = CurrentScreen::Main;
                    app.title_input.clear();
                    app.description_input.clear();
                    app.start_date_input.clear();
                    app.end_date_input.clear();
                    app.editing_id = None;
                    app.input_focus = InputFocus::Title;
                }
            }
            CurrentScreen::Main
            | CurrentScreen::Searching
            | CurrentScreen::ConfirmingDelete
            | CurrentScreen::Gantt
            | CurrentScreen::Help => {
                app.toggle_completed();
            }
        },
        Action::Tab => {
            app.next_field();
        }
        Action::BackTab => {
            app.next_field();
            app.next_field();
        }
        Action::MoveDateLeft => {
            app.move_date_left();
        }
        Action::MoveDateRight => {
            app.move_date_right();
        }
        Action::MoveDateUp => {
            app.move_date_up();
        }
        Action::MoveDateDown => {
            app.move_date_down();
        }
        Action::SelectDate => {
            app.select_date();
        }
        Action::Macro(actions) => {
            for sub_action in actions {
                if handle_action(app, sub_action, config)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config: &Config,
) -> io::Result<()> {
    let tick_rate = std::time::Duration::from_millis(50);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let is_typing_focus = (app.current_screen == CurrentScreen::Adding
                    || app.current_screen == CurrentScreen::Editing
                    || app.current_screen == CurrentScreen::JiraConfiguring)
                    && (app.input_focus == InputFocus::Title
                        || app.input_focus == InputFocus::Description
                        || app.input_focus == InputFocus::JiraDomain
                        || app.input_focus == InputFocus::JiraEmail
                        || app.input_focus == InputFocus::JiraToken
                        || app.input_focus == InputFocus::JiraProjects
                        || app.input_focus == InputFocus::JiraLabels);

                let mut action = config.get_action(&app.current_screen, &key);

                // Hardcoded fallback for navigation keys in case config is old/missing them
                if action.is_none() {
                    match key.code {
                        KeyCode::Char('h') if !is_typing_focus => action = Some(Action::ToggleHelp),
                        KeyCode::Tab => action = Some(Action::Tab),
                        KeyCode::BackTab => action = Some(Action::BackTab),
                        KeyCode::Enter => action = Some(Action::Enter),
                        KeyCode::Esc => action = Some(Action::Esc),
                        _ => {}
                    }
                }

                if is_typing_focus {
                    if let Some(ref a) = action {
                        match a {
                            Action::Tab | Action::BackTab | Action::Enter | Action::Esc => {}
                            _ => {
                                if let KeyCode::Char(_) = key.code {
                                    if key.modifiers.is_empty()
                                        || key.modifiers == event::KeyModifiers::SHIFT
                                    {
                                        action = None;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(a) = action {
                    if handle_action(app, a, config)? {
                        return Ok(());
                    }
                } else {
                    match app.current_screen {
                        CurrentScreen::Adding
                        | CurrentScreen::Editing
                        | CurrentScreen::JiraConfiguring => match app.input_focus {
                            InputFocus::Title => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.title_input.clear()
                                }
                                KeyCode::Char(c) => app.title_input.push(c),
                                KeyCode::Backspace => {
                                    app.title_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::Description => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.description_input.clear()
                                }
                                KeyCode::Char('w')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    let mut graphemes = app
                                        .description_input
                                        .graphemes(true)
                                        .collect::<Vec<&str>>();
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
                                    app.description_input = graphemes.concat();
                                }
                                KeyCode::Char(c) => app.description_input.push(c),
                                KeyCode::Backspace => {
                                    app.description_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::StartDate => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.start_date_input.clear()
                                }
                                KeyCode::Char(c) => app.start_date_input.push(c),
                                KeyCode::Backspace => {
                                    app.start_date_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::EndDate => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.end_date_input.clear()
                                }
                                KeyCode::Char(c) => app.end_date_input.push(c),
                                KeyCode::Backspace => {
                                    app.end_date_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::JiraDomain => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.jira_domain_input.clear()
                                }
                                KeyCode::Char(c) => app.jira_domain_input.push(c),
                                KeyCode::Backspace => {
                                    app.jira_domain_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::JiraEmail => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.jira_email_input.clear()
                                }
                                KeyCode::Char(c) => app.jira_email_input.push(c),
                                KeyCode::Backspace => {
                                    app.jira_email_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::JiraToken => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.jira_api_token_input.clear()
                                }
                                KeyCode::Char(c) => app.jira_api_token_input.push(c),
                                KeyCode::Backspace => {
                                    app.jira_api_token_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::JiraProjects => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.jira_projects_input.clear()
                                }
                                KeyCode::Char(c) => app.jira_projects_input.push(c),
                                KeyCode::Backspace => {
                                    app.jira_projects_input.pop();
                                }
                                _ => {}
                            },
                            InputFocus::JiraLabels => match key.code {
                                KeyCode::Char('u')
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.jira_labels_input.clear()
                                }
                                KeyCode::Char(c) => app.jira_labels_input.push(c),
                                KeyCode::Backspace => {
                                    app.jira_labels_input.pop();
                                }
                                _ => {}
                            },
                        },
                        CurrentScreen::Searching => match key.code {
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
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = std::time::Instant::now();
        }
    }
}
