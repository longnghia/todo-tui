use crate::task::{Task, TaskStatus};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Serialize, Deserialize)]
pub struct TodoApp {
    pub tasks: Vec<Task>,
}

impl TodoApp {
    pub fn new() -> TodoApp {
        TodoApp { tasks: vec![] }
    }

    pub fn load_from_file(filename: &Path) -> io::Result<TodoApp> {
        if filename.exists() {
            let content = fs::read_to_string(filename)?;
            let tasks = serde_json::from_str(&content)?;
            Ok(tasks)
        } else {
            Ok(TodoApp::new())
        }
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn add_task(
        &mut self,
        description: String,
        current_status: Option<TaskStatus>,
        current_index: Option<usize>,
    ) {
        let tasks = if description.contains(": ") && description.contains("; ") {
            let parts: Vec<&str> = description.splitn(2, ": ").collect();
            if parts.len() == 2 {
                let name = parts[0].trim();
                let subtasks: Vec<String> = parts[1]
                    .split(";")
                    .map(|s| format!("{}: {}", name, s.trim()))
                    .collect();
                subtasks
            } else {
                vec![description]
            }
        } else {
            vec![description]
        };

        let mut insert_index = match (&current_status, current_index) {
            (Some(TaskStatus::Pending | TaskStatus::Undone), Some(index)) => index + 1,
            _ => self
                .tasks
                .iter()
                .rposition(|t| t.status == TaskStatus::Undone)
                .map(|i| i + 1)
                .unwrap_or(0),
        };

        for task_description in tasks {
            let status = match current_status {
                Some(TaskStatus::Pending) => TaskStatus::Pending,
                _ => TaskStatus::Undone,
            };

            let task = Task {
                description: task_description,
                status,
                created_at: Some(Local::now()),
            };

            self.tasks.insert(insert_index, task);
            insert_index += 1;
        }
    }

    pub fn delete_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            self.tasks.remove(index);
        }
    }

    pub fn remove_done_tasks(&mut self) {
        self.tasks.retain(|task| task.status != TaskStatus::Done);
    }

    pub fn edit_task(&mut self, index: usize, new_description: String) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.description = new_description;
        }
    }

    pub fn toggle_task(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.status = match task.status {
                TaskStatus::Undone => TaskStatus::Done,
                TaskStatus::Pending => TaskStatus::Undone,
                TaskStatus::Done => TaskStatus::Undone,
            };
            self.reorder_tasks();
        }
    }

    pub fn toggle_pending(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.status = match task.status {
                TaskStatus::Undone => TaskStatus::Pending,
                TaskStatus::Pending => TaskStatus::Undone,
                TaskStatus::Done => TaskStatus::Pending,
            };
            self.reorder_tasks();
        }
    }

    pub fn reorder_tasks(&mut self) {
        self.tasks.sort_by_key(|t| match t.status {
            TaskStatus::Undone => 0,
            TaskStatus::Pending => 1,
            TaskStatus::Done => 2,
        });
    }

    pub fn filter_tasks(&self, query: &str) -> Vec<Task> {
        self.tasks
            .iter()
            .filter(|task| task.description.contains(query))
            .cloned()
            .collect()
    }

    pub fn completion_percentage(&self) -> f32 {
        let done_count = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count();
        let undone_count = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Undone)
            .count();
        let total_count = done_count + undone_count;

        if total_count == 0 {
            0.0
        } else {
            (done_count as f32 / total_count as f32) * 100.0
        }
    }
}
