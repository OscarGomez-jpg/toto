mod app;
mod list;

use chrono::Local;
use clap::{arg, command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

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
        let limit = matches
            .get_one::<String>("list")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(items.len());

        for item in items.iter().take(limit) {
            let status = if item.completed { "[X]" } else { "[ ]" };
            let important = if item.important { "!" } else { " " };
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
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                        }
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
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.input.clear();
                            app.editing_id = None;
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

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = std::time::Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let bg_color = Color::Rgb(20, 20, 25);
    let card_bg = Color::Rgb(30, 30, 35);
    let primary_text = Color::Rgb(224, 224, 224);
    let dim_text = Color::Rgb(120, 120, 130);
    let accent_blue = Color::Rgb(0, 153, 255);
    let alert_red = Color::Rgb(255, 82, 82);

    let main_block = Block::default().style(Style::default().bg(bg_color).fg(primary_text));
    f.render_widget(main_block, f.area());

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // --- HEADER ---
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(outer_layout[0]);

    let ascii_logo = vec![
        Line::from(vec![Span::styled(
            " ▟████▙",
            Style::default().fg(accent_blue),
        )]),
        Line::from(vec![Span::styled(
            " ▝▘ ▟█▘ ",
            Style::default().fg(accent_blue),
        )]),
        Line::from(vec![
            Span::styled("   ▟█▘  ", Style::default().fg(accent_blue)),
            Span::styled(
                " T O T O",
                Style::default()
                    .fg(primary_text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ▟█▘   ", Style::default().fg(accent_blue)),
            Span::styled(" [ v2.2 ]", Style::default().fg(dim_text)),
        ]),
    ];
    f.render_widget(Paragraph::new(ascii_logo), header_chunks[0]);

    let search_bar = Paragraph::new(format!("  Search: {}", app.search_query))
        .style(Style::default().fg(primary_text))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(
                    if app.current_screen == CurrentScreen::Searching {
                        accent_blue
                    } else {
                        dim_text
                    },
                )),
        );
    f.render_widget(search_bar, header_chunks[1]);

    if app.current_screen == CurrentScreen::Searching {
        f.set_cursor_position((
            header_chunks[1].x + app.search_query.len() as u16 + 10,
            header_chunks[1].y,
        ));
    }

    // --- MAIN CONTENT ---
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(25)])
        .split(outer_layout[1]);

    let items: Vec<ListItem> = app
        .get_filtered_items()
        .iter()
        .map(|item| {
            let status_icon = if item.completed { "⬢" } else { "⬡" };
            let important_marker = if item.important { "!" } else { " " };
            let mut text_style = Style::default().fg(primary_text);
            let mut icon_style = Style::default().fg(accent_blue);
            if item.completed {
                text_style = text_style.fg(dim_text).add_modifier(Modifier::DIM);
                icon_style = icon_style.fg(dim_text);
            }
            let content = Line::from(vec![
                Span::styled(format!("{:<2} ", item.id), Style::default().fg(dim_text)),
                Span::styled(format!("{} ", status_icon), icon_style),
                Span::styled(
                    format!("{} ", important_marker),
                    if item.important {
                        Style::default().fg(alert_red)
                    } else {
                        Style::default().fg(bg_color)
                    },
                ),
                Span::styled(format!("{}", item.content), text_style),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(dim_text))
                .title(" Tasks ")
                .title_style(Style::default().fg(dim_text)),
        )
        .highlight_style(Style::default().bg(card_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");

    f.render_stateful_widget(list, content_layout[0], &mut app.list_state);

    // Sidebar Stats (Left Aligned for labels)
    let total = app.todo_list.get_all().len();
    let completed = app
        .todo_list
        .get_all()
        .iter()
        .filter(|t| t.completed)
        .count();
    let now = Local::now();

    let stats_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " PROGRESS",
            Style::default().fg(dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Total      ", Style::default().fg(dim_text)),
            Span::styled(total.to_string(), Style::default().fg(primary_text)),
        ]),
        Line::from(vec![
            Span::styled(" Done       ", Style::default().fg(dim_text)),
            Span::styled(completed.to_string(), Style::default().fg(primary_text)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(" STATUS", Style::default().fg(dim_text))]),
        Line::from(vec![
            Span::styled(" Date       ", Style::default().fg(dim_text)),
            Span::styled(
                now.format("%Y-%m-%d").to_string(),
                Style::default().fg(primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Time       ", Style::default().fg(dim_text)),
            Span::styled(
                now.format("%H:%M:%S").to_string(),
                Style::default().fg(accent_blue),
            ),
        ]),
        Line::from(vec![
            Span::styled(" OS         ", Style::default().fg(dim_text)),
            Span::styled(
                std::env::consts::OS.to_uppercase(),
                Style::default().fg(primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" DB         ", Style::default().fg(dim_text)),
            Span::styled("SQLITE", Style::default().fg(primary_text)),
            Span::styled(
                if (app.ticks / 10) % 2 == 0 {
                    " ●"
                } else {
                    " ○"
                },
                Style::default().fg(accent_blue),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " METRICS",
            Style::default().fg(dim_text),
        )]),
        Line::from(vec![
            Span::styled(" Latency    ", Style::default().fg(dim_text)),
            Span::styled(
                format!("{}ms", (app.ticks % 15) + 10),
                Style::default().fg(accent_blue),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Entropy    ", Style::default().fg(dim_text)),
            Span::styled(
                format!("0x{:04X}", app.ticks % 0xFFFF),
                Style::default().fg(primary_text),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Kernel     ", Style::default().fg(dim_text)),
            Span::styled("L-042", Style::default().fg(primary_text)),
        ]),
    ];
    let stats = Paragraph::new(stats_text);
    f.render_widget(stats, content_layout[1]);

    // --- FOOTER ---
    let help_message = match app.current_screen {
        CurrentScreen::Main => "a:add  e:edit  c:done  i:important  d:delete  /:search  q:quit",
        CurrentScreen::Adding => "enter:save  esc:cancel",
        CurrentScreen::Editing => "enter:update  esc:cancel",
        CurrentScreen::Searching => "enter:done  esc:reset",
        CurrentScreen::ConfirmingDelete => "Confirm delete? (y/n)",
    };
    let footer = Paragraph::new(help_message)
        .style(Style::default().fg(dim_text))
        .alignment(Alignment::Center);
    f.render_widget(footer, outer_layout[2]);

    // Popups
    if app.current_screen == CurrentScreen::Adding || app.current_screen == CurrentScreen::Editing {
        let area = centered_rect(50, 15, f.area());
        f.render_widget(Clear, area);
        let title = if app.current_screen == CurrentScreen::Adding {
            " Add Task "
        } else {
            " Edit Task "
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent_blue))
            .bg(card_bg)
            .title(title)
            .title_style(Style::default().fg(accent_blue));
        let input = Paragraph::new(format!("\n  > {}", app.input))
            .style(Style::default().fg(primary_text))
            .block(input_block);
        f.render_widget(input, area);
        f.set_cursor_position((area.x + app.input.len() as u16 + 5, area.y + 2));
    } else if app.current_screen == CurrentScreen::ConfirmingDelete {
        let area = centered_rect(40, 20, f.area());
        f.render_widget(Clear, area);
        let confirm = Paragraph::new("\nConfirm deletion?\n\n(y) Yes / (n) No")
            .style(Style::default().fg(alert_red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(alert_red))
                    .bg(card_bg)
                    .title(" Warning "),
            )
            .alignment(Alignment::Center);
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
