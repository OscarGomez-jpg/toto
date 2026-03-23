mod app;
mod list;

use std::io;
use clap::{arg, command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::app::{App, CurrentScreen};
use crate::list::TodoList;

fn main() -> io::Result<()> {
    let matches = command!()
        .arg(arg!(-a --add <CONTENT> "Add a new todo").required(false))
        .arg(arg!(-l --list [LIMIT] "List todos").required(false))
        .arg(arg!(-r --remove <ID> "Remove a todo by ID").required(false))
        .arg(arg!(-c --done <ID> "Toggle completion status of a todo").required(false))
        .arg(arg!(-i --important <ID> "Toggle importance of a todo").required(false))
        .arg(arg!(-e --edit <ID_CONTENT> "Edit a todo (format: 'ID:New Content')").required(false))
        .get_matches();

    let mut todo = TodoList::load();
    let mut performed_action = false;

    if let Some(content) = matches.get_one::<String>("add") {
        todo.add_line(content.to_owned());
        println!("Added: {}", content);
        performed_action = true;
    }

    if let Some(id_str) = matches.get_one::<String>("remove") {
        if let Ok(id) = id_str.parse::<i64>() {
            let msg = todo.remove(id);
            println!("{}", msg);
            performed_action = true;
        } else {
            println!("Invalid ID for remove: {}", id_str);
            return Ok(());
        }
    }

    if let Some(id_str) = matches.get_one::<String>("done") {
        if let Ok(id) = id_str.parse::<i64>() {
            todo.toggle_completed(id);
            println!("Toggled completion for task {}", id);
            performed_action = true;
        } else {
            println!("Invalid ID for done: {}", id_str);
            return Ok(());
        }
    }

    if let Some(id_str) = matches.get_one::<String>("important") {
        if let Ok(id) = id_str.parse::<i64>() {
            todo.toggle_important(id);
            println!("Toggled importance for task {}", id);
            performed_action = true;
        } else {
            println!("Invalid ID for important: {}", id_str);
            return Ok(());
        }
    }

    if let Some(id_content) = matches.get_one::<String>("edit") {
        if let Some((id_str, content)) = id_content.split_once(':') {
            if let Ok(id) = id_str.trim().parse::<i64>() {
                todo.update_content(id, content.trim().to_string());
                println!("Updated task {} content", id);
                performed_action = true;
            } else {
                println!("Invalid ID for edit: {}", id_str);
                return Ok(());
            }
        } else {
            println!("Invalid edit format. Use 'ID:New Content'");
            return Ok(());
        }
    }

    if matches.contains_id("list") {
        let items = todo.get_all();
        let limit = matches.get_one::<String>("list")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(items.len());
        
        for item in items.iter().take(limit) {
            let status = if item.completed { "[X]" } else { "[ ]" };
            let important = if item.important { "⭐" } else { "  " };
            println!("{} {} {}: {}", important, status, item.id, item.content);
        }
        performed_action = true;
    }

    if performed_action {
        return Ok(());
    }

    run_tui()
}

fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => {
                        app.current_screen = CurrentScreen::Adding;
                        app.input.clear();
                    }
                    KeyCode::Char('e') => {
                        let items = app.get_filtered_items();
                        if let Some(i) = app.list_state.selected() {
                            if i < items.len() {
                                app.current_screen = CurrentScreen::Editing;
                                app.editing_id = Some(items[i].id);
                                app.input = items[i].content.clone();
                            }
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('x') => {
                        if app.list_state.selected().is_some() {
                            app.current_screen = CurrentScreen::ConfirmingDelete;
                        }
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
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Esc => {
                        app.search_query.clear();
                    }
                    _ => {}
                },
                CurrentScreen::Adding => match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.todo_list.add_line(app.input.clone());
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); },
                    KeyCode::Esc => { app.current_screen = CurrentScreen::Main; },
                    _ => {}
                },
                CurrentScreen::Editing => match key.code {
                    KeyCode::Enter => {
                        if let Some(id) = app.editing_id {
                            app.todo_list.update_content(id, app.input.clone());
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                            app.editing_id = None;
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); },
                    KeyCode::Esc => { 
                        app.current_screen = CurrentScreen::Main;
                        app.input.clear();
                        app.editing_id = None;
                    },
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
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.list_state.select(Some(0));
                    }
                    _ => {}
                },
                CurrentScreen::ConfirmingDelete => match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        app.remove_selected();
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {
                        app.current_screen = CurrentScreen::Main;
                    }
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // List
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Search bar
    let search_title = if app.current_screen == CurrentScreen::Searching { " Searching (Type to filter) " } else { " Search (Press / to filter) " };
    let search_bar = Paragraph::new(app.search_query.as_str())
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search_bar, chunks[0]);
    if app.current_screen == CurrentScreen::Searching {
        f.set_cursor_position((chunks[0].x + app.search_query.len() as u16 + 1, chunks[0].y + 1));
    }

    // List
    let items: Vec<ListItem> = app.get_filtered_items().iter().map(|item| {
        let status = if item.completed { " [X] " } else { " [ ] " };
        let important = if item.important { " ⭐ " } else { "    " };
        
        let mut style = Style::default();
        if item.completed {
            style = style.fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT);
        }
        if item.important {
            style = style.fg(Color::Yellow);
        }

        let content = Line::from(vec![
            Span::styled(important, Style::default().fg(Color::Yellow)),
            Span::styled(status, if item.completed { Style::default().fg(Color::Green) } else { Style::default() }),
            Span::styled(item.content.clone(), style),
        ]);
        ListItem::new(content)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Todo List "))
        .highlight_style(Style::default().bg(Color::Blue))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Help
    let help_message = match app.current_screen {
        CurrentScreen::Main => "q:quit | a:add | e:edit | d:del | c:done | i:imp | /:search",
        CurrentScreen::Adding => "Enter:save | Esc:cancel",
        CurrentScreen::Editing => "Enter:save | Esc:cancel",
        CurrentScreen::Searching => "Enter:finish | Backspace:del",
        CurrentScreen::ConfirmingDelete => "Confirm delete? (y/n)",
    };
    let help = Paragraph::new(help_message).block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help, chunks[2]);

    // Popups
    if app.current_screen == CurrentScreen::Adding || app.current_screen == CurrentScreen::Editing {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);
        let title = if app.current_screen == CurrentScreen::Adding { " Add New Task " } else { " Edit Task " };
        let input = Paragraph::new(app.input.as_str())
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(input, area);
        f.set_cursor_position((area.x + app.input.len() as u16 + 1, area.y + 1));
    } else if app.current_screen == CurrentScreen::ConfirmingDelete {
        let area = centered_rect(40, 20, f.area());
        f.render_widget(Clear, area);
        let confirm = Paragraph::new("\nAre you sure you want to delete this task?\n\n(y)es / (n)o")
            .block(Block::default().borders(Borders::ALL).title(" Confirm Delete "))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(confirm, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
