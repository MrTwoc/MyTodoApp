use crate::db::pool::create_pool;
use crate::services::dashboard_service::{DashboardService, DashboardTeamStats, DashboardTaskStats};
use salvo::prelude::*;

#[endpoint]
pub async fn get_overview(depot: &mut Depot, res: &mut Response) {
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

    let pool = match create_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Database connection failed",
                "message": e.to_string()
            })));
            return;
        }
    };

    match DashboardService::overview(&pool, user_id).await {
        Ok(data) => {
            res.status_code(StatusCode::OK);
            let payload = serde_json::to_value(data).unwrap_or_else(|_| {
                serde_json::json!({
                    "personal_tasks": serde_json::json!({"total":0,"active":0,"completed":0,"paused":0}),
                    "team_tasks": serde_json::json!({"total":0,"active":0,"completed":0,"paused":0}),
                    "recent_personal_tasks": [],
                    "recent_team_tasks": []
                })
            });
            res.render(Json(payload));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch dashboard overview",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_task_overview(depot: &mut Depot, res: &mut Response) {
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

    let pool = match create_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Database connection failed",
                "message": e.to_string()
            })));
            return;
        }
    };

    match DashboardService::tasks(&pool, user_id).await {
        Ok(data) => {
            let payload = DashboardTaskStats {
                personal_tasks: data.personal_tasks,
                team_tasks: data.team_tasks,
            };
            res.status_code(StatusCode::OK);
            let payload = serde_json::to_value(payload).unwrap_or_else(|_| {
                serde_json::json!({
                    "personal_tasks": serde_json::json!({"total":0,"active":0,"completed":0,"paused":0}),
                    "team_tasks": serde_json::json!({"total":0,"active":0,"completed":0,"paused":0})
                })
            });
            res.render(Json(payload));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task overview",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_team_overview(depot: &mut Depot, res: &mut Response) {
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

    let pool = match create_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Database connection failed",
                "message": e.to_string()
            })));
            return;
        }
    };

    match DashboardService::teams(&pool, user_id).await {
        Ok(data) => {
            let payload = DashboardTeamStats { teams: data.teams };
            res.status_code(StatusCode::OK);
            let payload = serde_json::to_value(payload)
                .unwrap_or_else(|_| serde_json::json!({"teams": []}));
            res.render(Json(payload));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch team overview",
                "message": e.to_string()
            })));
        }
    }
}
