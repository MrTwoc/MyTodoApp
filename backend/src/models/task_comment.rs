/*
    任务评论
*/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub comment_id: u64,
    pub task_id: u64,
    pub user_id: u64,
    pub content: String,
    pub parent_id: Option<u64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}
