use salvo::{handler, prelude::Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemRole {
    Owner,  // 系统所有者/超级管理员，拥有最高权限
    Admin,  // 系统管理员，拥有较高权限
    Member, // 普通成员，基本权限
}

impl SystemRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Owner => "owner".to_string(),
            Self::Admin => "admin".to_string(),
            Self::Member => "member".to_string(),
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Self::Owner => 255,
            Self::Admin => 128,
            Self::Member => 1,
        }
    }

    pub fn has_permission(&self, required_level: u8) -> bool {
        self.level() >= required_level
    }
}

const AUTH_HEADER: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

#[handler]
pub async fn require_auth(
    req: &mut salvo::Request,
    depot: &mut salvo::Depot,
    res: &mut salvo::Response,
    ctrl: &mut salvo::FlowCtrl,
) {
    let token = extract_token(req);

    match token {
        Some(token) => match crate::utils::jwt::verify_access_token(&token) {
            Ok(token_data) => {
                let claims = token_data.claims;
                depot.insert("user_id", claims.user_id);
                depot.insert("username", claims.username);
                depot.insert("role", claims.role);
                ctrl.call_next(req, depot, res).await;
            }
            Err(e) => {
                tracing::warn!("Token verification failed: {:?}", e);
                unauthorized_response(res);
                ctrl.skip_rest();
            }
        },
        None => {
            tracing::warn!("Missing Authorization header");
            unauthorized_response(res);
            ctrl.skip_rest();
        }
    }
}

fn extract_token(req: &salvo::Request) -> Option<String> {
    let auth_header = req.header::<String>(AUTH_HEADER)?;
    if auth_header.starts_with(BEARER_PREFIX) {
        Some(auth_header[BEARER_PREFIX.len()..].to_string())
    } else {
        None
    }
}

fn unauthorized_response(res: &mut salvo::Response) {
    use salvo::http::StatusCode;
    use salvo::prelude::Json;
    res.status_code(StatusCode::UNAUTHORIZED);
    res.render(Json(serde_json::json!({
        "error": "Unauthorized",
        "message": "Missing or invalid authentication token"
    })));
}

impl Default for SystemRole {
    fn default() -> Self {
        Self::Member
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamRole {
    Owner,  // 团队所有者，拥有最高权限，可以管理团队所有内容
    Admin,  // 团队管理员，可以管理团队成员和任务
    Member, // 普通成员，基本的任务操作权限
}

impl TeamRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Owner => "owner".to_string(),
            Self::Admin => "admin".to_string(),
            Self::Member => "member".to_string(),
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Self::Owner => 255,
            Self::Admin => 128,
            Self::Member => 1,
        }
    }

    pub fn has_permission(&self, required_level: u8) -> bool {
        self.level() >= required_level
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_manage_tasks(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member)
    }

    pub fn can_delete_team(&self) -> bool {
        matches!(self, Self::Owner)
    }

    pub fn can_transfer_ownership(&self) -> bool {
        matches!(self, Self::Owner)
    }
}

