use crate::db::db_sub_task::DbSubTask;
use crate::db::db_task::DbTask;
use crate::models::task::{SubTask, Task, TaskStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubTaskRequest {
    pub sub_task_name: String,
    pub sub_task_description: Option<String>,
    pub sub_task_status: Option<TaskStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubTaskRequest {
    pub sub_task_name: Option<String>,
    pub sub_task_description: Option<String>,
    pub sub_task_status: Option<TaskStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubTaskResponse {
    pub sub_task: SubTask,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubTaskListResponse {
    pub sub_tasks: Vec<SubTask>,
}

pub struct SubTaskService;

impl SubTaskService {
    pub async fn create_sub_task(
        pool: &PgPool,
        task_id: u64,
        request: CreateSubTaskRequest,
    ) -> Result<SubTask> {
        let status = request.sub_task_status.unwrap_or(TaskStatus::Active);
        let sub_task = DbSubTask::create_sub_task(
            pool,
            task_id,
            &request.sub_task_name,
            request.sub_task_description.as_deref(),
            Some(status),
        )
        .await?;

        sync_parent_task_status(pool, task_id).await?;
        Ok(sub_task)
    }

    pub async fn list_sub_tasks(pool: &PgPool, task_id: u64) -> Result<Vec<SubTask>> {
        DbSubTask::list_sub_tasks(pool, task_id).await
    }

    pub async fn update_sub_task(
        pool: &PgPool,
        task_id: u64,
        sub_task_id: u64,
        request: UpdateSubTaskRequest,
    ) -> Result<Option<SubTask>> {
        let updated = DbSubTask::update_sub_task(
            pool,
            task_id,
            sub_task_id,
            request.sub_task_name.as_deref(),
            request.sub_task_description.as_deref(),
            request.sub_task_status,
        )
        .await?;

        if updated.is_some() {
            sync_parent_task_status(pool, task_id).await?;
        }

        Ok(updated)
    }

    pub async fn delete_sub_task(
        pool: &PgPool,
        task_id: u64,
        sub_task_id: u64,
    ) -> Result<bool> {
        let deleted = DbSubTask::delete_sub_task(pool, task_id, sub_task_id).await?;
        if deleted {
            sync_parent_task_status(pool, task_id).await?;
        }
        Ok(deleted)
    }
}

pub async fn sync_parent_task_status(pool: &PgPool, task_id: u64) -> Result<()> {
    let subtasks = DbSubTask::list_sub_tasks(pool, task_id).await?;
    let target_status = if subtasks.is_empty() {
        TaskStatus::Active
    } else if subtasks
        .iter()
        .all(|sub_task| matches!(sub_task.sub_task_status, TaskStatus::Completed))
    {
        TaskStatus::Completed
    } else {
        TaskStatus::Active
    };

    let _ = DbTask::set_task_status(pool, task_id, target_status).await?;

    Ok(())
}
