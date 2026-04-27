use crate::db::db_group::DbGroup;
use crate::db::db_group_member::DbGroupMember;
use crate::db::db_team::DbTeam;
use crate::models::team::{Group, Team};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub group_leader_id: Option<u64>,
    pub group_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub group_name: Option<String>,
    pub group_leader_id: Option<u64>,
    pub group_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddGroupMemberRequest {
    pub user_id: u64,
    pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupMemberRequest {
    pub level: u8,
}

pub struct GroupService;

impl GroupService {
    pub async fn create_group(
        pool: &PgPool,
        team_id: u64,
        user_id: u64,
        request: CreateGroupRequest,
    ) -> Result<Group> {
        let leader_id = request.group_leader_id.unwrap_or(user_id);
        let group = DbGroup::create_group(
            pool,
            &request.group_name,
            leader_id,
            team_id,
            request.group_description.as_deref(),
        )
        .await?;

        add_group_to_parent(pool, team_id, group.group_id).await?;
        Ok(group)
    }

    pub async fn get_group(pool: &PgPool, group_id: u64) -> Result<Option<Group>> {
        DbGroup::get_group_by_id(pool, group_id).await
    }

    pub async fn list_groups(pool: &PgPool, team_id: u64) -> Result<Vec<Group>> {
        DbGroup::list_groups(pool, Some(team_id), None).await
    }

    pub async fn update_group(
        pool: &PgPool,
        group_id: u64,
        request: UpdateGroupRequest,
    ) -> Result<Option<Group>> {
        DbGroup::update_group(
            pool,
            group_id,
            request.group_name.as_deref(),
            request.group_leader_id,
            request.group_description.as_deref(),
        )
        .await
    }

    pub async fn delete_group(pool: &PgPool, group_id: u64) -> Result<bool> {
        let group = DbGroup::get_group_by_id(pool, group_id).await?;
        let deleted = DbGroup::delete_group(pool, group_id).await?;
        if deleted {
            if let Some(found) = group {
                remove_group_from_parent(pool, found.team_id, group_id).await?;
            }
        }
        Ok(deleted)
    }

    pub async fn get_team_of_group(pool: &PgPool, group_id: u64) -> Result<Option<Team>> {
        let group = DbGroup::get_group_by_id(pool, group_id).await?;
        match group {
            Some(group) => TeamService::get_team(pool, group.team_id).await,
            None => Ok(None),
        }
    }

    pub async fn list_members(
        pool: &PgPool,
        group_id: u64,
    ) -> Result<Vec<crate::models::team::Member>> {
        DbGroupMember::get_group_members(pool, group_id).await
    }

    pub async fn add_member(
        pool: &PgPool,
        group_id: u64,
        request: AddGroupMemberRequest,
    ) -> Result<bool> {
        DbGroupMember::add_group_member(pool, group_id, request.user_id, request.level).await
    }

    pub async fn remove_member(pool: &PgPool, group_id: u64, user_id: u64) -> Result<bool> {
        DbGroupMember::remove_group_member(pool, group_id, user_id).await
    }

    pub async fn update_member_level(
        pool: &PgPool,
        group_id: u64,
        user_id: u64,
        request: UpdateGroupMemberRequest,
    ) -> Result<bool> {
        DbGroupMember::update_member_level(pool, group_id, user_id, request.level).await
    }

    /// 成员主动退出小组。组长不能调用此方法（须用 delete_group 解散）。
    pub async fn leave_group(pool: &PgPool, group_id: u64, user_id: u64) -> Result<bool> {
        // 1. 检查小组是否存在
        let group = DbGroup::get_group_by_id(pool, group_id).await?;
        let group = group.ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        // 2. 组长不能退出，只能解散小组
        if group.group_leader_id == user_id {
            return Err(anyhow::anyhow!(
                "Group leader cannot leave. Please delete the group instead."
            ));
        }

        // 3. 移除成员
        DbGroupMember::remove_group_member(pool, group_id, user_id).await
    }
}

async fn add_group_to_parent(pool: &PgPool, team_id: u64, group_id: u64) -> Result<()> {
    let team = DbTeam::get_team_by_id(pool, team_id).await?;
    let mut team = match team {
        Some(team) => team,
        None => return Ok(()),
    };
    if team.group_ids.contains(&group_id) {
        return Ok(());
    }

    team.group_ids.push(group_id);
    let group_ids_json = serde_json::to_value(&team.group_ids)?;

    let result = sqlx::query(
        r#"
        UPDATE teams
        SET group_ids = $1
        WHERE team_id = $2
        "#,
    )
    .bind(group_ids_json)
    .bind(team_id as i64)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(
            "附加小组到父团队: team_id = {}, group_id = {}",
            team_id,
            group_id
        );
    }

    Ok(())
}

async fn remove_group_from_parent(pool: &PgPool, team_id: u64, group_id: u64) -> Result<()> {
    let team = DbTeam::get_team_by_id(pool, team_id).await?;
    let mut team = match team {
        Some(team) => team,
        None => return Ok(()),
    };
    let before = team.group_ids.len();
    team.group_ids.retain(|id| *id != group_id);
    if team.group_ids.len() == before {
        return Ok(());
    }

    let group_ids_json = serde_json::to_value(&team.group_ids)?;
    let result = sqlx::query(
        r#"
        UPDATE teams
        SET group_ids = $1
        WHERE team_id = $2
        "#,
    )
    .bind(group_ids_json)
    .bind(team_id as i64)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(
            "从父团队移除小组: team_id = {}, group_id = {}",
            team_id,
            group_id
        );
    }

    Ok(())
}

use crate::services::team_service::TeamService;
