use crate::db::db_sub_team::DbSubTeam;
use crate::db::db_sub_team_member::DbSubTeamMember;
use crate::db::db_team::DbTeam;
use crate::models::team::{Member, SubTeam, Team};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubTeamRequest {
    pub sub_team_name: String,
    pub sub_team_leader_id: Option<u64>,
    pub sub_team_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubTeamRequest {
    pub sub_team_name: Option<String>,
    pub sub_team_leader_id: Option<u64>,
    pub sub_team_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddSubTeamMemberRequest {
    pub user_id: u64,
    pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubTeamMemberRequest {
    pub level: u8,
}

pub struct SubTeamService;

impl SubTeamService {
    pub async fn create_sub_team(
        pool: &PgPool,
        team_id: u64,
        user_id: u64,
        request: CreateSubTeamRequest,
    ) -> Result<SubTeam> {
        let leader_id = request.sub_team_leader_id.unwrap_or(user_id);
        let sub_team = DbSubTeam::create_sub_team(
            pool,
            &request.sub_team_name,
            leader_id,
            team_id,
            request.sub_team_description.as_deref(),
        )
        .await?;

        add_sub_team_to_parent(pool, team_id, sub_team.sub_team_id).await?;
        Ok(sub_team)
    }

    pub async fn get_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<Option<SubTeam>> {
        DbSubTeam::get_sub_team_by_id(pool, sub_team_id).await
    }

    pub async fn list_sub_teams(pool: &PgPool, team_id: u64) -> Result<Vec<SubTeam>> {
        DbSubTeam::list_sub_teams(pool, Some(team_id), None).await
    }

    pub async fn update_sub_team(
        pool: &PgPool,
        sub_team_id: u64,
        request: UpdateSubTeamRequest,
    ) -> Result<Option<SubTeam>> {
        DbSubTeam::update_sub_team(
            pool,
            sub_team_id,
            request.sub_team_name.as_deref(),
            request.sub_team_leader_id,
            request.sub_team_description.as_deref(),
        )
        .await
    }

    pub async fn delete_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<bool> {
        let sub_team = DbSubTeam::get_sub_team_by_id(pool, sub_team_id).await?;
        let deleted = DbSubTeam::delete_sub_team(pool, sub_team_id).await?;
        if deleted {
            if let Some(found) = sub_team {
                remove_sub_team_from_parent(pool, found.team_id, sub_team_id).await?;
            }
        }
        Ok(deleted)
    }

    pub async fn get_team_of_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<Option<Team>> {
        let sub_team = DbSubTeam::get_sub_team_by_id(pool, sub_team_id).await?;
        match sub_team {
            Some(sub_team) => TeamService::get_team(pool, sub_team.team_id).await,
            None => Ok(None),
        }
    }

    pub async fn list_members(pool: &PgPool, sub_team_id: u64) -> Result<Vec<Member>> {
        DbSubTeamMember::get_sub_team_members(pool, sub_team_id).await
    }

    pub async fn add_member(
        pool: &PgPool,
        sub_team_id: u64,
        request: AddSubTeamMemberRequest,
    ) -> Result<bool> {
        DbSubTeamMember::add_sub_team_member(pool, sub_team_id, request.user_id, request.level).await
    }

    pub async fn remove_member(pool: &PgPool, sub_team_id: u64, user_id: u64) -> Result<bool> {
        DbSubTeamMember::remove_sub_team_member(pool, sub_team_id, user_id).await
    }

    pub async fn update_member_level(
        pool: &PgPool,
        sub_team_id: u64,
        user_id: u64,
        request: UpdateSubTeamMemberRequest,
    ) -> Result<bool> {
        DbSubTeamMember::update_member_level(pool, sub_team_id, user_id, request.level).await
    }
}

async fn add_sub_team_to_parent(pool: &PgPool, team_id: u64, sub_team_id: u64) -> Result<()> {
    let team = DbTeam::get_team_by_id(pool, team_id).await?;
    let mut team = match team {
        Some(team) => team,
        None => return Ok(()),
    };
    if team.sub_team_ids.contains(&sub_team_id) {
        return Ok(());
    }

    team.sub_team_ids.push(sub_team_id);
    let sub_team_ids_json = serde_json::to_value(&team.sub_team_ids)?;

    let result = sqlx::query(
        r#"
        UPDATE teams
        SET sub_team_ids = $1
        WHERE team_id = $2
        "#,
    )
    .bind(sub_team_ids_json)
    .bind(team_id as i64)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(
            "附加子团队到父团队: team_id = {}, sub_team_id = {}",
            team_id,
            sub_team_id
        );
    }

    Ok(())
}

async fn remove_sub_team_from_parent(pool: &PgPool, team_id: u64, sub_team_id: u64) -> Result<()> {
    let team = DbTeam::get_team_by_id(pool, team_id).await?;
    let mut team = match team {
        Some(team) => team,
        None => return Ok(()),
    };
    let before = team.sub_team_ids.len();
    team.sub_team_ids.retain(|id| *id != sub_team_id);
    if team.sub_team_ids.len() == before {
        return Ok(());
    }

    let sub_team_ids_json = serde_json::to_value(&team.sub_team_ids)?;
    let result = sqlx::query(
        r#"
        UPDATE teams
        SET sub_team_ids = $1
        WHERE team_id = $2
        "#,
    )
    .bind(sub_team_ids_json)
    .bind(team_id as i64)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(
            "从父团队移除子团队: team_id = {}, sub_team_id = {}",
            team_id,
            sub_team_id
        );
    }

    Ok(())
}

use crate::services::team_service::TeamService;
