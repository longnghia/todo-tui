use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use std::env;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    path::PathBuf,
    time::Duration,
    time::Instant,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    description: String,
    done: bool,
    created_at: Option<DateTime<Local>>,
}

#[derive(Serialize, Deserialize)]
struct TodoApp {
    tasks: Vec<Task>,
}

impl TodoApp {
    fn new() -> TodoApp {
        TodoApp { tasks: vec![] }
    }

    fn load_from_file(filename: &Path) -> io::Result<TodoApp> {
        if filename.exists() {
            let content = fs::read_to_string(filename)?;
            let tasks = serde_json::from_str(&content)?;
            Ok(tasks)
        } else {
            Ok(TodoApp::new())
        }
    }

    fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    fn add_task(&mut self, description: String) {
        let task = Task {
            description,
            done: false,
            created_at: Some(Local::now()),
        };
        let index = self
            .tasks
            .iter()
            .position(|t| t.done)
            .unwrap_or(self.tasks.len());
        self.tasks.insert(index, task);
    }

    fn edit_task(&mut self, index: usize, new_description: String) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.description = new_description;
        }
    }

    fn toggle_task(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.done = !task.done;
            self.reorder_tasks();
        }
    }

    fn reorder_tasks(&mut self) {
        self.tasks.sort_by_key(|t| t.done);
    }

    fn filter_tasks(&self, query: &str) -> Vec<Task> {
        self.tasks
            .iter()
            .filter(|task| task.description.contains(query))
            .cloned()
            .collect()
    }
}

enum InputMode {
    View,
    Add,
    Edit,
    Filter,
}

fn ui<B: Backend>(
    f: &mut Frame<B>,
    app: &TodoApp,
    state: &mut ListState,
    filter: &str,
    input: &str,
    input_mode: &InputMode,
    status_message: &Option<String>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(5),    // Area for tasks list
                Constraint::Length(3), // Area for input at the bottom
                Constraint::Length(1), // Status message at the bottom
            ]
            .as_ref(),
        )
        .split(f.size());

    // Render tasks list
    let tasks: Vec<ListItem> = app
        .filter_tasks(filter)
        .iter()
        .map(|task| {
            let style = if task.done {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::Black)
            };
            let content = Spans::from(vec![Span::styled(
                format!(
                    "{} {}",
                    if task.done { "[x]" } else { "[ ]" },
                    task.description
                ),
                style,
            )]);
            ListItem::new(content)
        })
        .collect();

    let tasks_list = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title("Todo List"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(tasks_list, chunks[0], state);

    // Render input box at the bottom for adding a new task, editing, or filtering
    let input_text = match input_mode {
        InputMode::Add => format!("New Task: {}", input),
        InputMode::Filter => format!("Filter: {}", input),
        InputMode::Edit => format!("Edit Task: {}", input),
        InputMode::View => "".to_string(),
    };

    let input_box = Paragraph::new(input_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input_box, chunks[1]);

    // Render the status message if it exists
    if let Some(message) = status_message {
        let status_widget = Paragraph::new(message.as_ref())
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("Status"));

        f.render_widget(status_widget, chunks[2]); // Render status message at the bottom
    } else {
        // Render an empty status message area when there is no message
        let empty_status =
            Paragraph::new("").block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(empty_status, chunks[2]);
    }
}

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
                match key.code {
                    KeyCode::Char('q') => {
                        // Clear the terminal and exit
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    KeyCode::Char('b') => {
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
                    KeyCode::Char('j') if matches!(input_mode, InputMode::View) => {
                        let tasks_filtered_len = app.filter_tasks(&filter).len();
                        if current_index + 1 < tasks_filtered_len {
                            current_index += 1;
                            list_state.select(Some(current_index));
                        }
                    }
                    KeyCode::Char('k') if matches!(input_mode, InputMode::View) => {
                        if current_index > 0 {
                            current_index -= 1;
                            list_state.select(Some(current_index));
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let InputMode::View = input_mode {
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
                        } else {
                            input.push(' ');
                        }
                    }
                    KeyCode::Char('o') if matches!(input_mode, InputMode::View) => {
                        input_mode = InputMode::Add;
                        input.clear();
                    }
                    KeyCode::Char('/') if matches!(input_mode, InputMode::View) => {
                        input_mode = InputMode::Filter;
                        input.clear();
                    }
                    KeyCode::Char('i') if matches!(input_mode, InputMode::View) => {
                        input_mode = InputMode::Edit;
                        input.clear();
                        if let Some(task) = app.filter_tasks(&filter).get(current_index) {
                            input = task.description.clone();
                        }
                    }
                    KeyCode::Enter => {
                        match input_mode {
                            InputMode::Add => {
                                app.add_task(input.clone());
                                app.save_to_file(&todo_file_path);
                                input_mode = InputMode::View;
                            }
                            InputMode::Filter => {
                                filter = input.clone();
                                input_mode = InputMode::View;
                            }
                            InputMode::Edit => {
                                let tasks_filtered = app.filter_tasks(&filter);
                                if let Some(task) = tasks_filtered.get(current_index) {
                                    let original_index = app
                                        .tasks
                                        .iter()
                                        .position(|t| t.description == task.description)
                                        .unwrap();
                                    app.edit_task(original_index, input.clone());
                                    app.save_to_file(&todo_file_path);
                                    input_mode = InputMode::View;
                                }
                            }
                            _ => {}
                        }
                        input.clear();
                    }
                    KeyCode::Char(c)
                        if matches!(
                            input_mode,
                            InputMode::Add | InputMode::Filter | InputMode::Edit
                        ) =>
                    {
                        input.push(c);
                    }
                    KeyCode::Backspace
                        if matches!(
                            input_mode,
                            InputMode::Add | InputMode::Filter | InputMode::Edit
                        ) =>
                    {
                        input.pop();
                    }
                    KeyCode::Esc => {
                        input.clear();
                        input_mode = InputMode::View;
                    }
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        input_mode = InputMode::View;
                        input.clear();
                    }
                    _ => {}
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
