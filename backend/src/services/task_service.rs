use crate::db::db_task::DbTask;
use crate::db::db_task_log::{DbTaskLog, TaskLogAction};
use crate::models::task::{Task, TaskStatus};
use crate::utils::validator::{
    validate_task_deadline, validate_task_description, validate_task_difficulty,
    validate_task_keywords, validate_task_name, validate_task_priority,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_keywords: Option<Vec<String>>,
    pub task_priority: Option<u8>,
    pub task_difficulty: Option<u8>,
    pub task_deadline: Option<i64>,
    pub task_team_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub task_name: Option<String>,
    pub task_description: Option<String>,
    pub task_keywords: Option<Vec<String>>,
    pub task_priority: Option<u8>,
    pub task_difficulty: Option<u8>,
    pub task_deadline: Option<Option<i64>>,
    pub task_status: Option<TaskStatus>,
    pub task_leader_id: Option<u64>,
    pub task_team_id: Option<Option<u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub task_status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskPriorityRequest {
    pub task_priority: u8,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<TaskStatus>,
    pub priority: Option<u8>,
    pub deadline_before: Option<i64>,
    pub deadline_after: Option<i64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub team_id: Option<u64>,
}

pub struct TaskService;

impl TaskService {
    pub async fn create_task(
        pool: &PgPool,
        user_id: u64,
        request: CreateTaskRequest,
    ) -> Result<Task> {
        validate_task_name(&request.task_name)?;

        if let Some(ref desc) = request.task_description {
            validate_task_description(desc)?;
        }

        if let Some(priority) = request.task_priority {
            validate_task_priority(priority)?;
        }

        if let Some(difficulty) = request.task_difficulty {
            validate_task_difficulty(difficulty)?;
        }

        if let Some(deadline) = request.task_deadline {
            validate_task_deadline(deadline)?;
        }

        if let Some(ref keywords) = request.task_keywords {
            validate_task_keywords(keywords)?;
        }

        let keywords = request
            .task_keywords
            .unwrap_or_default()
            .into_iter()
            .collect::<HashSet<String>>();

        let task = DbTask::create_task(
            pool,
            &request.task_name,
            request.task_description.as_deref(),
            keywords,
            request.task_priority.unwrap_or(0),
            request.task_difficulty.unwrap_or(0),
            request.task_deadline,
            user_id,
            request.task_team_id,
            None,
        )
        .await?;

        let _ = DbTaskLog::create_task_log(
            pool,
            task.task_id,
            user_id,
            TaskLogAction::Created,
            None,
            None,
            Some(&format!("Created task: {}", task.task_name)),
        )
        .await;

        Ok(task)
    }

    pub async fn get_task_by_id(pool: &PgPool, task_id: u64) -> Result<Option<Task>> {
        DbTask::get_task_by_id(pool, task_id).await
    }

    pub async fn list_tasks(
        pool: &PgPool,
        user_id: Option<u64>,
        team_id: Option<u64>,
        query: ListTasksQuery,
    ) -> Result<Vec<Task>> {
        DbTask::list_tasks(
            pool,
            user_id,
            team_id,
            query.status,
            query.priority,
            query.deadline_before,
            query.deadline_after,
            query.limit,
            query.offset,
            false,
        )
        .await
    }

    pub async fn list_deleted_tasks(
        pool: &PgPool,
        user_id: Option<u64>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Task>> {
        DbTask::list_deleted_tasks(pool, user_id, limit, offset).await
    }

    pub async fn restore_task(pool: &PgPool, task_id: u64) -> Result<bool> {
        DbTask::restore_task(pool, task_id).await
    }

    pub async fn permanent_delete_task(pool: &PgPool, task_id: u64) -> Result<bool> {
        DbTask::permanent_delete_task(pool, task_id).await
    }

    pub async fn update_task(
        pool: &PgPool,
        task_id: u64,
        request: UpdateTaskRequest,
    ) -> Result<Option<Task>> {
        if let Some(ref name) = request.task_name {
            validate_task_name(name)?;
        }

        if let Some(ref desc) = request.task_description {
            validate_task_description(desc)?;
        }

        if let Some(priority) = request.task_priority {
            validate_task_priority(priority)?;
        }

        if let Some(difficulty) = request.task_difficulty {
            validate_task_difficulty(difficulty)?;
        }

        if let Some(Some(deadline)) = request.task_deadline {
            validate_task_deadline(deadline)?;
        }

        if let Some(ref keywords) = request.task_keywords {
            validate_task_keywords(keywords)?;
        }

        DbTask::update_task(
            pool,
            task_id,
            request.task_name.as_deref(),
            request.task_description.as_deref(),
            request
                .task_keywords
                .map(|k| k.into_iter().collect::<HashSet<String>>()),
            request.task_priority,
            request.task_difficulty,
            request.task_deadline,
            request.task_status,
            request.task_leader_id,
            request.task_team_id,
            None,
        )
        .await
    }

    pub async fn delete_task(pool: &PgPool, task_id: u64) -> Result<bool> {
        DbTask::delete_task(pool, task_id).await
    }

    pub async fn update_task_status(
        pool: &PgPool,
        task_id: u64,
        status: TaskStatus,
    ) -> Result<Option<Task>> {
        DbTask::set_task_status(pool, task_id, status).await?;
        DbTask::get_task_by_id(pool, task_id).await
    }

    pub async fn update_task_priority(
        pool: &PgPool,
        task_id: u64,
        priority: u8,
    ) -> Result<Option<Task>> {
        // update_task: name, desc, keywords, priority, difficulty, deadline, status, leader, team, group
        DbTask::update_task(
            pool,
            task_id,
            None,
            None,
            None,
            Some(priority),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// 将团队任务指派给指定小组
    /// 仅团队 leader 有权操作
    pub async fn assign_task_to_group(
        pool: &PgPool,
        task_id: u64,
        group_id: u64,
        requester_id: u64,
    ) -> Result<Option<Task>> {
        // 1. 获取任务
        let task = DbTask::get_task_by_id(pool, task_id).await?;
        let task = task.ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // 2. 任务必须属于某个团队
        let team_id = task.task_team_id.ok_or_else(|| {
            anyhow::anyhow!("Task does not belong to any team, cannot assign to group")
        })?;

        // 3. 校验请求者是团队 leader
        let team = crate::db::db_team::DbTeam::get_team_by_id(pool, team_id).await?;
        let team = team.ok_or_else(|| anyhow::anyhow!("Team not found"))?;
        if team.team_leader_id != requester_id {
            return Err(anyhow::anyhow!(
                "Forbidden: only team leader can assign task to group"
            ));
        }

        // 4. 校验小组属于该团队
        let group = crate::db::db_group::DbGroup::get_group_by_id(pool, group_id).await?;
        let group = group.ok_or_else(|| anyhow::anyhow!("Group not found"))?;
        if group.team_id != team_id {
            return Err(anyhow::anyhow!(
                "Forbidden: group does not belong to the task's team"
            ));
        }

        // 5. 更新任务的 task_group_id = Some(group_id)
        // update_task args: name, desc, keywords, priority, difficulty, deadline, status, leader, team, group
        DbTask::update_task(
            pool,
            task_id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some(group_id)),
        )
        .await
    }
}
