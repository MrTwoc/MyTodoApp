/*
    任务操作日志数据库操作
*/
use crate::models::task_log::{Log_TaskLog, TaskLogAction};
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbTaskLog;

impl DbTaskLog {
    /// 创建任务日志
    pub async fn create_task_log(
        pool: &PgPool,
        task_id: u64,
        operator_id: u64,
        action: TaskLogAction,
        old_value: Option<&str>,
        new_value: Option<&str>,
        details: Option<&str>,
    ) -> Result<Log_TaskLog> {
        let created_at = chrono::Utc::now().timestamp();
        let action_str = serde_json::to_string(&action)?;

        let result = sqlx::query(
            r#"
            INSERT INTO task_logs (task_id, operator_id, action, old_value, new_value, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING log_id, task_id, operator_id, action, old_value, new_value, details, created_at
            "#,
        )
        .bind(task_id as i64)
        .bind(operator_id as i64)
        .bind(action_str)
        .bind(old_value)
        .bind(new_value)
        .bind(details)
        .bind(created_at)
        .fetch_one(pool)
        .await?;

        tracing::info!(
            "创建任务日志成功: task_id = {}, operator_id = {}, action = {:?}",
            task_id,
            operator_id,
            action
        );

        Ok(Self::row_to_task_log(result)?)
    }

    /// 获取任务日志列表（支持多条件筛选和分页）
    pub async fn list_task_logs(
        pool: &PgPool,
        task_id: Option<u64>,
        operator_id: Option<u64>,
        action: Option<TaskLogAction>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TaskLog>> {
        let mut query = String::from(
            "SELECT log_id, task_id, operator_id, action, old_value, new_value, details, created_at
             FROM task_logs WHERE 1=1",
        );
        let mut param_count = 1;

        if let Some(tid) = task_id {
            query.push_str(&format!(" AND task_id = ${}", param_count));
            param_count += 1;
        }
        if let Some(oid) = operator_id {
            query.push_str(&format!(" AND operator_id = ${}", param_count));
            param_count += 1;
        }
        if let Some(ref act) = action {
            let action_str = serde_json::to_string(act)?;
            query.push_str(&format!(" AND action = ${}", param_count));
            param_count += 1;
        }
        if let Some(start) = start_time {
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            param_count += 1;
        }
        if let Some(end) = end_time {
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            param_count += 1;
        }
        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
        }

        let mut sql_query = sqlx::query(&query);

        if let Some(tid) = task_id {
            sql_query = sql_query.bind(tid as i64);
        }
        if let Some(oid) = operator_id {
            sql_query = sql_query.bind(oid as i64);
        }
        if let Some(ref act) = action {
            let action_str = serde_json::to_string(act)?;
            sql_query = sql_query.bind(action_str);
        }
        if let Some(start) = start_time {
            sql_query = sql_query.bind(start);
        }
        if let Some(end) = end_time {
            sql_query = sql_query.bind(end);
        }
        if let Some(lim) = limit {
            sql_query = sql_query.bind(lim as i64);
        }
        if let Some(off) = offset {
            sql_query = sql_query.bind(off as i64);
        }

        let rows = sql_query.fetch_all(pool).await?;

        let logs: Vec<Log_TaskLog> = rows
            .into_iter()
            .map(|row| Self::row_to_task_log(row))
            .collect::<Result<Vec<_>, _>>()?;

        tracing::info!("获取任务日志列表成功: count = {}", logs.len());

        Ok(logs)
    }

    /// 获取单个任务的日志列表
    pub async fn get_task_logs(
        pool: &PgPool,
        task_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TaskLog>> {
        Self::list_task_logs(pool, Some(task_id), None, None, None, None, limit, offset).await
    }

    /// 获取操作者的任务日志列表
    pub async fn get_operator_logs(
        pool: &PgPool,
        operator_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TaskLog>> {
        Self::list_task_logs(
            pool,
            None,
            Some(operator_id),
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    fn row_to_task_log(row: sqlx::postgres::PgRow) -> Result<Log_TaskLog> {
        let action_str: String = row.try_get("action")?;
        let action: TaskLogAction = serde_json::from_str(&action_str)?;

        Ok(Log_TaskLog {
            log_id: row.try_get::<i64, _>("log_id")? as u64,
            task_id: row.try_get::<i64, _>("task_id")? as u64,
            operator_id: row.try_get::<i64, _>("operator_id")? as u64,
            action,
            old_value: row.try_get("old_value")?,
            new_value: row.try_get("new_value")?,
            details: row.try_get("details")?,
            created_at: row.try_get::<i64, _>("created_at")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_log_crud() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        // 创建测试用户
        let test_user = crate::db::db_user::DbUser::create_user(
            &pool,
            "test_task_log_user",
            "TestPass123!",
            "test_task_log@example.com",
            "",
        )
        .await
        .unwrap();

        // 创建测试任务
        let test_task = crate::db::db_task::DbTask::create_task(
            &pool,
            "测试任务",
            Some("测试任务描述"),
            std::collections::HashSet::new(),
            1,
            0,
            None,
            test_user.user_id,
            None,
        )
        .await
        .unwrap();

        // 测试创建任务日志
        let log = DbTaskLog::create_task_log(
            &pool,
            test_task.task_id,
            test_user.user_id,
            TaskLogAction::Created,
            None,
            None,
            Some("创建了新任务"),
        )
        .await
        .unwrap();

        assert_eq!(log.task_id, test_task.task_id);
        assert_eq!(log.operator_id, test_user.user_id);
        assert!(matches!(log.action, TaskLogAction::Created));

        // 测试状态变更日志
        let status_log = DbTaskLog::create_task_log(
            &pool,
            test_task.task_id,
            test_user.user_id,
            TaskLogAction::StatusChanged,
            Some("pending"),
            Some("in_progress"),
            Some("任务状态从待处理变更为进行中"),
        )
        .await
        .unwrap();

        assert_eq!(status_log.old_value, Some("pending".to_string()));
        assert_eq!(status_log.new_value, Some("in_progress".to_string()));

        // 测试获取任务日志列表
        let logs = DbTaskLog::get_task_logs(&pool, test_task.task_id, None, None)
            .await
            .unwrap();
        assert_eq!(logs.len(), 2);

        // 测试获取操作者日志列表
        let operator_logs = DbTaskLog::get_operator_logs(&pool, test_user.user_id, None, None)
            .await
            .unwrap();
        assert_eq!(operator_logs.len(), 2);

        // 测试带筛选条件的日志查询
        let filtered_logs = DbTaskLog::list_task_logs(
            &pool,
            Some(test_task.task_id),
            Some(test_user.user_id),
            Some(TaskLogAction::Created),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(filtered_logs.len(), 1);

        // 清理测试数据
        crate::db::db_task::DbTask::delete_task(&pool, test_task.task_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, test_user.user_id)
            .await
            .unwrap();
    }
}
