use crate::db::db_team::DbTeam;
use crate::models::team::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTeamRequest {
    pub team_name: String,
    pub team_description: Option<String>,
    pub team_visibility: Option<String>,
    pub team_member_limit: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTeamRequest {
    pub team_name: Option<String>,
    pub team_description: Option<String>,
    pub team_visibility: Option<String>,
    pub team_status: Option<String>,
    pub team_member_limit: Option<u16>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListTeamsQuery {
    pub leader_id: Option<u64>,
    pub user_id: Option<u64>,
}

pub struct TeamService;

impl TeamService {
    pub async fn create_team(
        pool: &PgPool,
        user_id: u64,
        request: CreateTeamRequest,
    ) -> Result<Team> {
        let team = DbTeam::create_team(pool, &request.team_name, user_id).await?;

        // Add creator as owner (level=255)
        DbTeam::add_team_member(pool, team.team_id, user_id, 255).await?;

        // If additional settings were provided, update them
        if request.team_description.is_some()
            || request.team_visibility.is_some()
            || request.team_member_limit.is_some()
        {
            let visibility = request
                .team_visibility
                .as_deref()
                .map(|v| match v {
                    "Public" => TeamVisibility::Public,
                    _ => TeamVisibility::Private,
                });

            let updated = DbTeam::update_team(
                pool,
                team.team_id,
                None,
                request.team_description.as_deref(),
                visibility,
                None,
                None,
                request.team_member_limit,
            )
            .await?;

            if let Some(t) = updated {
                return Ok(t);
            }
        }

        // Re-fetch to include members
        let team = DbTeam::get_team_by_id(pool, team.team_id)
            .await?
            .unwrap_or(team);

        Ok(team)
    }

    pub async fn get_team(pool: &PgPool, team_id: u64) -> Result<Option<Team>> {
        DbTeam::get_team_by_id(pool, team_id).await
    }

    pub async fn list_teams(
        pool: &PgPool,
        user_id: Option<u64>,
        leader_id: Option<u64>,
    ) -> Result<Vec<Team>> {
        DbTeam::list_teams(pool, leader_id, user_id).await
    }

    pub async fn update_team(
        pool: &PgPool,
        team_id: u64,
        request: UpdateTeamRequest,
    ) -> Result<Option<Team>> {
        let visibility = request.team_visibility.as_deref().map(|v| match v {
            "Public" => TeamVisibility::Public,
            _ => TeamVisibility::Private,
        });

        let status = request.team_status.as_deref().map(|s| match s {
            "Closed" => TeamStatus::Closed,
            _ => TeamStatus::Active,
        });

        DbTeam::update_team(
            pool,
            team_id,
            request.team_name.as_deref(),
            request.team_description.as_deref(),
            visibility,
            status,
            None,
            request.team_member_limit,
        )
        .await
    }

    pub async fn delete_team(pool: &PgPool, team_id: u64) -> Result<bool> {
        DbTeam::delete_team(pool, team_id).await
    }

    pub async fn add_member(
        pool: &PgPool,
        team_id: u64,
        user_id: u64,
        level: u8,
    ) -> Result<bool> {
        DbTeam::add_team_member(pool, team_id, user_id, level).await
    }

    pub async fn remove_member(pool: &PgPool, team_id: u64, user_id: u64) -> Result<bool> {
        DbTeam::remove_team_member(pool, team_id, user_id).await
    }

    pub async fn get_members(pool: &PgPool, team_id: u64) -> Result<Vec<Member>> {
        DbTeam::get_team_members(pool, team_id).await
    }

    pub async fn update_member_role(
        pool: &PgPool,
        team_id: u64,
        user_id: u64,
        level: u8,
    ) -> Result<bool> {
        DbTeam::update_member_role(pool, team_id, user_id, level).await
    }

    pub async fn check_membership(pool: &PgPool, team_id: u64, user_id: u64) -> Result<bool> {
        DbTeam::check_team_membership(pool, team_id, user_id).await
    }

    pub async fn create_invite(
        pool: &PgPool,
        team_id: u64,
        inviter_id: u64,
        invitee_ids: Vec<u64>,
        expire_hours: i64,
    ) -> Result<TeamInvite> {
        DbTeam::create_team_invite(pool, team_id, inviter_id, invitee_ids, expire_hours).await
    }

    pub async fn get_invites(pool: &PgPool, team_id: u64) -> Result<Vec<TeamInvite>> {
        DbTeam::get_team_invites(pool, Some(team_id), None).await
    }

    pub async fn create_join_request(
        pool: &PgPool,
        team_id: u64,
        user_id: u64,
    ) -> Result<JoinRequest> {
        DbTeam::create_join_request(pool, team_id, user_id).await
    }

    pub async fn get_join_requests(
        pool: &PgPool,
        team_id: Option<u64>,
        user_id: Option<u64>,
    ) -> Result<Vec<JoinRequest>> {
        DbTeam::get_join_requests(pool, team_id, user_id, None).await
    }

    pub async fn update_join_request_status(
        pool: &PgPool,
        request_id: u64,
        status: &str,
        reviewer_id: Option<u64>,
        review_message: Option<&str>,
    ) -> Result<bool> {
        DbTeam::update_join_request_status(pool, request_id, status, reviewer_id, review_message)
            .await
    }

    pub async fn get_team_logs(
        _pool: &PgPool,
        _team_id: u64,
    ) -> Result<Vec<serde_json::Value>> {
        Ok(vec![])
    }
}
