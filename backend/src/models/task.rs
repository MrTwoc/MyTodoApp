/*
 * 任务结构体
*/

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: u64,
    // 任务名称
    pub task_name: String,
    // 任务描述
    pub task_description: Option<String>,
    // 任务标签、关键词列表，HashSet<String>类型
    pub task_keywords: HashSet<String>,
    // 任务优先级
    pub task_priority: u8,
    // 任务截止时间，Unix时间戳，可选（无截止时间为None）
    pub task_deadline: Option<i64>,
    // 任务完成时间，Unix时间戳
    pub task_complete_time: Option<i64>,
    // 任务状态: 进行中或已完成
    pub task_status: TaskStatus,
    // 任务创建时间，Unix时间戳
    pub task_create_time: i64,
    // 任务负责人 雪花ID，u64类型(user中user_id)
    pub task_leader_id: u64,
    // 任务团队ID，雪花ID，u64类型(team中team_id)
    pub task_team_id: Option<u64>,
    // 任务更新时间，Unix时间戳
    pub task_update_time: Option<i64>,
    // 任务难度值(0-10)
    pub task_difficulty: u8,
    // 是否收藏
    pub is_favorite: bool,
    // 是否已删除（软删除）
    pub is_deleted: bool,
    // 删除时间
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Active,    // 进行中 / 默认运行中
    Completed, // 已完成
    Paused,    // 已暂停
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    // 子任务ID，雪花ID
    pub sub_task_id: u64,
    // 对应父任务ID
    pub task_id: u64,
    // 子任务标题
    pub sub_task_name: String,
    // 子任务描述
    pub sub_task_description: Option<String>,
    // 子任务状态
    pub sub_task_status: TaskStatus,
    // 创建时间
    pub sub_task_create_time: i64,
    // 更新时间
    pub sub_task_update_time: Option<i64>,
    // 完成时间
    pub sub_task_complete_time: Option<i64>,
}
