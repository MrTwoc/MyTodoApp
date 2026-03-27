/*
    团队操作日志数据库操作
*/
use crate::models::team_log::{Log_TeamLog, LogAction};
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbTeamLog;

impl DbTeamLog {
    /// 创建团队日志
    pub async fn create_team_log(
        pool: &PgPool,
        team_id: u64,
        operator_id: u64,
        action: LogAction,
        target_type: &str,
        target_id: Option<u64>,
        details: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<Log_TeamLog> {
        let created_at = chrono::Utc::now().timestamp();
        let action_str = serde_json::to_string(&action)?;

        let result = sqlx::query(
            r#"
            INSERT INTO team_logs (team_id, operator_id, action, target_type, target_id, details, created_at, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING log_id, team_id, operator_id, action, target_type, target_id, details, created_at, ip_address
            "#,
        )
        .bind(team_id as i64)
        .bind(operator_id as i64)
        .bind(action_str)
        .bind(target_type)
        .bind(target_id.map(|id| id as i64))
        .bind(details)
        .bind(created_at)
        .bind(ip_address)
        .fetch_one(pool)
        .await?;

        tracing::info!(
            "创建团队日志成功: team_id = {}, operator_id = {}, action = {:?}",
            team_id,
            operator_id,
            action
        );

        Ok(Self::row_to_team_log(result)?)
    }

    /// 获取团队日志列表（支持多条件筛选和分页）
    pub async fn list_team_logs(
        pool: &PgPool,
        team_id: Option<u64>,
        operator_id: Option<u64>,
        action: Option<LogAction>,
        target_type: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TeamLog>> {
        let mut query = String::from(
            "SELECT log_id, team_id, operator_id, action, target_type, target_id, details, created_at, ip_address
             FROM team_logs WHERE 1=1",
        );
        let mut param_count = 1;

        if let Some(tid) = team_id {
            query.push_str(&format!(" AND team_id = ${}", param_count));
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
        if let Some(tt) = target_type {
            query.push_str(&format!(" AND target_type = ${}", param_count));
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

        if let Some(tid) = team_id {
            sql_query = sql_query.bind(tid as i64);
        }
        if let Some(oid) = operator_id {
            sql_query = sql_query.bind(oid as i64);
        }
        if let Some(ref act) = action {
            let action_str = serde_json::to_string(act)?;
            sql_query = sql_query.bind(action_str);
        }
        if let Some(tt) = target_type {
            sql_query = sql_query.bind(tt);
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

        let logs: Vec<Log_TeamLog> = rows
            .into_iter()
            .map(|row| Self::row_to_team_log(row))
            .collect::<Result<Vec<_>, _>>()?;

        tracing::info!("获取团队日志列表成功: count = {}", logs.len());

        Ok(logs)
    }

    /// 获取单个团队的日志列表
    pub async fn get_team_logs(
        pool: &PgPool,
        team_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TeamLog>> {
        Self::list_team_logs(
            pool,
            Some(team_id),
            None,
            None,
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    /// 获取操作者的团队日志列表
    pub async fn get_operator_logs(
        pool: &PgPool,
        operator_id: u64,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TeamLog>> {
        Self::list_team_logs(
            pool,
            None,
            Some(operator_id),
            None,
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    /// 获取特定目标类型的团队日志列表
    pub async fn get_target_logs(
        pool: &PgPool,
        team_id: u64,
        target_type: &str,
        target_id: Option<u64>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Log_TeamLog>> {
        Self::list_team_logs(
            pool,
            Some(team_id),
            None,
            None,
            Some(target_type),
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    fn row_to_team_log(row: sqlx::postgres::PgRow) -> Result<Log_TeamLog> {
        let action_str: String = row.try_get("action")?;
        let action: LogAction = serde_json::from_str(&action_str)?;

        Ok(Log_TeamLog {
            log_id: row.try_get::<i64, _>("log_id")? as u64,
            team_id: row.try_get::<i64, _>("team_id")? as u64,
            operator_id: row.try_get::<i64, _>("operator_id")? as u64,
            action,
            target_type: row.try_get("target_type")?,
            target_id: row
                .try_get::<Option<i64>, _>("target_id")?
                .map(|id| id as u64),
            details: row.try_get("details")?,
            created_at: row.try_get::<i64, _>("created_at")?,
            ip_address: row.try_get("ip_address")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_team_log_crud() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        let test_user = crate::db::db_user::DbUser::create_user(
            &pool,
            "test_team_log_user",
            "TestPass123!",
            "test_team_log@example.com",
            "1380013801",
        )
        .await
        .unwrap();

        let test_team =
            crate::db::db_team::DbTeam::create_team(&pool, "test_team_log_team", test_user.user_id)
                .await
                .unwrap();

        let log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TeamCreated,
            "team",
            None,
            Some("团队创建成功"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();

        assert_eq!(log.team_id, test_team.team_id);
        assert!(matches!(log.action, LogAction::TeamCreated));
        assert_eq!(log.details, Some("团队创建成功".to_string()));

        let logs = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            Some(LogAction::TeamCreated),
            None,
            None,
            None,
            Some(10),
            None,
        )
        .await
        .unwrap();

        assert!(!logs.is_empty());
        assert_eq!(logs[0].team_id, test_team.team_id);

        let all_logs = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            None,
            None,
            None,
            None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        assert!(!all_logs.is_empty());

        let member_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::MemberJoined,
            "member",
            Some(test_user.user_id),
            Some("成员加入团队"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();

        assert!(matches!(member_log.action, LogAction::MemberJoined));
        assert_eq!(member_log.target_id, Some(test_user.user_id));

        // 测试 get_team_logs 便捷函数
        let team_logs = DbTeamLog::get_team_logs(&pool, test_team.team_id, Some(10), None)
            .await
            .unwrap();
        assert!(!team_logs.is_empty());
        assert_eq!(team_logs[0].team_id, test_team.team_id);

        // 测试 get_operator_logs 便捷函数
        let operator_logs = DbTeamLog::get_operator_logs(&pool, test_user.user_id, Some(10), None)
            .await
            .unwrap();
        assert!(!operator_logs.is_empty());
        assert_eq!(operator_logs[0].operator_id, test_user.user_id);

        // 测试更多 LogAction 类型
        let update_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TeamUpdated,
            "team",
            None,
            Some("团队信息更新"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(update_log.action, LogAction::TeamUpdated));

        let request_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::RequestApproved,
            "request",
            Some(12345),
            Some("入队申请通过"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(request_log.action, LogAction::RequestApproved));

        let task_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TaskCreated,
            "task",
            Some(99999),
            Some("团队任务创建"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(task_log.action, LogAction::TaskCreated));

        // 测试 get_target_logs 便捷函数
        let target_logs = DbTeamLog::get_target_logs(
            &pool,
            test_team.team_id,
            "task",
            Some(99999),
            Some(10),
            None,
        )
        .await
        .unwrap();
        assert!(!target_logs.is_empty());
        assert_eq!(target_logs[0].target_type, "task");

        // 测试 list_team_logs 多条件筛选
        let filtered_logs = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            Some(test_user.user_id),
            Some(LogAction::TaskCreated),
            Some("task"),
            None,
            None,
            Some(10),
            None,
        )
        .await
        .unwrap();
        assert!(!filtered_logs.is_empty());
        for log in &filtered_logs {
            assert_eq!(log.team_id, test_team.team_id);
            assert_eq!(log.operator_id, test_user.user_id);
            assert!(matches!(log.action, LogAction::TaskCreated));
        }

        crate::db::db_team::DbTeam::delete_team(&pool, test_team.team_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, test_user.user_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_team_log_all_actions() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        let test_user = crate::db::db_user::DbUser::create_user(
            &pool,
            "test_all_actions_user",
            "TestPass123!",
            "test_all_actions@example.com",
            "1380013802",
        )
        .await
        .unwrap();

        let test_team = crate::db::db_team::DbTeam::create_team(
            &pool,
            "test_all_actions_team",
            test_user.user_id,
        )
        .await
        .unwrap();

        let member_left_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::MemberLeft,
            "member",
            Some(test_user.user_id),
            Some("成员离开团队"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(member_left_log.action, LogAction::MemberLeft));

        let member_removed_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::MemberRemoved,
            "member",
            Some(test_user.user_id),
            Some("成员被移除"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(
            member_removed_log.action,
            LogAction::MemberRemoved
        ));

        let member_role_changed_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::MemberRoleChanged,
            "member",
            Some(test_user.user_id),
            Some("成员角色变更为管理员"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(
            member_role_changed_log.action,
            LogAction::MemberRoleChanged
        ));

        let team_closed_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TeamClosed,
            "team",
            None,
            Some("团队已关闭"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(team_closed_log.action, LogAction::TeamClosed));

        let subteam_created_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::SubTeamCreated,
            "subteam",
            Some(99999),
            Some("子团队创建成功"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(
            subteam_created_log.action,
            LogAction::SubTeamCreated
        ));

        let subteam_deleted_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::SubTeamDeleted,
            "subteam",
            Some(99999),
            Some("子团队已删除"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(
            subteam_deleted_log.action,
            LogAction::SubTeamDeleted
        ));

        let request_rejected_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::RequestRejected,
            "request",
            Some(88888),
            Some("入队申请被拒绝"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(
            request_rejected_log.action,
            LogAction::RequestRejected
        ));

        let task_completed_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TaskCompleted,
            "task",
            Some(77777),
            Some("任务已完成"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(
            task_completed_log.action,
            LogAction::TaskCompleted
        ));

        let task_deleted_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TaskDeleted,
            "task",
            Some(77777),
            Some("任务已删除"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        assert!(matches!(task_deleted_log.action, LogAction::TaskDeleted));

        let all_logs = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            None,
            None,
            None,
            None,
            Some(100),
            None,
        )
        .await
        .unwrap();

        let actions: Vec<LogAction> = all_logs.iter().map(|l| l.action.clone()).collect();
        assert!(actions.contains(&LogAction::MemberLeft));
        assert!(actions.contains(&LogAction::MemberRemoved));
        assert!(actions.contains(&LogAction::MemberRoleChanged));
        assert!(actions.contains(&LogAction::TeamClosed));
        assert!(actions.contains(&LogAction::SubTeamCreated));
        assert!(actions.contains(&LogAction::SubTeamDeleted));
        assert!(actions.contains(&LogAction::RequestRejected));
        assert!(actions.contains(&LogAction::TaskCompleted));
        assert!(actions.contains(&LogAction::TaskDeleted));

        crate::db::db_team::DbTeam::delete_team(&pool, test_team.team_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, test_user.user_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_team_log_pagination() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        let test_user = crate::db::db_user::DbUser::create_user(
            &pool,
            "test_pagination_user",
            "TestPass123!",
            "test_pagination@example.com",
            "1380013803",
        )
        .await
        .unwrap();

        let test_team = crate::db::db_team::DbTeam::create_team(
            &pool,
            "test_pagination_team",
            test_user.user_id,
        )
        .await
        .unwrap();

        for i in 0..15 {
            DbTeamLog::create_team_log(
                &pool,
                test_team.team_id,
                test_user.user_id,
                LogAction::TaskCreated,
                "task",
                Some(i as u64),
                Some("task"),
                Some("127.0.0.1"),
            )
            .await
            .unwrap();
        }

        let page1 = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            Some(LogAction::TaskCreated),
            None,
            None,
            None,
            Some(5),
            Some(0),
        )
        .await
        .unwrap();
        assert_eq!(page1.len(), 5);

        let page2 = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            Some(LogAction::TaskCreated),
            None,
            None,
            None,
            Some(5),
            Some(5),
        )
        .await
        .unwrap();
        assert_eq!(page2.len(), 5);

        let page3 = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            Some(LogAction::TaskCreated),
            None,
            None,
            None,
            Some(5),
            Some(10),
        )
        .await
        .unwrap();
        assert_eq!(page3.len(), 5);

        let all_count = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            Some(LogAction::TaskCreated),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(all_count.len(), 15);

        crate::db::db_team::DbTeam::delete_team(&pool, test_team.team_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, test_user.user_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_team_log_time_filter() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        let unique_suffix = chrono::Utc::now().timestamp_millis();
        let username = format!("test_time_filter_user_{}", unique_suffix);
        let teamname = format!("test_time_filter_team_{}", unique_suffix);
        let phone = format!("138{}{:04}", unique_suffix % 10000, unique_suffix % 10000);

        let test_user = crate::db::db_user::DbUser::create_user(
            &pool,
            &username,
            "TestPass123!",
            &format!("test_time_filter_{}@example.com", unique_suffix),
            &phone,
        )
        .await
        .unwrap();

        let test_team =
            crate::db::db_team::DbTeam::create_team(&pool, &teamname, test_user.user_id)
                .await
                .unwrap();

        let before_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TeamCreated,
            "team",
            None,
            Some("早期日志"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();
        let current_time = chrono::Utc::now().timestamp();

        let after_log = DbTeamLog::create_team_log(
            &pool,
            test_team.team_id,
            test_user.user_id,
            LogAction::TaskCreated,
            "task",
            None,
            Some("近期日志"),
            Some("127.0.0.1"),
        )
        .await
        .unwrap();

        let logs_after = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            None,
            None,
            Some(current_time),
            None,
            Some(10),
            None,
        )
        .await
        .unwrap();
        assert!(logs_after.iter().all(|l| l.created_at >= current_time));

        let logs_before = DbTeamLog::list_team_logs(
            &pool,
            Some(test_team.team_id),
            None,
            None,
            None,
            None,
            Some(current_time),
            Some(10),
            None,
        )
        .await
        .unwrap();
        assert!(logs_before.iter().all(|l| l.created_at <= current_time));

        crate::db::db_team::DbTeam::delete_team(&pool, test_team.team_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, test_user.user_id)
            .await
            .unwrap();
    }
}
