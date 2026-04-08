use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::pool::DbPool;
use crate::services::sub_team_service::{
    AddSubTeamMemberRequest, CreateSubTeamRequest, SubTeamService, UpdateSubTeamMemberRequest,
    UpdateSubTeamRequest,
};

#[endpoint]
pub async fn create_sub_team(
    team_id: PathParam<u64>,
    depot: &mut Depot,
    req: &mut Request,
    res: &mut Response,
) {
    let team_id: u64 = team_id.into_inner();
    let actor_id = match depot.get::<i64>("user_id").ok() {
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

    let request: CreateSubTeamRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": e.to_string()
            })));
            return;
        }
    };

    let user_id = request.sub_team_leader_id.unwrap_or(actor_id);

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::create_sub_team(pool, team_id, user_id, request).await {
        Ok(sub_team) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Sub team created successfully",
                "sub_team": serde_json::to_value(&sub_team).unwrap_or_default()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create sub team",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn list_sub_teams(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::list_sub_teams(pool, team_id).await {
        Ok(sub_teams) => {
            let sub_teams: Vec<serde_json::Value> = sub_teams
                .into_iter()
                .map(|sub_team| serde_json::to_value(sub_team).unwrap_or_default())
                .collect();
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "sub_teams": sub_teams
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to list sub teams",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_sub_team(sub_team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let sub_team_id: u64 = sub_team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::get_sub_team(pool, sub_team_id).await {
        Ok(Some(sub_team)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "sub_team": serde_json::to_value(sub_team).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub team not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to get sub team",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_sub_team(
    sub_team_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let sub_team_id: u64 = sub_team_id.into_inner();
    let request: UpdateSubTeamRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": e.to_string()
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::update_sub_team(pool, sub_team_id, request).await {
        Ok(Some(sub_team)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub team updated successfully",
                "sub_team": serde_json::to_value(sub_team).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub team not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update sub team",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn delete_sub_team(sub_team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let sub_team_id: u64 = sub_team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::delete_sub_team(pool, sub_team_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub team deleted successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub team not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to delete sub team",
                "message": e.to_string()
            })));
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AddSubTeamMemberRequestBody {
    pub user_id: u64,
    pub level: u8,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateSubTeamMemberLevel {
    pub level: u8,
}

#[endpoint]
pub async fn list_sub_team_members(sub_team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let sub_team_id: u64 = sub_team_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::list_members(pool, sub_team_id).await {
        Ok(members) => {
            let members: Vec<serde_json::Value> = members
                .into_iter()
                .map(|member| serde_json::to_value(member).unwrap_or_default())
                .collect();
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "members": members
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch sub team members",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn add_sub_team_member(
    sub_team_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let sub_team_id: u64 = sub_team_id.into_inner();

    let request: AddSubTeamMemberRequestBody = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": e.to_string()
            })));
            return;
        }
    };

    let service_request = AddSubTeamMemberRequest {
        user_id: request.user_id,
        level: request.level,
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::add_member(pool, sub_team_id, service_request).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub team member added successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::CONFLICT);
            res.render(Json(serde_json::json!({
                "error": "Member already exists"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to add sub team member",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn remove_sub_team_member(sub_team_id: PathParam<u64>, user_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let sub_team_id: u64 = sub_team_id.into_inner();
    let user_id: u64 = user_id.into_inner();

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::remove_member(pool, sub_team_id, user_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub team member removed successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub team member not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to remove sub team member",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_sub_team_member_level(
    sub_team_id: PathParam<u64>,
    user_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let sub_team_id: u64 = sub_team_id.into_inner();
    let user_id: u64 = user_id.into_inner();

    let request: UpdateSubTeamMemberLevel = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": e.to_string()
            })));
            return;
        }
    };

    let service_request = UpdateSubTeamMemberRequest {
        level: request.level,
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::update_member_level(pool, sub_team_id, user_id, service_request).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub team member level updated successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub team member not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to update sub team member level",
                "message": e.to_string()
            })));
        }
    }
}
