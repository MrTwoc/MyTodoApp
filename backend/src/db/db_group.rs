use crate::models::team::Group;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbGroup;

impl DbGroup {
    pub async fn create_group(
        pool: &PgPool,
        group_name: &str,
        group_leader_id: u64,
        team_id: u64,
        group_description: Option<&str>,
    ) -> Result<Group> {
        let group_id = crate::utils::id_generator::generate_team_id();
        let group_create_time = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO groups (group_id, group_name, group_leader_id, team_id, group_create_time, group_description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING group_id, group_name, group_leader_id, team_id, group_create_time, group_description
            "#,
        )
        .bind(group_id as i64)
        .bind(group_name)
        .bind(group_leader_id as i64)
        .bind(team_id as i64)
        .bind(group_create_time)
        .bind(group_description)
        .fetch_one(pool)
        .await?;

        tracing::info!(
            "创建小组成功: group_id = {}, group_name = {}",
            group_id,
            group_name
        );

        Ok(Self::row_to_group(result)?)
    }

    pub async fn get_group_by_id(pool: &PgPool, group_id: u64) -> Result<Option<Group>> {
        let result = sqlx::query(
            r#"
            SELECT group_id, group_name, group_leader_id, team_id, group_create_time, group_description
            FROM groups
            WHERE group_id = $1
            "#,
        )
        .bind(group_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => {
                let mut group = Self::row_to_group(row)?;

                // Load members from group_members table
                let members =
                    crate::db::db_group_member::DbGroupMember::get_group_members(pool, group_id)
                        .await?;

                group.group_members = members;

                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    pub async fn list_groups(
        pool: &PgPool,
        team_id: Option<u64>,
        group_leader_id: Option<u64>,
    ) -> Result<Vec<Group>> {
        let mut query = String::from(
            "SELECT group_id, group_name, group_leader_id, team_id, group_create_time, group_description FROM groups WHERE 1=1",
        );
        let mut bind_idx = 1;

        if team_id.is_some() {
            query.push_str(&format!(" AND team_id = ${}", bind_idx));
            bind_idx += 1;
        }
        if group_leader_id.is_some() {
            query.push_str(&format!(" AND group_leader_id = ${}", bind_idx));
        }

        let mut sql_query = sqlx::query(&query);

        if let Some(tid) = team_id {
            sql_query = sql_query.bind(tid as i64);
        }
        if let Some(lid) = group_leader_id {
            sql_query = sql_query.bind(lid as i64);
        }

        let rows = sql_query.fetch_all(pool).await?;
        let mut groups = Vec::new();

        for row in rows {
            let mut group = Self::row_to_group(row)?;

            // Load members from group_members table
            let members =
                crate::db::db_group_member::DbGroupMember::get_group_members(pool, group.group_id)
                    .await?;

            group.group_members = members;

            groups.push(group);
        }

        Ok(groups)
    }

    pub async fn update_group(
        pool: &PgPool,
        group_id: u64,
        group_name: Option<&str>,
        group_leader_id: Option<u64>,
        group_description: Option<&str>,
    ) -> Result<Option<Group>> {
        let existing = Self::get_group_by_id(pool, group_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let existing = existing.unwrap();
        let new_name = group_name.unwrap_or(&existing.group_name);
        let new_leader_id = group_leader_id
            .map(|id| id as i64)
            .unwrap_or(existing.group_leader_id as i64);
        let new_description = group_description.or(existing.group_description.as_deref());

        let result = sqlx::query(
            r#"
            UPDATE groups
            SET group_name = $1, group_leader_id = $2, group_description = $3
            WHERE group_id = $4
            RETURNING group_id, group_name, group_leader_id, team_id, group_create_time, group_description
            "#,
        )
        .bind(new_name)
        .bind(new_leader_id)
        .bind(new_description)
        .bind(group_id as i64)
        .fetch_one(pool)
        .await?;

        tracing::info!("更新小组成功: group_id = {}", group_id);

        let mut group = Self::row_to_group(result)?;

        // Load members from group_members table
        let members =
            crate::db::db_group_member::DbGroupMember::get_group_members(pool, group_id).await?;

        group.group_members = members;

        Ok(Some(group))
    }

    pub async fn delete_group(pool: &PgPool, group_id: u64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM groups WHERE group_id = $1")
            .bind(group_id as i64)
            .execute(pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            tracing::info!("删除小组成功: group_id = {}", group_id);
        }

        Ok(deleted)
    }

    fn row_to_group(row: sqlx::postgres::PgRow) -> Result<Group> {
        Ok(Group {
            group_id: row.get::<i64, _>("group_id") as u64,
            group_name: row.get("group_name"),
            group_leader_id: row.get::<i64, _>("group_leader_id") as u64,
            team_id: row.get::<i64, _>("team_id") as u64,
            group_create_time: row.get("group_create_time"),
            group_description: row.get("group_description"),
            group_members: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_group_crud() {
        let pool = crate::db::pool::create_pool().await.unwrap();

        let timestamp = chrono::Utc::now().timestamp();
        let unique_suffix = timestamp.to_string();

        let leader = crate::db::db_user::DbUser::create_user(
            &pool,
            &format!("test_group_leader_{}", unique_suffix),
            "TestPass123!",
            &format!("test_group_leader_{}@example.com", unique_suffix),
            &format!("1380000{:04}", timestamp % 10000),
        )
        .await
        .unwrap();

        let team_leader = crate::db::db_user::DbUser::create_user(
            &pool,
            &format!("test_team_leader_for_group_{}", unique_suffix),
            "TestPass123!",
            &format!("test_team_leader_for_group_{}@example.com", unique_suffix),
            &format!("1380001{:04}", timestamp % 10000),
        )
        .await
        .unwrap();

        let team =
            crate::db::db_team::DbTeam::create_team(&pool, "Test Parent Team", team_leader.user_id)
                .await
                .unwrap();

        let group = DbGroup::create_group(
            &pool,
            "Test Group",
            leader.user_id,
            team.team_id,
            Some("Test group description"),
        )
        .await
        .unwrap();

        println!("创建小组: {:?}", group);
        assert_eq!(group.group_name, "Test Group");
        assert_eq!(group.group_leader_id, leader.user_id);
        assert_eq!(group.team_id, team.team_id);

        let found = DbGroup::get_group_by_id(&pool, group.group_id)
            .await
            .unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.group_id, group.group_id);
        assert_eq!(found.group_name, "Test Group");

        let list = DbGroup::list_groups(&pool, Some(team.team_id), None)
            .await
            .unwrap();
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);

        let updated = DbGroup::update_group(
            &pool,
            group.group_id,
            Some("Updated Group Name"),
            None,
            Some("Updated description"),
        )
        .await
        .unwrap();
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().group_name, "Updated Group Name");

        let deleted = DbGroup::delete_group(&pool, group.group_id).await.unwrap();
        assert!(deleted);

        let found_after_delete = DbGroup::get_group_by_id(&pool, group.group_id)
            .await
            .unwrap();
        assert!(found_after_delete.is_none());

        crate::db::db_team::DbTeam::delete_team(&pool, team.team_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, leader.user_id)
            .await
            .unwrap();
        crate::db::db_user::DbUser::delete_user(&pool, team_leader.user_id)
            .await
            .unwrap();
    }
}
