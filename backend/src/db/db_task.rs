use crate::models::task::{Task, TaskStatus};
use crate::utils::id_generator::generate_task_id;
use anyhow::Result;
use sqlx::{PgPool, Row};
use std::collections::HashSet;

pub struct DbTask;

impl DbTask {
    /// 将 TaskStatus 转换为数据库字符串
    fn task_status_to_db_string(status: &TaskStatus) -> &'static str {
        match status {
            TaskStatus::Active => "Active",
            TaskStatus::Completed => "Completed",
            TaskStatus::Paused => "Paused",
        }
    }

    /// 创建新任务
    pub async fn create_task(
        pool: &PgPool,
        task_name: &str,
        task_description: Option<&str>,
        task_keywords: HashSet<String>,
        task_priority: u8,
        task_difficulty: u8,
        task_deadline: Option<i64>,
        task_leader_id: u64,
        task_team_id: Option<u64>,
    ) -> Result<Task> {
        let task_id = generate_task_id();
        let task_create_time = chrono::Utc::now().timestamp();
        let task_status = TaskStatus::Active;
        let task_complete_time: Option<i64> = None;
        let task_update_time: Option<i64> = None;
        let is_favorite = false;
        let is_deleted = false;
        let deleted_at: Option<i64> = None;

        let task_keywords_json = serde_json::to_value(&task_keywords)?;

        let result = sqlx::query(
            r#"
            INSERT INTO tasks (
                task_id, task_name, task_description, task_keywords,
                task_priority, task_difficulty, task_deadline, task_complete_time,
                task_status, task_create_time, task_leader_id,
                task_team_id, task_update_time, is_favorite, is_deleted, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING task_id, task_name, task_description, task_keywords,
                      task_priority, task_difficulty, task_deadline, task_complete_time,
                      task_status, task_create_time, task_leader_id,
                      task_team_id, task_update_time, is_favorite, is_deleted, deleted_at
            "#,
        )
        .bind(task_id as i64)
        .bind(task_name)
        .bind(task_description)
        .bind(task_keywords_json)
        .bind(task_priority as i32)
        .bind(task_difficulty as i16)
        .bind(task_deadline)
        .bind(task_complete_time)
        .bind(Self::task_status_to_db_string(&task_status))
        .bind(task_create_time)
        .bind(task_leader_id as i64)
        .bind(task_team_id.map(|id| id as i64))
        .bind(task_update_time)
        .bind(is_favorite)
        .bind(is_deleted)
        .bind(deleted_at)
        .fetch_one(pool)
        .await?;

        tracing::info!("创建任务成功: task_id = {}", task_id);

        Ok(Self::row_to_task(result)?)
    }

    /// 根据任务ID查询任务
    pub async fn get_task_by_id(pool: &PgPool, task_id: u64) -> Result<Option<Task>> {
        let result = sqlx::query(
            r#"
            SELECT task_id, task_name, task_description, task_keywords,
                   task_priority, task_difficulty, task_deadline, task_complete_time,
                   task_status, task_create_time, task_leader_id,
                   task_team_id, task_update_time, is_favorite, is_deleted, deleted_at
            FROM tasks
            WHERE task_id = $1
            "#,
        )
        .bind(task_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_task(row)?)),
            None => Ok(None),
        }
    }

    /// 获取任务列表（支持多条件筛选）
    pub async fn list_tasks(
        pool: &PgPool,
        task_leader_id: Option<u64>,
        task_team_id: Option<u64>,
        task_status: Option<TaskStatus>,
        task_priority: Option<u8>,
        task_deadline_before: Option<i64>,
        task_deadline_after: Option<i64>,
        limit: Option<u32>,
        offset: Option<u32>,
        include_deleted: bool,
    ) -> Result<Vec<Task>> {
        let mut query = String::from(
            "SELECT task_id, task_name, task_description, task_keywords,
                    task_priority, task_difficulty, task_deadline, task_complete_time,
                    task_status, task_create_time, task_leader_id,
                    task_team_id, task_update_time, is_favorite, is_deleted, deleted_at
             FROM tasks WHERE is_deleted = false",
        );
        let mut param_count = 1;

        let mut user_team_ids: Vec<i64> = Vec::new();
        let mut leader_filter_added = false;
        
        if let Some(leader_id) = task_leader_id {
            let teams = crate::db::db_team::DbTeam::list_teams(pool, None, Some(leader_id)).await?;
            user_team_ids = teams.into_iter().map(|t| t.team_id as i64).collect();
            
            if user_team_ids.is_empty() {
                query.push_str(&format!(" AND task_leader_id = ${}", param_count));
                param_count += 1;
                leader_filter_added = true;
            } else {
                let team_ids_str: Vec<String> = user_team_ids.iter().map(|id| id.to_string()).collect();
                query.push_str(&format!(
                    " AND (task_leader_id = ${} OR task_team_id IN ({}))",
                    param_count,
                    team_ids_str.join(", ")
                ));
                param_count += 1;
                leader_filter_added = true;
            }
        } else if let Some(team_id) = task_team_id {
            query.push_str(&format!(" AND task_team_id = ${}", param_count));
            param_count += 1;
        }
        if let Some(status) = &task_status {
            query.push_str(&format!(" AND task_status = ${}", param_count));
            param_count += 1;
        }
        if let Some(priority) = task_priority {
            query.push_str(&format!(" AND task_priority = ${}", param_count));
            param_count += 1;
        }
        if let Some(before) = task_deadline_before {
            query.push_str(&format!(" AND task_deadline <= ${}", param_count));
            param_count += 1;
        }
        if let Some(after) = task_deadline_after {
            query.push_str(&format!(" AND task_deadline >= ${}", param_count));
            param_count += 1;
        }

        query.push_str(" ORDER BY task_create_time DESC");

        if let Some(limit_val) = limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            param_count += 1;
        }
        if let Some(offset_val) = offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            param_count += 1;
        }

        let use_leader_filter = leader_filter_added;
        let team_filter = if !leader_filter_added { task_team_id } else { None };

        let mut query_builder = sqlx::query(&query);

        if let Some(leader_id) = task_leader_id {
            if use_leader_filter {
                query_builder = query_builder.bind(leader_id as i64);
            }
        }
        if let Some(team_id) = team_filter {
            query_builder = query_builder.bind(team_id as i64);
        }
        if let Some(status) = task_status {
            query_builder = query_builder.bind(Self::task_status_to_db_string(&status));
        }
        if let Some(priority) = task_priority {
            query_builder = query_builder.bind(priority as i32);
        }
        if let Some(before) = task_deadline_before {
            query_builder = query_builder.bind(before);
        }
        if let Some(after) = task_deadline_after {
            query_builder = query_builder.bind(after);
        }
        if let Some(limit_val) = limit {
            query_builder = query_builder.bind(limit_val as i32);
        }
        if let Some(offset_val) = offset {
            query_builder = query_builder.bind(offset_val as i32);
        }

        let rows = query_builder.fetch_all(pool).await?;

        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    /// 更新任务信息（支持部分字段更新）
    pub async fn update_task(
        pool: &PgPool,
        task_id: u64,
        task_name: Option<&str>,
        task_description: Option<&str>,
        task_keywords: Option<HashSet<String>>,
        task_priority: Option<u8>,
        task_difficulty: Option<u8>,
        task_deadline: Option<Option<i64>>, // Option<Option> 表示可以更新为None
        task_status: Option<TaskStatus>,
        task_leader_id: Option<u64>,
        task_team_id: Option<Option<u64>>, // Option<Option> 表示可以更新为None
    ) -> Result<Option<Task>> {
        let mut updates = Vec::new();
        let mut param_count = 1usize;

        if task_name.is_some() {
            updates.push(format!("task_name = ${}", param_count));
            param_count += 1;
        }
        if task_description.is_some() {
            updates.push(format!("task_description = ${}", param_count));
            param_count += 1;
        }
        if task_keywords.is_some() {
            updates.push(format!("task_keywords = ${}", param_count));
            param_count += 1;
        }
        if task_priority.is_some() {
            updates.push(format!("task_priority = ${}", param_count));
            param_count += 1;
        }
        if task_difficulty.is_some() {
            updates.push(format!("task_difficulty = ${}", param_count));
            param_count += 1;
        }
        if task_deadline.is_some() {
            updates.push(format!("task_deadline = ${}", param_count));
            param_count += 1;
        }
        if task_status.is_some() {
            updates.push(format!("task_status = ${}", param_count));
            param_count += 1;
        }
        if task_leader_id.is_some() {
            updates.push(format!("task_leader_id = ${}", param_count));
            param_count += 1;
        }
        if task_team_id.is_some() {
            updates.push(format!("task_team_id = ${}", param_count));
            param_count += 1;
        }
        updates.push(format!("task_update_time = ${}", param_count));
        param_count += 1;

        if updates.len() == 1 {
            // 只有task_update_time被更新，但我们仍然需要获取任务
            return Self::get_task_by_id(pool, task_id).await;
        }

        let set_clause = updates.join(", ");
        let query = format!(
            "UPDATE tasks SET {} WHERE task_id = ${} RETURNING task_id, task_name, task_description, task_keywords, task_priority, task_difficulty, task_deadline, task_complete_time, task_status, task_create_time, task_leader_id, task_team_id, task_update_time, is_favorite, is_deleted, deleted_at",
            set_clause, param_count
        );

        let mut row_result = sqlx::query(&query);

        if let Some(v) = task_name {
            row_result = row_result.bind(v);
        }
        if let Some(v) = task_description {
            row_result = row_result.bind(v);
        }
        if let Some(v) = task_keywords {
            let keywords_json = serde_json::to_value(&v)?;
            row_result = row_result.bind(keywords_json);
        }
        if let Some(v) = task_priority {
            row_result = row_result.bind(v as i32);
        }
        if let Some(v) = task_difficulty {
            row_result = row_result.bind(v as i32);
        }
        if let Some(v) = task_deadline {
            row_result = row_result.bind(v);
        }
        if let Some(v) = task_status {
            row_result = row_result.bind(Self::task_status_to_db_string(&v));
        }
        if let Some(v) = task_leader_id {
            row_result = row_result.bind(v as i64);
        }
        if let Some(v) = task_team_id {
            row_result = row_result.bind(v.map(|id| id as i64));
        }
        // 绑定task_update_time（当前时间戳）
        let update_time = chrono::Utc::now().timestamp();
        row_result = row_result.bind(update_time);

        row_result = row_result.bind(task_id as i64);

        let result = row_result.fetch_optional(pool).await?;

        match result {
            Some(row) => {
                tracing::info!("更新任务成功: task_id = {}", task_id);
                Ok(Some(Self::row_to_task(row)?))
            }
            None => Ok(None),
        }
    }

    /// 删除任务（软删除）
    pub async fn delete_task(pool: &PgPool, task_id: u64) -> Result<bool> {
        let delete_time = chrono::Utc::now().timestamp();
        let result = sqlx::query("UPDATE tasks SET is_deleted = true, deleted_at = $1 WHERE task_id = $2")
            .bind(delete_time)
            .bind(task_id as i64)
            .execute(pool)
            .await?;

        let affected = result.rows_affected();
        tracing::info!("软删除任务: task_id = {}, affected = {}", task_id, affected);
        Ok(affected > 0)
    }

    /// 恢复已删除的任务
    pub async fn restore_task(pool: &PgPool, task_id: u64) -> Result<bool> {
        let update_time = chrono::Utc::now().timestamp();
        let result = sqlx::query("UPDATE tasks SET is_deleted = false, deleted_at = NULL, task_update_time = $1 WHERE task_id = $2")
            .bind(update_time)
            .bind(task_id as i64)
            .execute(pool)
            .await?;

        let affected = result.rows_affected();
        tracing::info!("恢复任务: task_id = {}, affected = {}", task_id, affected);
        Ok(affected > 0)
    }

    /// 永久删除任务
    pub async fn permanent_delete_task(pool: &PgPool, task_id: u64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM tasks WHERE task_id = $1")
            .bind(task_id as i64)
            .execute(pool)
            .await?;

        let affected = result.rows_affected();
        tracing::info!("永久删除任务: task_id = {}, affected = {}", task_id, affected);
        Ok(affected > 0)
    }

    /// 获取已删除的任务列表（回收站）
    pub async fn list_deleted_tasks(
        pool: &PgPool,
        task_leader_id: Option<u64>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Task>> {
        let mut query = String::from(
            "SELECT task_id, task_name, task_description, task_keywords,
                    task_priority, task_difficulty, task_deadline, task_complete_time,
                    task_status, task_create_time, task_leader_id,
                    task_team_id, task_update_time, is_favorite, is_deleted, deleted_at
             FROM tasks WHERE is_deleted = true",
        );
        let mut param_count = 1;

        if let Some(leader_id) = task_leader_id {
            query.push_str(&format!(" AND task_leader_id = ${}", param_count));
            param_count += 1;
        }

        query.push_str(" ORDER BY deleted_at DESC");

        if let Some(limit_val) = limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            param_count += 1;
        }
        if let Some(offset_val) = offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            param_count += 1;
        }

        let mut query_builder = sqlx::query(&query);

        if let Some(leader_id) = task_leader_id {
            query_builder = query_builder.bind(leader_id as i64);
        }
        if let Some(limit_val) = limit {
            query_builder = query_builder.bind(limit_val as i32);
        }
        if let Some(offset_val) = offset {
            query_builder = query_builder.bind(offset_val as i32);
        }

        let rows = query_builder.fetch_all(pool).await?;

        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            tasks.push(Self::row_to_task(row)?);
        }
        Ok(tasks)
    }

    /// 切换任务收藏状态
    pub async fn toggle_favorite(pool: &PgPool, task_id: u64) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE tasks SET is_favorite = NOT is_favorite WHERE task_id = $1 RETURNING is_favorite",
        )
        .bind(task_id as i64)
        .fetch_one(pool)
        .await?;

        let new_status: bool = result.get("is_favorite");
        tracing::info!(
            "切换任务收藏状态: task_id = {}, is_favorite = {}",
            task_id,
            new_status
        );

        Ok(new_status)
    }

    /// 按状态设置任务状态，并同步更新完成时间
    pub async fn set_task_status(pool: &PgPool, task_id: u64, status: TaskStatus) -> Result<bool> {
        let update_time = chrono::Utc::now().timestamp();

        let (task_status, complete_time): (&str, Option<i64>) = match status {
            TaskStatus::Completed => ("Completed", Some(update_time)),
            _ => ("Active", None),
        };

        let result = sqlx::query(
            "UPDATE tasks SET task_status = $1, task_complete_time = $2, task_update_time = $3 WHERE task_id = $4",
        )
        .bind(task_status)
        .bind(complete_time)
        .bind(update_time)
        .bind(task_id as i64)
        .execute(pool)
        .await?;

        let affected = result.rows_affected();
        tracing::info!(
            "设置任务状态: task_id = {}, status = {:?}, complete_time = {:?}, affected = {}",
            task_id,
            status,
            complete_time,
            affected
        );

        Ok(affected > 0)
    }

    /// 将数据库行转换为 Task 结构体
    fn row_to_task(row: sqlx::postgres::PgRow) -> Result<Task> {
        let task_id: i64 = row.get("task_id");
        let task_name: String = row.get("task_name");
        let task_description: Option<String> = row.get("task_description");
        let task_keywords: serde_json::Value = row.get("task_keywords");
        let task_keywords: HashSet<String> =
            serde_json::from_value(task_keywords).unwrap_or_default();
        let task_priority: i32 = row.get("task_priority");
        let task_difficulty: i16 = row.get("task_difficulty");
        let task_deadline: Option<i64> = row.get("task_deadline");
        let task_complete_time: Option<i64> = row.get("task_complete_time");
        let task_status: String = row.get("task_status");
        let task_create_time: i64 = row.get("task_create_time");
        let task_leader_id: i64 = row.get("task_leader_id");
        let task_team_id: Option<i64> = row.get("task_team_id");
        let task_update_time: Option<i64> = row.get("task_update_time");
        let is_favorite: bool = row.get("is_favorite");
        let is_deleted: bool = row.get("is_deleted");
        let deleted_at: Option<i64> = row.get("deleted_at");

        let task_status = match task_status.as_str() {
            "Active" => TaskStatus::Active,
            "Completed" => TaskStatus::Completed,
            "Paused" => TaskStatus::Paused,
            _ => TaskStatus::Active,
        };

        Ok(Task {
            task_id: task_id as u64,
            task_name,
            task_description,
            task_keywords,
            task_priority: task_priority as u8,
            task_difficulty: task_difficulty as u8,
            task_deadline,
            task_complete_time,
            task_status,
            task_create_time,
            task_leader_id: task_leader_id as u64,
            task_team_id: task_team_id.map(|id| id as u64),
            task_update_time,
            is_favorite,
            is_deleted,
            deleted_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::create_pool;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_task_crud() {
        let pool = create_pool().await.unwrap();

        // 创建任务
        let mut keywords = HashSet::new();
        keywords.insert("重要".to_string());
        keywords.insert("紧急".to_string());

        let task = DbTask::create_task(
            &pool,
            "测试任务",
            Some("这是一个测试任务"),
            keywords.clone(),
            2,
            0,
            Some(chrono::Utc::now().timestamp() + 86400), // 1天后
            1,                                            // 假设存在用户ID 1
            None,
        )
        .await
        .unwrap();

        println!("创建任务: {:?}", task);
        assert_eq!(task.task_name, "测试任务");
        assert_eq!(task.task_priority, 2);
        assert!(task.task_deadline.is_some());
        assert_eq!(task.task_status, TaskStatus::Active);

        // 查询任务（按ID）
        let found = DbTask::get_task_by_id(&pool, task.task_id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.task_id, task.task_id);
        assert_eq!(found.task_name, "测试任务");

        // 测试列表查询
        let tasks =
            DbTask::list_tasks(&pool, Some(1), None, None, None, None, None, Some(10), None, false)
                .await
                .unwrap();
        assert!(tasks.len() >= 1);

        // 更新任务
        let updated = DbTask::update_task(
            &pool,
            task.task_id,
            Some("更新后的任务名"),
            Some("更新后的描述"),
            None,
            Some(3),
            Some(5),
            Some(None), // 移除截止时间
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.task_name, "更新后的任务名");
        assert_eq!(updated.task_priority, 3);
        assert!(updated.task_deadline.is_none());

        // 完成任务
        let completed = DbTask::set_task_status(&pool, task.task_id, TaskStatus::Completed)
            .await
            .unwrap();
        assert!(completed);
        let completed_task = DbTask::get_task_by_id(&pool, task.task_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(completed_task.task_status, TaskStatus::Completed);
        assert!(completed_task.task_complete_time.is_some());

        // 删除任务
        let deleted = DbTask::delete_task(&pool, task.task_id).await.unwrap();
        assert!(deleted);

        // 确认任务已删除
        let found = DbTask::get_task_by_id(&pool, task.task_id).await.unwrap();
        assert!(found.is_none());
    }
}
