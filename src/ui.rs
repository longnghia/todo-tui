use crate::app::TodoApp;
use crate::task::TaskStatus;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub enum InputMode {
    View,
    Add,
    Edit,
    Filter,
}

pub fn ui<B: Backend>(
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
                Constraint::Length(3), // Status message at the bottom
            ]
            .as_ref(),
        )
        .split(f.size());

    // Use a consistent color scheme
    let undone_color = Color::Red;
    let pending_color = Color::Yellow;
    let done_color = Color::Green;

    // Render tasks list
    let tasks: Vec<ListItem> = app
        .filter_tasks(filter)
        .iter()
        .map(|task| {
            let (symbol, style) = match task.status {
                TaskStatus::Undone => ("[ ]", Style::default().fg(undone_color)),
                TaskStatus::Pending => ("[-]", Style::default().fg(pending_color)),
                TaskStatus::Done => (
                    "[x]",
                    Style::default()
                        .fg(done_color)
                        .add_modifier(Modifier::CROSSED_OUT),
                ),
            };
            let content = Spans::from(vec![Span::styled(
                format!("{} {}", symbol, task.description),
                style,
            )]);
            ListItem::new(content)
        })
        .collect();

    let completion_percentage = app.completion_percentage();
    let title = format!(
        "Todo List (d: delete, D: remove done, Space: toggle) {:.1}% Complete",
        completion_percentage
    );
    let tasks_list = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        )))
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
