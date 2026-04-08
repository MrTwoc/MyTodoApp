use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::pool::DbPool;
use crate::services::team_service::{
    CreateTeamRequest, ListTeamsQuery, TeamService, UpdateTeamRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: u64,
    pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInviteRequest {
    pub invitee_ids: Vec<u64>,
    pub expire_hours: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateJoinRequestBody {
    pub status: String,
    pub reviewer_id: Option<u64>,
    pub review_message: Option<String>,
}

#[endpoint]
pub async fn create_team(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let user_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => *id as u64,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "User not authenticated"
            })));
            return;
        }
    };

    let request: CreateTeamRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": msg
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::create_team(pool, user_id, request).await {
        Ok(team) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Team created successfully",
                "team": serde_json::to_value(&team).unwrap_or_default()
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create team",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn get_team(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::get_team(pool, team_id).await {
        Ok(Some(team)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "team": serde_json::to_value(&team).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Team not found"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch team",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn list_teams(depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let auth_user_id: Option<u64> = depot.get::<i64>("user_id").ok().map(|id| *id as u64);

    let query: ListTeamsQuery = req.parse_queries().unwrap_or_default();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    // Query params override depot user_id for filtering
    let filter_user_id = query.user_id.or(auth_user_id);

    match TeamService::list_teams(pool, filter_user_id, query.leader_id).await {
        Ok(teams) => {
            let total = teams.len() as u32;
            let teams_json: Vec<serde_json::Value> = teams
                .iter()
                .map(|t| serde_json::to_value(t).unwrap_or_default())
                .collect();
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "teams": teams_json,
                "total": total
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch teams",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn update_team(team_id: PathParam<u64>, req: &mut Request, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let request: UpdateTeamRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": msg
            })));
            return;
        }
    };

    let pool = match crate::db::pool::create_pool().await {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Database connection failed",
                "message": msg
            })));
            return;
        }
    };

    match TeamService::update_team(&pool, team_id, request).await {
        Ok(Some(team)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Team updated successfully",
                "team": serde_json::to_value(&team).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Team not found"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update team",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn delete_team(team_id: PathParam<u64>, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let pool = match crate::db::pool::create_pool().await {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Database connection failed",
                "message": msg
            })));
            return;
        }
    };

    match TeamService::delete_team(&pool, team_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Team deleted successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Team not found"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to delete team",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn add_member(team_id: PathParam<u64>, req: &mut Request, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let request: AddMemberRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": msg
            })));
            return;
        }
    };

    let pool = match crate::db::pool::create_pool().await {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Database connection failed",
                "message": msg
            })));
            return;
        }
    };

    match TeamService::add_member(&pool, team_id, request.user_id, request.level).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Member added successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::CONFLICT);
            res.render(Json(serde_json::json!({
                "error": "Member already exists"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to add member",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn remove_member(team_id: PathParam<u64>, user_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();
    let user_id: u64 = user_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::remove_member(pool, team_id, user_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Member removed successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Member not found"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to remove member",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn update_member_role(
    team_id: PathParam<u64>,
    user_id: PathParam<u64>,
    depot: &mut Depot,
    req: &mut Request,
    res: &mut Response,
) {
    let team_id: u64 = team_id.into_inner();
    let target_user_id: u64 = user_id.into_inner();

    let current_user_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => *id as u64,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "User not authenticated"
            })));
            return;
        }
    };

    let request: UpdateRoleRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": msg
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::update_member_role(pool, team_id, current_user_id, target_user_id, request.level).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Member role updated successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Member not found"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Forbidden") {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": msg
                })));
            } else {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(serde_json::json!({
                    "error": "Failed to update member role",
                    "message": msg
                })));
            }
        }
    }
}

#[endpoint]
pub async fn get_members(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::get_members(pool, team_id).await {
        Ok(members) => {
            let members_json: Vec<serde_json::Value> = members
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "members": members_json
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch members",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn create_invite(
    team_id: PathParam<u64>,
    depot: &mut Depot,
    req: &mut Request,
    res: &mut Response,
) {
    let team_id: u64 = team_id.into_inner();

    let inviter_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => *id as u64,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "User not authenticated"
            })));
            return;
        }
    };

    let request: CreateInviteRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": msg
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::create_invite(
        pool,
        team_id,
        inviter_id,
        request.invitee_ids,
        request.expire_hours,
    )
    .await
    {
        Ok(invite) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Invite created successfully",
                "invite": serde_json::to_value(&invite).unwrap_or_default()
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create invite",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn create_join_request(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let user_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => *id as u64,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "User not authenticated"
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::create_join_request(pool, team_id, user_id).await {
        Ok(join_req) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Join request created successfully",
                "request": serde_json::to_value(&join_req).unwrap_or_default()
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create join request",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn update_join_request_status(
    team_id: PathParam<u64>,
    request_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let _team_id: u64 = team_id.into_inner();
    let request_id: u64 = request_id.into_inner();

    let body: UpdateJoinRequestBody = match req.parse_json().await {
        Ok(b) => b,
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": msg
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::update_join_request_status(
        pool,
        request_id,
        &body.status,
        body.reviewer_id,
        body.review_message.as_deref(),
    )
    .await
    {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Join request status updated successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Join request not found"
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to update join request status",
                "message": msg
            })));
        }
    }
}

#[endpoint]
pub async fn get_team_logs(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match TeamService::get_team_logs(pool, team_id).await {
        Ok(logs) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "logs": logs
            })));
        }
        Err(e) => {
            let msg = e.to_string();
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch team logs",
                "message": msg
            })));
        }
    }
}
