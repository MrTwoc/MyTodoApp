use crate::models::task::{SubTask, TaskStatus};
use crate::utils::id_generator::generate_task_id;
use anyhow::Result;
use chrono::Utc;
use sqlx::{PgPool, Row};

pub struct DbSubTask;

impl DbSubTask {
    fn task_status_to_db_string(status: &TaskStatus) -> &'static str {
        match status {
            TaskStatus::Active => "Active",
            TaskStatus::Completed => "Completed",
            TaskStatus::Paused => "Paused",
        }
    }

    fn parse_db_status(status: &str) -> TaskStatus {
        match status {
            "Active" => TaskStatus::Active,
            "Completed" => TaskStatus::Completed,
            "Paused" => TaskStatus::Paused,
            _ => TaskStatus::Active,
        }
    }

    pub async fn create_sub_task(
        pool: &PgPool,
        task_id: u64,
        sub_task_name: &str,
        sub_task_description: Option<&str>,
        sub_task_status: Option<TaskStatus>,
    ) -> Result<SubTask> {
        let sub_task_id = generate_task_id();
        let sub_task_create_time = Utc::now().timestamp();
        let status = sub_task_status.unwrap_or(TaskStatus::Active);
        let status_db = Self::task_status_to_db_string(&status);
        let sub_task_complete_time = if matches!(status, TaskStatus::Completed) {
            Some(sub_task_create_time)
        } else {
            None
        };

        let result = sqlx::query(
            r#"
            INSERT INTO sub_tasks (
                sub_task_id, task_id, sub_task_name, sub_task_description,
                sub_task_status, sub_task_create_time, sub_task_complete_time
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING sub_task_id, task_id, sub_task_name, sub_task_description,
                      sub_task_status, sub_task_create_time, sub_task_update_time, sub_task_complete_time
            "#,
        )
        .bind(sub_task_id as i64)
        .bind(task_id as i64)
        .bind(sub_task_name)
        .bind(sub_task_description)
        .bind(status_db)
        .bind(sub_task_create_time)
        .bind(sub_task_complete_time)
        .fetch_one(pool)
        .await?;

        tracing::info!("创建子任务成功: sub_task_id = {}, task_id = {}", sub_task_id, task_id);

        Ok(Self::row_to_sub_task(result)?)
    }

    pub async fn get_sub_task_by_id(
        pool: &PgPool,
        task_id: u64,
        sub_task_id: u64,
    ) -> Result<Option<SubTask>> {
        let result = sqlx::query(
            r#"
            SELECT sub_task_id, task_id, sub_task_name, sub_task_description,
                   sub_task_status, sub_task_create_time, sub_task_update_time, sub_task_complete_time
            FROM sub_tasks
            WHERE task_id = $1 AND sub_task_id = $2
            "#,
        )
        .bind(task_id as i64)
        .bind(sub_task_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_sub_task(row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_sub_tasks(pool: &PgPool, task_id: u64) -> Result<Vec<SubTask>> {
        let rows = sqlx::query(
            r#"
            SELECT sub_task_id, task_id, sub_task_name, sub_task_description,
                   sub_task_status, sub_task_create_time, sub_task_update_time, sub_task_complete_time
            FROM sub_tasks
            WHERE task_id = $1
            ORDER BY sub_task_create_time DESC
            "#,
        )
        .bind(task_id as i64)
        .fetch_all(pool)
        .await?;

        let mut sub_tasks = Vec::with_capacity(rows.len());
        for row in rows {
            sub_tasks.push(Self::row_to_sub_task(row)?);
        }

        Ok(sub_tasks)
    }

    pub async fn update_sub_task(
        pool: &PgPool,
        task_id: u64,
        sub_task_id: u64,
        sub_task_name: Option<&str>,
        sub_task_description: Option<&str>,
        sub_task_status: Option<TaskStatus>,
    ) -> Result<Option<SubTask>> {
        let existing = match Self::get_sub_task_by_id(pool, task_id, sub_task_id).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        let new_name = sub_task_name.unwrap_or(&existing.sub_task_name);
        let new_description = sub_task_description.or(existing.sub_task_description.as_deref());
        let new_status = sub_task_status.unwrap_or(existing.sub_task_status.clone());
        let new_status_db = Self::task_status_to_db_string(&new_status);
        let new_complete_time = if matches!(new_status, TaskStatus::Completed) {
            Some(Utc::now().timestamp())
        } else if new_status == TaskStatus::Active {
            None
        } else {
            existing.sub_task_complete_time
        };
        let update_time = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            UPDATE sub_tasks
            SET sub_task_name = $1,
                sub_task_description = $2,
                sub_task_status = $3,
                sub_task_update_time = $4,
                sub_task_complete_time = $5
            WHERE task_id = $6 AND sub_task_id = $7
            RETURNING sub_task_id, task_id, sub_task_name, sub_task_description,
                      sub_task_status, sub_task_create_time, sub_task_update_time, sub_task_complete_time
            "#,
        )
        .bind(new_name)
        .bind(new_description)
        .bind(new_status_db)
        .bind(update_time)
        .bind(new_complete_time)
        .bind(task_id as i64)
        .bind(sub_task_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_sub_task(row)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_sub_task(pool: &PgPool, task_id: u64, sub_task_id: u64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM sub_tasks WHERE task_id = $1 AND sub_task_id = $2")
            .bind(task_id as i64)
            .bind(sub_task_id as i64)
            .execute(pool)
            .await?;

        let affected = result.rows_affected();
        tracing::info!(
            "删除子任务成功: task_id = {}, sub_task_id = {}, affected = {}",
            task_id,
            sub_task_id,
            affected
        );

        Ok(affected > 0)
    }

    pub async fn set_subtasks_status_by_task_id(
        pool: &PgPool,
        task_id: u64,
        status: TaskStatus,
    ) -> Result<u64> {
        let status_db = Self::task_status_to_db_string(&status);
        let complete_time = if matches!(status, TaskStatus::Completed) {
            Some(Utc::now().timestamp())
        } else {
            None
        };
        let update_time = Utc::now().timestamp();

        let rows_affected = match status {
            TaskStatus::Completed => {
                sqlx::query(
                    r#"
                    UPDATE sub_tasks
                    SET sub_task_status = $1,
                        sub_task_update_time = $2,
                        sub_task_complete_time = $3
                    WHERE task_id = $4
                    "#,
                )
                .bind(status_db)
                .bind(update_time)
                .bind(complete_time)
                .bind(task_id as i64)
                .execute(pool)
                .await?
            }
            _ => {
                sqlx::query(
                    r#"
                    UPDATE sub_tasks
                    SET sub_task_status = $1,
                        sub_task_update_time = $2,
                        sub_task_complete_time = NULL
                    WHERE task_id = $3
                    "#,
                )
                .bind(status_db)
                .bind(update_time)
                .bind(task_id as i64)
                .execute(pool)
                .await?
            }
        }
        .rows_affected();

        tracing::info!(
            "更新子任务状态: task_id = {}, status = {}, affected = {}",
            task_id,
            status_db,
            rows_affected
        );

        Ok(rows_affected)
    }

    fn row_to_sub_task(row: sqlx::postgres::PgRow) -> Result<SubTask> {
        let sub_task_id: i64 = row.get("sub_task_id");
        let task_id: i64 = row.get("task_id");
        let sub_task_name: String = row.get("sub_task_name");
        let sub_task_description: Option<String> = row.get("sub_task_description");
        let status: String = row.get("sub_task_status");
        let sub_task_create_time: i64 = row.get("sub_task_create_time");
        let sub_task_update_time: Option<i64> = row.get("sub_task_update_time");
        let sub_task_complete_time: Option<i64> = row.get("sub_task_complete_time");

        Ok(SubTask {
            sub_task_id: sub_task_id as u64,
            task_id: task_id as u64,
            sub_task_name,
            sub_task_description,
            sub_task_status: Self::parse_db_status(&status),
            sub_task_create_time,
            sub_task_update_time,
            sub_task_complete_time,
        })
    }
}
