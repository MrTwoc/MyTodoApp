/*
    任务评论数据库操作
*/
pub use crate::models::task_comment::TaskComment;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbTaskComment;

impl DbTaskComment {
    /// 创建评论
    pub async fn create_comment(
        pool: &PgPool,
        task_id: u64,
        user_id: u64,
        content: &str,
        parent_id: Option<u64>,
    ) -> Result<TaskComment> {
        let created_at = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO task_comments (task_id, user_id, content, parent_id, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING comment_id, task_id, user_id, content, parent_id, created_at, updated_at
            "#,
        )
        .bind(task_id as i64)
        .bind(user_id as i64)
        .bind(content)
        .bind(parent_id.map(|id| id as i64))
        .bind(created_at)
        .fetch_one(pool)
        .await?;

        tracing::info!(
            "创建任务评论成功: task_id = {}, user_id = {}",
            task_id,
            user_id
        );

        Ok(Self::row_to_task_comment(result)?)
    }

    /// 获取任务的评论列表
    pub async fn get_comments_by_task_id(
        pool: &PgPool,
        task_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<TaskComment>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let result = sqlx::query(
            r#"
            SELECT comment_id, task_id, user_id, content, parent_id, created_at, updated_at
            FROM task_comments
            WHERE task_id = $1
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(task_id as i64)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

        let comments: Vec<TaskComment> = result
            .into_iter()
            .map(|row| Self::row_to_task_comment(row))
            .collect::<Result<Vec<_>, _>>()?;

        tracing::info!(
            "获取任务评论列表成功: task_id = {}, count = {}",
            task_id,
            comments.len()
        );

        Ok(comments)
    }

    /// 更新评论
    pub async fn update_comment(
        pool: &PgPool,
        comment_id: u64,
        content: &str,
    ) -> Result<Option<TaskComment>> {
        let updated_at = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            UPDATE task_comments
            SET content = $1, updated_at = $2
            WHERE comment_id = $3
            RETURNING comment_id, task_id, user_id, content, parent_id, created_at, updated_at
            "#,
        )
        .bind(content)
        .bind(updated_at)
        .bind(comment_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => {
                tracing::info!("更新任务评论成功: comment_id = {}", comment_id);
                Ok(Some(Self::row_to_task_comment(row)?))
            }
            None => {
                tracing::warn!("任务评论不存在: comment_id = {}", comment_id);
                Ok(None)
            }
        }
    }

    /// 删除评论
    pub async fn delete_comment(pool: &PgPool, comment_id: u64) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM task_comments
            WHERE comment_id = $1
            "#,
        )
        .bind(comment_id as i64)
        .execute(pool)
        .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            tracing::info!("删除任务评论成功: comment_id = {}", comment_id);
        } else {
            tracing::warn!("任务评论不存在: comment_id = {}", comment_id);
        }

        Ok(deleted)
    }

    /// 获取评论
    pub async fn get_comment_by_id(pool: &PgPool, comment_id: u64) -> Result<Option<TaskComment>> {
        let result = sqlx::query(
            r#"
            SELECT comment_id, task_id, user_id, content, parent_id, created_at, updated_at
            FROM task_comments
            WHERE comment_id = $1
            "#,
        )
        .bind(comment_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_task_comment(row)?)),
            None => Ok(None),
        }
    }

    fn row_to_task_comment(row: sqlx::postgres::PgRow) -> Result<TaskComment> {
        Ok(TaskComment {
            comment_id: row.try_get::<i64, _>("comment_id")? as u64,
            task_id: row.try_get::<i64, _>("task_id")? as u64,
            user_id: row.try_get::<i64, _>("user_id")? as u64,
            content: row.try_get("content")?,
            parent_id: row.try_get::<Option<i64>, _>("parent_id")?.map(|id| id as u64),
            created_at: row.try_get::<i64, _>("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