impl Default for TeamRole {
    fn default() -> Self {
        Self::Member
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleLevel {
    pub level: u8,
    pub name: String,
}

impl RoleLevel {
    pub fn new(level: u8, name: String) -> Self {
        Self { level, name }
    }

    pub fn has_permission(&self, required_level: u8) -> bool {
        self.level >= required_level
    }

    pub fn can_manage(&self, other: &RoleLevel) -> bool {
        self.level > other.level
    }
}

pub fn is_owner(level: u8) -> bool {
    level == 255
}

pub fn is_admin(level: u8) -> bool {
    level >= 128
}

pub fn is_member(level: u8) -> bool {
    level >= 1
}

pub fn compare_roles(role1: &str, role2: &str) -> Option<std::cmp::Ordering> {
    let level1 = TeamRole::from_str(role1).map(|r| r.level());
    let level2 = TeamRole::from_str(role2).map(|r| r.level());

    match (level1, level2) {
        (Some(l1), Some(l2)) => Some(l1.cmp(&l2)),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    User,
    Task,
    Team,
    SubTeam,
}

pub struct PermissionService;

impl PermissionService {
    pub fn check_user_ownership(user_id: u64, resource_owner_id: u64) -> bool {
        user_id == resource_owner_id
    }

    pub fn check_task_ownership(user_id: u64, task_leader_id: u64) -> bool {
        user_id == task_leader_id
    }

    pub fn check_team_ownership(user_id: u64, team_leader_id: u64) -> bool {
        user_id == team_leader_id
    }

    pub fn check_sub_team_ownership(user_id: u64, sub_team_leader_id: u64) -> bool {
        user_id == sub_team_leader_id
    }

    pub fn check_resource_ownership(
        resource_type: ResourceType,
        user_id: u64,
        resource_owner_id: u64,
    ) -> bool {
        match resource_type {
            ResourceType::User => Self::check_user_ownership(user_id, resource_owner_id),
            ResourceType::Task => Self::check_task_ownership(user_id, resource_owner_id),
            ResourceType::Team => Self::check_team_ownership(user_id, resource_owner_id),
            ResourceType::SubTeam => Self::check_sub_team_ownership(user_id, resource_owner_id),
        }
    }

    pub fn can_access_team(
        user_id: u64,
        team_leader_id: u64,
        team_members: &[crate::models::team::Member],
    ) -> bool {
        if Self::check_team_ownership(user_id, team_leader_id) {
            return true;
        }
        team_members.iter().any(|m| m.user_id == user_id)
    }

    pub fn can_manage_team_member(requester_role: TeamRole, target_role: TeamRole) -> bool {
        if requester_role.can_manage_members() {
            let requester_level = requester_role.level();
            let target_level = target_role.level();
            requester_level > target_level
        } else {
            false
        }
    }

    pub fn can_assign_task(
        user_id: u64,
        team_id: u64,
        team_leader_id: u64,
        team_members: &[crate::models::team::Member],
    ) -> bool {
        if Self::check_team_ownership(user_id, team_leader_id) {
            return true;
        }
        team_members.iter().any(|m| {
            m.user_id == user_id
                && m.team_id.map(|id| id == team_id).unwrap_or(false)
                && m.level >= 1
        })
    }

    pub fn can_create_task(
        user_id: u64,
        team_id: Option<u64>,
        team_leader_id: u64,
        team_members: &[crate::models::team::Member],
    ) -> bool {
        if let Some(tid) = team_id {
            if Self::check_team_ownership(user_id, team_leader_id) {
                return true;
            }
            team_members.iter().any(|m| {
                m.user_id == user_id
                    && m.team_id.map(|id| id == tid).unwrap_or(false)
                    && m.level >= 1
            })
        } else {
            true
        }
    }

    pub fn can_update_task(user_id: u64, task_leader_id: u64, team_leader_id: u64) -> bool {
        if Self::check_task_ownership(user_id, task_leader_id) {
            return true;
        }
        Self::check_team_ownership(user_id, team_leader_id)
    }

    pub fn can_delete_task(user_id: u64, task_leader_id: u64, team_leader_id: u64) -> bool {
        Self::check_team_ownership(user_id, team_leader_id)
            || Self::check_task_ownership(user_id, task_leader_id)
    }

    pub fn has_team_access(
        user_id: u64,
        team_id: u64,
        team_leader_id: u64,
        team_members: &[crate::models::team::Member],
    ) -> Result<bool, &'static str> {
        if Self::check_team_ownership(user_id, team_leader_id) {
            return Ok(true);
        }
        let is_member = team_members
            .iter()
            .any(|m| m.user_id == user_id && m.team_id.map(|id| id == team_id).unwrap_or(false));
        if is_member {
            Ok(true)
        } else {
            Err("User is not a member of this team")
        }
    }

    pub fn has_sub_team_access(
        user_id: u64,
        sub_team_id: u64,
        sub_team_leader_id: u64,
        sub_team_members: &[crate::models::team::Member],
    ) -> Result<bool, &'static str> {
        if Self::check_sub_team_ownership(user_id, sub_team_leader_id) {
            return Ok(true);
        }
        let is_member = sub_team_members.iter().any(|m| {
            m.user_id == user_id && m.sub_team_id.map(|id| id == sub_team_id).unwrap_or(false)
        });
        if is_member {
            Ok(true)
        } else {
            Err("User is not a member of this sub-team")
        }
    }
}

fn forbidden_response(res: &mut salvo::Response, message: &str) {
    use salvo::http::StatusCode;
    use salvo::prelude::Json;
    res.status_code(StatusCode::FORBIDDEN);
    res.render(Json(serde_json::json!({
        "error": "Forbidden",
        "message": message
    })));
}

#[handler]
pub async fn require_team_member(
    req: &mut salvo::Request,
    depot: &mut salvo::Depot,
    res: &mut salvo::Response,
    ctrl: &mut salvo::FlowCtrl,
) {
    let user_id = match depot.get::<i64>("user_id") {
        Ok(id) => *id as u64,
        Err(_) => {
            unauthorized_response(res);
            ctrl.skip_rest();
            return;
        }
    };

    let team_id = match req.param::<u64>("team_id") {
        Some(id) => id,
        None => {
            res.status_code(salvo::http::StatusCode::BAD_REQUEST);
            res.render(Json(
                serde_json::json!({"error": "Bad Request", "message": "Missing team_id"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    let pool = match crate::db::pool::create_pool().await {
        Ok(p) => p,
        Err(_) => {
            res.status_code(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(
                serde_json::json!({"error": "Internal Server Error", "message": "Failed to create database pool"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    let team = match crate::db::db_team::DbTeam::get_team_by_id(&pool, team_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            res.status_code(salvo::http::StatusCode::NOT_FOUND);
            res.render(Json(
                serde_json::json!({"error": "Not Found", "message": "Team not found"}),
            ));
            ctrl.skip_rest();
            return;
        }
        Err(_) => {
            res.status_code(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(
                serde_json::json!({"error": "Internal Server Error", "message": "Database error"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    let is_owner = team.team_leader_id == user_id;
    let is_member = team.team_members.iter().any(|m| m.user_id == user_id);

    if is_owner || is_member {
        depot.insert("team_id", team_id);
        ctrl.call_next(req, depot, res).await;
    } else {
        forbidden_response(res, "User is not a member of this team");
        ctrl.skip_rest();
    }
}

#[handler]
pub async fn require_team_admin(
    req: &mut salvo::Request,
    depot: &mut salvo::Depot,
    res: &mut salvo::Response,
    ctrl: &mut salvo::FlowCtrl,
) {
    let user_id = match depot.get::<i64>("user_id") {
        Ok(id) => *id as u64,
        Err(_) => {
            unauthorized_response(res);
            ctrl.skip_rest();
            return;
        }
    };

    let team_id = match req.param::<u64>("team_id") {
        Some(id) => id,
        None => {
            res.status_code(salvo::http::StatusCode::BAD_REQUEST);
            res.render(Json(
                serde_json::json!({"error": "Bad Request", "message": "Missing team_id"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    let pool = match crate::db::pool::create_pool().await {
        Ok(p) => p,
        Err(_) => {
            res.status_code(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(
                serde_json::json!({"error": "Internal Server Error", "message": "Failed to create database pool"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    let team = match crate::db::db_team::DbTeam::get_team_by_id(&pool, team_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            res.status_code(salvo::http::StatusCode::NOT_FOUND);
            res.render(Json(
                serde_json::json!({"error": "Not Found", "message": "Team not found"}),
            ));
            ctrl.skip_rest();
            return;
        }
        Err(_) => {
            res.status_code(salvo::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(
                serde_json::json!({"error": "Internal Server Error", "message": "Database error"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    if team.team_leader_id == user_id {
        depot.insert("team_role", TeamRole::Owner.to_string());
        ctrl.call_next(req, depot, res).await;
        return;
    }

    let member = team.team_members.iter().find(|m| m.user_id == user_id);
    match member {
        Some(m) => {
            let is_admin = m.level >= 128;
            if is_admin {
                depot.insert("team_role", TeamRole::Admin.to_string());
                ctrl.call_next(req, depot, res).await;
            } else {
                forbidden_response(res, "Admin privileges required");
                ctrl.skip_rest();
            }
        }
        None => {
            forbidden_response(res, "User is not a member of this team");
            ctrl.skip_rest();
        }
    }
}

#[handler]
pub async fn require_resource_owner(
    req: &mut salvo::Request,
    depot: &mut salvo::Depot,
    res: &mut salvo::Response,
    ctrl: &mut salvo::FlowCtrl,
) {
    let user_id = match depot.get::<i64>("user_id") {
        Ok(id) => *id as u64,
        Err(_) => {
            unauthorized_response(res);
            ctrl.skip_rest();
            return;
        }
    };

    let resource_type = req
        .query::<String>("resource_type")
        .or_else(|| req.param::<String>("resource_type"));

    let resource_id = match req.param::<u64>("resource_id") {
        Some(id) => id,
        None => {
            res.status_code(salvo::http::StatusCode::BAD_REQUEST);
            res.render(Json(
                serde_json::json!({"error": "Bad Request", "message": "Missing resource_id"}),
            ));
            ctrl.skip_rest();
            return;
        }
    };

    let pool = match crate::db::pool::create_pool().await {
        Ok(p) => p,
        Err(_) => {
            forbidden_response(res, "Database error");
            ctrl.skip_rest();
            return;
        }
    };

    let is_owner = match resource_type.as_deref() {
        Some("user") => resource_id == user_id,
        Some("task") => {
            match crate::db::db_task::DbTask::get_task_by_id(&pool, resource_id).await {
                Ok(Some(task)) => task.task_leader_id == user_id,
                _ => false,
            }
        }
        Some("team") => {
            match crate::db::db_team::DbTeam::get_team_by_id(&pool, resource_id).await {
                Ok(Some(team)) => team.team_leader_id == user_id,
                _ => false,
            }
        }
        Some("sub_team") => {
            match crate::db::db_sub_team::DbSubTeam::get_sub_team_by_id(&pool, resource_id).await {
                Ok(Some(sub_team)) => sub_team.sub_team_leader_id == user_id,
                _ => false,
            }
        }
        _ => false,
    };

    if is_owner {
        ctrl.call_next(req, depot, res).await;
    } else {
        forbidden_response(res, "You do not have permission to access this resource");
        ctrl.skip_rest();
    }
}
