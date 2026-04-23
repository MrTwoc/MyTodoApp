use crate::models::team::Member;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbGroupMember;

impl DbGroupMember {
    pub async fn add_group_member(
        pool: &PgPool,
        group_id: u64,
        user_id: u64,
        level: u8,
    ) -> Result<bool> {
        let join_time = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO group_members (group_id, user_id, level, join_time)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (group_id, user_id) DO NOTHING
            "#,
        )
        .bind(group_id as i64)
        .bind(user_id as i64)
        .bind(level as i32)
        .bind(join_time)
        .execute(pool)
        .await?;

        let inserted = result.rows_affected() > 0;
        if inserted {
            tracing::info!(
                "添加小组成员成功: group_id = {}, user_id = {}",
                group_id,
                user_id
            );
        }

        Ok(inserted)
    }

    pub async fn remove_group_member(pool: &PgPool, group_id: u64, user_id: u64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id as i64)
            .bind(user_id as i64)
            .execute(pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            tracing::info!(
                "移除小组成员成功: group_id = {}, user_id = {}",
                group_id,
                user_id
            );
        }

        Ok(deleted)
    }

    pub async fn get_group_members(pool: &PgPool, group_id: u64) -> Result<Vec<Member>> {
        let rows = sqlx::query(
            r#"
            SELECT group_id, user_id, level, join_time
            FROM group_members
            WHERE group_id = $1
            "#,
        )
        .bind(group_id as i64)
        .fetch_all(pool)
        .await?;

        let mut members = Vec::new();
        for row in rows {
            members.push(Member {
                team_id: None,
                group_id: Some(row.get::<i64, _>("group_id") as u64),
                user_id: row.get::<i64, _>("user_id") as u64,
                username: None,
                level: row.get::<i32, _>("level") as u8,
                join_time: row.get("join_time"),
            });
        }

        Ok(members)
    }

    pub async fn update_member_level(
        pool: &PgPool,
        group_id: u64,
        user_id: u64,
        new_level: u8,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE group_members
            SET level = $1
            WHERE group_id = $2 AND user_id = $3
            "#,
        )
        .bind(new_level as i32)
        .bind(group_id as i64)
        .bind(user_id as i64)
        .execute(pool)
        .await?;

        let updated = result.rows_affected() > 0;
        if updated {
            tracing::info!(
                "更新小组成员级别成功: group_id = {}, user_id = {}, new_level = {}",
                group_id,
                user_id,
                new_level
            );
        }

        Ok(updated)
    }

    pub async fn is_member(pool: &PgPool, group_id: u64, user_id: u64) -> Result<bool> {
        let result =
            sqlx::query("SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2")
                .bind(group_id as i64)
                .bind(user_id as i64)
                .fetch_optional(pool)
                .await?;

        Ok(result.is_some())
    }

    pub async fn get_member_level(
        pool: &PgPool,
        group_id: u64,
        user_id: u64,
    ) -> Result<Option<u8>> {
        let result =
            sqlx::query("SELECT level FROM group_members WHERE group_id = $1 AND user_id = $2")
                .bind(group_id as i64)
                .bind(user_id as i64)
                .fetch_optional(pool)
                .await?;

        match result {
            Some(row) => Ok(Some(row.get::<i32, _>("level") as u8)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_group_member_operations() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        let timestamp = chrono::Utc::now().timestamp();
        let unique_suffix = timestamp.to_string();

        let leader = crate::db::db_user::DbUser::create_user(
            &pool,
            &format!("test_group_member_leader_{}", unique_suffix),
            "TestPass123!",
            &format!("test_group_member_leader_{}@example.com", unique_suffix),
            &format!("1380000{:04}", timestamp % 10000),
        )
        .await
        .unwrap();

        let member = crate::db::db_user::DbUser::create_user(
            &pool,
            &format!("test_group_member_user_{}", unique_suffix),
            "TestPass123!",
            &format!("test_group_member_user_{}@example.com", unique_suffix),
            &format!("1380001{:04}", timestamp % 10000),
        )
        .await
        .unwrap();

        let team_leader = crate::db::db_user::DbUser::create_user(
            &pool,
            &format!("test_team_for_group_member_{}", unique_suffix),
            "TestPass123!",
            &format!("test_team_for_group_member_{}@example.com", unique_suffix),
            &format!("1380002{:04}", timestamp % 10000),
        )
        .await
        .unwrap();

        let team = crate::db::db_team::DbTeam::create_team(
            &pool,
            "Test Team For Group Member",
            team_leader.user_id,
        )
        .await
        .unwrap();

        let group = crate::db::db_group::DbGroup::create_group(
            &pool,
            "Test Group For Member",
            leader.user_id,
            team.team_id,
            Some("Test group for member operations"),
        )
        .await
        .unwrap();

        let added = DbGroupMember::add_group_member(&pool, group.group_id, member.user_id, 2)
            .await
            .unwrap();
        assert!(added);

        let is_member_check = DbGroupMember::is_member(&pool, group.group_id, member.user_id)
            .await
            .unwrap();
        assert!(is_member_check);

        let level = DbGroupMember::get_member_level(&pool, group.group_id, member.user_id)
            .await
            .unwrap();
        assert!(level.is_some());
        assert_eq!(level.unwrap(), 2);

        let members = DbGroupMember::get_group_members(&pool, group.group_id)
            .await
            .unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].user_id, member.user_id);

        let updated = DbGroupMember::update_member_level(&pool, group.group_id, member.user_id, 5)
            .await
            .unwrap();
        assert!(updated);

        let new_level = DbGroupMember::get_member_level(&pool, group.group_id, member.user_id)
            .await
            .unwrap();
        assert_eq!(new_level.unwrap(), 5);

        let removed = DbGroupMember::remove_group_member(&pool, group.group_id, member.user_id)
            .await
            .unwrap();
        assert!(removed);

        let is_member_after = DbGroupMember::is_member(&pool, group.group_id, member.user_id)
            .await
            .unwrap();
        assert!(!is_member_after);

        let members_after = DbGroupMember::get_group_members(&pool, group.group_id)
            .await
            .unwrap();
        assert!(members_after.is_empty());

        crate::db::db_group::DbGroup::delete_group(&pool, group.group_id)
            .await
            .unwrap();
        crate::db::db_team::DbTeam::delete_team(&pool, team.team_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, leader.user_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, member.user_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, team_leader.user_id)
            .await
            .unwrap();
    }
}
