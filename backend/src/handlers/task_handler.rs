use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::db_task_log::{DbTaskLog, TaskLogAction};
use crate::db::pool::DbPool;
use crate::services::task_service::{
    CreateTaskRequest, ListTasksQuery, TaskService, UpdateTaskPriorityRequest, UpdateTaskRequest,
    UpdateTaskStatusRequest,
};
use crate::services::team_service::TeamService;

#[endpoint]
pub async fn create_task(req: &mut Request, depot: &mut Depot, res: &mut Response) {
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

    let request: CreateTaskRequest = match req.parse_json().await {
        Ok(req) => req,
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

    match TaskService::create_task(pool, user_id, request).await {
        Ok(task) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Task created successfully",
                "task": serde_json::to_value(&task).unwrap_or_default()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_task(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match TaskService::get_task_by_id(pool, task_id).await {
        Ok(Some(task)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "task": serde_json::to_value(&task).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn list_tasks(depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let user_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => Some(*id as u64),
        None => None,
    };

    let query: ListTasksQuery = req.parse_queries().unwrap_or_default();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match TaskService::list_tasks(pool, user_id, query.team_id, query).await {
        Ok(tasks) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "tasks": tasks.iter().map(|t| serde_json::to_value(t).unwrap_or_default()).collect::<Vec<_>>()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch tasks",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_task(
    task_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let task_id: u64 = task_id.into_inner();

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

    let request: UpdateTaskRequest = match req.parse_json().await {
        Ok(req) => req,
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

    let old_task = TaskService::get_task_by_id(pool, task_id).await.ok().flatten();

    let mut changes = Vec::new();
    if let Some(ref old) = old_task {
        if let Some(ref name) = request.task_name {
            if &old.task_name != name {
                changes.push(format!("name: {} → {}", old.task_name, name));
            }
        }
        if let Some(ref desc) = request.task_description {
            if old.task_description.as_deref() != Some(desc) {
                changes.push("description changed".to_string());
            }
        }
        if let Some(priority) = request.task_priority {
            if old.task_priority != priority {
                changes.push(format!("priority: {} → {}", old.task_priority, priority));
            }
        }
        if let Some(difficulty) = request.task_difficulty {
            if old.task_difficulty != difficulty {
                changes.push(format!("difficulty: {} → {}", old.task_difficulty, difficulty));
            }
        }
        if let Some(Some(deadline)) = request.task_deadline {
            let old_deadline = old.task_deadline.unwrap_or(0);
            if old_deadline != deadline {
                changes.push("deadline changed".to_string());
            }
        }
        if let Some(ref status) = request.task_status {
            if old.task_status != *status {
                changes.push(format!("status: {} → {}", old.task_status, status));
            }
        }
    }

    match TaskService::update_task(pool, task_id, request).await {
        Ok(Some(task)) => {
            let details = if changes.is_empty() {
                "Task updated".to_string()
            } else {
                changes.join(", ")
            };

            let _ = DbTaskLog::create_task_log(
                pool,
                task_id,
                user_id,
                TaskLogAction::Updated,
                None,
                None,
                Some(&details),
            )
            .await;

            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Task updated successfully",
                "task": serde_json::to_value(&task).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn delete_task(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

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

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match TaskService::get_task_by_id(pool, task_id).await {
        Ok(Some(task)) => {
            let is_task_leader = task.task_leader_id == user_id;
            let is_team_leader = if let Some(team_id) = task.task_team_id {
                match TeamService::get_team(pool, team_id).await {
                    Ok(Some(team)) => team.team_leader_id == user_id,
                    _ => false,
                }
            } else {
                false
            };

            if !is_task_leader && !is_team_leader {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You don't have permission to delete this task"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task",
                "message": e.to_string()
            })));
            return;
        }
    }

    match TaskService::delete_task(pool, task_id).await {
        Ok(true) => {
            let task_name = TaskService::get_task_by_id(pool, task_id).await
                .ok().flatten()
                .map(|t| t.task_name)
                .unwrap_or_else(|| "Unknown".to_string());
            let _ = DbTaskLog::create_task_log(
                pool,
                task_id,
                user_id,
                TaskLogAction::Deleted,
                None,
                None,
                Some(&format!("Deleted task: {}", task_name)),
            )
            .await;

            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Task deleted successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to delete task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_task_status(
    task_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let task_id: u64 = task_id.into_inner();

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

    let request: UpdateTaskStatusRequest = match req.parse_json().await {
        Ok(req) => req,
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

    match TaskService::get_task_by_id(pool, task_id).await {
        Ok(Some(task)) => {
            if task.task_leader_id != user_id {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You don't have permission to update this task status"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task",
                "message": e.to_string()
            })));
            return;
        }
    }

    match TaskService::update_task_status(pool, task_id, request.task_status.clone()).await {
        Ok(Some(task)) => {
            let old_status = task.task_status.to_string();
            let new_status = request.task_status.to_string();
            let _ = DbTaskLog::create_task_log(
                pool,
                task_id,
                user_id,
                TaskLogAction::StatusChanged,
                Some(&old_status),
                Some(&new_status),
                Some(&format!(
                    "Status changed from {} to {}",
                    old_status, new_status
                )),
            )
            .await;

            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Task status updated successfully",
                "task": serde_json::to_value(&task).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update task status",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_task_priority(
    task_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let task_id: u64 = task_id.into_inner();

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

    let request: UpdateTaskPriorityRequest = match req.parse_json().await {
        Ok(req) => req,
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

    match TaskService::get_task_by_id(pool, task_id).await {
        Ok(Some(task)) => {
            if task.task_leader_id != user_id {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You don't have permission to update this task priority"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task",
                "message": e.to_string()
            })));
            return;
        }
    }

    match TaskService::update_task_priority(pool, task_id, request.task_priority).await {
        Ok(Some(task)) => {
            let new_priority = task.task_priority.to_string();
            let _ = DbTaskLog::create_task_log(
                pool,
                task_id,
                user_id,
                TaskLogAction::PriorityChanged,
                None,
                Some(&new_priority),
                Some(&format!("Priority changed to {}", new_priority)),
            )
            .await;

            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Task priority updated successfully",
                "task": serde_json::to_value(&task).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update task priority",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn toggle_task_favorite(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let _user_id = match depot.get::<i64>("user_id").ok() {
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

    let task_id: u64 = task_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match crate::db::db_task::DbTask::toggle_favorite(pool, task_id).await {
        Ok(is_favorite) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "task_id": task_id,
                "is_favorite": is_favorite
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to toggle favorite",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_task_logs(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match crate::db::db_task_log::DbTaskLog::get_task_logs(pool, task_id, Some(50), None).await {
        Ok(logs) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "logs": logs.iter().map(|log| serde_json::to_value(log).unwrap_or_default()).collect::<Vec<_>>()
            })));
        }
        Err(e) => {
            tracing::error!("Failed to get task logs: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task logs",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_recycle_bin(depot: &mut Depot, res: &mut Response) {
    let user_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => Some(*id as u64),
        None => None,
    };

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match TaskService::list_deleted_tasks(pool, user_id, None, None).await {
        Ok(tasks) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "tasks": tasks.iter().map(|t| serde_json::to_value(t).unwrap_or_default()).collect::<Vec<_>>()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch recycle bin",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn restore_task(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

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

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match TaskService::get_task_by_id(pool, task_id).await {
        Ok(Some(task)) => {
            let is_task_leader = task.task_leader_id == user_id;
            let is_team_leader = if let Some(team_id) = task.task_team_id {
                match TeamService::get_team(pool, team_id).await {
                    Ok(Some(team)) => team.team_leader_id == user_id,
                    _ => false,
                }
            } else {
                false
            };

            if !is_task_leader && !is_team_leader {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You don't have permission to restore this task"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task",
                "message": e.to_string()
            })));
            return;
        }
    }

    match TaskService::restore_task(pool, task_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Task restored successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to restore task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn permanent_delete_task(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

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

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match TaskService::get_task_by_id(pool, task_id).await {
        Ok(Some(task)) => {
            let is_task_leader = task.task_leader_id == user_id;
            let is_team_leader = if let Some(team_id) = task.task_team_id {
                match TeamService::get_team(pool, team_id).await {
                    Ok(Some(team)) => team.team_leader_id == user_id,
                    _ => false,
                }
            } else {
                false
            };

            if !is_task_leader && !is_team_leader {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You don't have permission to permanently delete this task"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch task",
                "message": e.to_string()
            })));
            return;
        }
    }

    match TaskService::permanent_delete_task(pool, task_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Task permanently deleted"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to permanently delete task",
                "message": e.to_string()
            })));
        }
    }
}
