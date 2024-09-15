use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskStatus {
    Undone,
    Pending,
    Done,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub description: String,
    pub status: TaskStatus,
    pub created_at: Option<DateTime<Local>>,
}
