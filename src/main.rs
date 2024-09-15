mod app;
mod task;
mod ui;

use app::TodoApp;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    widgets::ListState,
    Terminal,
};
use ui::InputMode;
use crate::ui::ui;

fn get_todo_file_path() -> PathBuf {
    let home_dir = env::var("HOME").expect("Unable to get $HOME directory");
    let todo_file = Path::new(&home_dir).join("todo.json");

    // If the file doesn't exist, create an empty file
    if !todo_file.exists() {
        File::create(&todo_file).expect("Unable to create todo file");
    }

    todo_file
}

fn main() -> Result<(), io::Error> {
    let todo_file_path = get_todo_file_path();
    let mut app = TodoApp::load_from_file(&todo_file_path).unwrap_or_else(|_| TodoApp::new());
    app.reorder_tasks();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_index = 0;
    let mut filter = String::new();
    let mut input = String::new();
    let mut input_mode = InputMode::View;
    let mut status_message: Option<String> = None; // Temporary status message
    let mut message_time: Option<Instant> = None; // Time when message is shown
    let mut reset_dialog = false;
    let mut list_state = ListState::default();
    list_state.select(Some(current_index));

    loop {
        // Check if the status message should be cleared after 3 seconds
        if let Some(time) = message_time {
            if time.elapsed() > Duration::from_secs(3) {
                status_message = None; // Clear the message after 3 seconds
                message_time = None; // Reset the timer
            }
        }
        terminal.draw(|f| {
            ui(
                f,
                &app,
                &mut list_state,
                &filter,
                &input,
                &input_mode,
                &status_message,
            );
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, &input_mode) {
                    (KeyCode::Char('q'), InputMode::View) => {
                        // Clear the terminal and exit
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    (KeyCode::Char('b'), InputMode::View) => {
                        let current_date = Local::now().format("%Y-%m-%d").to_string();
                        let backup_file_name = format!("todo.{}.json", current_date);
                        let backup_file_path = todo_file_path.with_file_name(backup_file_name);

                        if fs::copy(&todo_file_path, backup_file_path).is_ok() {
                            status_message = Some("Backup created successfully!".to_string());
                        } else {
                            status_message = Some("Backup failed.".to_string());
                        }
                        message_time = Some(Instant::now()); // Start the 3-second timer
                    }
                    (KeyCode::Char('r'), InputMode::View) => {
                        reset_dialog = true;
                        status_message =
                            Some("Press 'y' to confirm reset, 'n' to cancel.".to_string());
                        message_time = Some(Instant::now()); // Show status message
                    }
                    (KeyCode::Char('j'), InputMode::View) => {
                        let tasks_filtered_len = app.filter_tasks(&filter).len();
                        if current_index + 1 < tasks_filtered_len {
                            current_index += 1;
                            list_state.select(Some(current_index));
                        }
                    }
                    (KeyCode::Char('k'), InputMode::View) => {
                        if current_index > 0 {
                            current_index -= 1;
                            list_state.select(Some(current_index));
                        }
                    }
                    (KeyCode::Char(' '), InputMode::View) => {
                        let tasks_filtered = app.filter_tasks(&filter);
                        if let Some(task) = tasks_filtered.get(current_index) {
                            let original_index = app
                                .tasks
                                .iter()
                                .position(|t| t.description == task.description)
                                .unwrap();
                            app.toggle_task(original_index);
                            app.save_to_file(&todo_file_path);
                        }
                    }
                    (KeyCode::Char('-'), InputMode::View) => {
                        let tasks_filtered = app.filter_tasks(&filter);
                        if let Some(task) = tasks_filtered.get(current_index) {
                            let original_index = app
                                .tasks
                                .iter()
                                .position(|t| t.description == task.description)
                                .unwrap();
                            app.toggle_pending(original_index);
                            let _ = app.save_to_file(&todo_file_path);
                        }
                    }
                    (KeyCode::Char('o'), InputMode::View) => {
                        input_mode = InputMode::Add;
                        input.clear();
                    }
                    (KeyCode::Char('d'), InputMode::View) => {
                        let tasks_filtered = app.filter_tasks(&filter);
                        if let Some(task) = tasks_filtered.get(current_index) {
                            let original_index = app
                                .tasks
                                .iter()
                                .position(|t| t.description == task.description)
                                .unwrap();
                            app.delete_task(original_index);
                            let _ = app.save_to_file(&todo_file_path).unwrap();
                            status_message = Some("Task deleted.".to_string());
                            message_time = Some(Instant::now());
                            if current_index >= tasks_filtered.len() - 1 && current_index > 0 {
                                current_index -= 1;
                            }
                            list_state.select(Some(current_index));
                        }
                    }
                    (KeyCode::Char('D'), InputMode::View) => {
                        app.remove_done_tasks();
                        let _ = app.save_to_file(&todo_file_path).unwrap();
                        status_message = Some("Completed tasks removed.".to_string());
                        message_time = Some(Instant::now());
                        current_index = 0;
                        list_state.select(Some(current_index));
                    }
                    (KeyCode::Char('/'), InputMode::View) => {
                        input_mode = InputMode::Filter;
                        input.clear();
                    }
                    (KeyCode::Char('i'), InputMode::View) => {
                        input_mode = InputMode::Edit;
                        input.clear();
                        if let Some(task) = app.filter_tasks(&filter).get(current_index) {
                            input = task.description.clone();
                        }
                    }
                    (KeyCode::Enter, InputMode::Add) => {
                        let tasks_filtered = app.filter_tasks(&filter);
                        let (current_status, current_index) =
                            if let Some(current_task) = tasks_filtered.get(current_index) {
                                let original_index = app
                                    .tasks
                                    .iter()
                                    .position(|t| t.description == current_task.description)
                                    .unwrap();
                                (Some(current_task.status.clone()), Some(original_index))
                            } else {
                                (None, None)
                            };

                        app.add_task(input.clone(), current_status, current_index);
                        app.reorder_tasks();
                        let _ = app.save_to_file(&todo_file_path).unwrap();
                        input_mode = InputMode::View;
                        input.clear();
                    }
                    (KeyCode::Enter, InputMode::Edit) => {
                        let tasks_filtered = app.filter_tasks(&filter);
                        if let Some(task) = tasks_filtered.get(current_index) {
                            let original_index = app
                                .tasks
                                .iter()
                                .position(|t| t.description == task.description)
                                .unwrap();
                            app.edit_task(original_index, input.clone());
                            let _ = app.save_to_file(&todo_file_path);
                            input_mode = InputMode::View;
                        }
                        input.clear();
                    }
                    (KeyCode::Enter, InputMode::Filter) => {
                        filter = input.clone();
                        input_mode = InputMode::View;
                    }
                    (KeyCode::Char(c), InputMode::Add | InputMode::Filter | InputMode::Edit) => {
                        input.push(c);
                    }
                    (KeyCode::Backspace, InputMode::Add | InputMode::Filter | InputMode::Edit) => {
                        input.pop();
                    }
                    (KeyCode::Esc, _) => {
                        input.clear();
                        input_mode = InputMode::View;
                    }
                    (KeyCode::Char('c'), _) if key.modifiers == KeyModifiers::CONTROL => {
                        input_mode = InputMode::View;
                        input.clear();
                    }
                    _ => {}
                }
                if reset_dialog {
                    match key.code {
                        KeyCode::Char('y') => {
                            let current_date = Local::now().format("%Y-%m-%d").to_string();
                            let backup_file_name = format!("todo.{}.json", current_date);
                            let backup_file_path = todo_file_path.with_file_name(backup_file_name);
                            if fs::copy(&todo_file_path, backup_file_path).is_ok() {
                                fs::write(&todo_file_path, "[]")
                                    .expect("Unable to clear todo file");
                                status_message =
                                    Some("Backup created and todo list reset.".to_string());
                            } else {
                                status_message = Some("Backup failed. Reset canceled.".to_string());
                            }

                            message_time = Some(Instant::now());
                            reset_dialog = false;
                        }
                        KeyCode::Char('n') => {
                            // Cancel the reset process
                            status_message = Some("Reset canceled.".to_string());
                            message_time = Some(Instant::now());
                            reset_dialog = false;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
