use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::pool::DbPool;
use crate::services::group_service::{
    AddGroupMemberRequest, CreateGroupRequest, GroupService, UpdateGroupMemberRequest,
    UpdateGroupRequest,
};

#[endpoint]
pub async fn create_group(
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

    let request: CreateGroupRequest = match req.parse_json().await {
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

    let user_id = request.group_leader_id.unwrap_or(actor_id);

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::create_group(pool, team_id, user_id, request).await {
        Ok(group) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Group created successfully",
                "group": serde_json::to_value(&group).unwrap_or_default()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create group",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn list_groups(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let team_id: u64 = team_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::list_groups(pool, team_id).await {
        Ok(groups) => {
            let groups: Vec<serde_json::Value> = groups
                .into_iter()
                .map(|group| serde_json::to_value(group).unwrap_or_default())
                .collect();
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "groups": groups
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to list groups",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_group(group_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let group_id: u64 = group_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::get_group(pool, group_id).await {
        Ok(Some(group)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "group": serde_json::to_value(&group).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Group not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to get group",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_group(
    group_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let group_id: u64 = group_id.into_inner();
    let request: UpdateGroupRequest = match req.parse_json().await {
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

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::update_group(pool, group_id, request).await {
        Ok(Some(group)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Group updated successfully",
                "group": serde_json::to_value(&group).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Group not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update group",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn delete_group(group_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let group_id: u64 = group_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::delete_group(pool, group_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Group deleted successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Group not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to delete group",
                "message": e.to_string()
            })));
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AddGroupMemberRequestBody {
    pub user_id: u64,
    pub level: u8,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UpdateGroupMemberLevel {
    pub level: u8,
}

#[endpoint]
pub async fn list_group_members(group_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let group_id: u64 = group_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::list_members(pool, group_id).await {
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
                "error": "Failed to fetch group members",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn add_group_member(
    group_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let group_id: u64 = group_id.into_inner();

    let request: AddGroupMemberRequestBody = match req.parse_json().await {
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

    let service_request = AddGroupMemberRequest {
        user_id: request.user_id,
        level: request.level,
    };

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::add_member(pool, group_id, service_request).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Group member added successfully"
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
                "error": "Failed to add group member",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn remove_group_member(
    group_id: PathParam<u64>,
    user_id: PathParam<u64>,
    depot: &mut Depot,
    res: &mut Response,
) {
    let group_id: u64 = group_id.into_inner();
    let user_id: u64 = user_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::remove_member(pool, group_id, user_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Group member removed successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Group member not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to remove group member",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_group_member_level(
    group_id: PathParam<u64>,
    user_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let group_id: u64 = group_id.into_inner();
    let user_id: u64 = user_id.into_inner();

    let request: UpdateGroupMemberLevel = match req.parse_json().await {
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

    let service_request = UpdateGroupMemberRequest {
        level: request.level,
    };

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match GroupService::update_member_level(pool, group_id, user_id, service_request).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Group member level updated successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Group member not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to update group member level",
                "message": e.to_string()
            })));
        }
    }
}
