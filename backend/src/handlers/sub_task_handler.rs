use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::pool::create_pool;
use crate::services::sub_task_service::{CreateSubTaskRequest, SubTaskService, UpdateSubTaskRequest};
use crate::ws;

#[endpoint]
pub async fn create_sub_task(task_id: PathParam<u64>, req: &mut Request, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

    let request: CreateSubTaskRequest = match req.parse_json().await {
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

    match SubTaskService::create_sub_task(&pool, task_id, request).await {
        Ok(sub_task) => {
            let payload = serde_json::to_value(&sub_task).unwrap_or_default();
            ws::push(
                "subtask.created",
                serde_json::json!({
                    "task_id": task_id,
                    "sub_task": payload,
                }),
            );

            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Sub task created successfully",
                "sub_task": payload
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create sub task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn list_sub_tasks(task_id: PathParam<u64>, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

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

    match SubTaskService::list_sub_tasks(&pool, task_id).await {
        Ok(sub_tasks) => {
            let sub_tasks: Vec<serde_json::Value> =
                sub_tasks
                    .into_iter()
                    .map(|sub_task| serde_json::to_value(sub_task).unwrap_or_default())
                    .collect();
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "sub_tasks": sub_tasks
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to list sub tasks",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_sub_task(
    task_id: PathParam<u64>,
    sub_task_id: PathParam<u64>,
    req: &mut Request,
    res: &mut Response,
) {
    let task_id: u64 = task_id.into_inner();
    let sub_task_id: u64 = sub_task_id.into_inner();

    let request: UpdateSubTaskRequest = match req.parse_json().await {
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

    match SubTaskService::update_sub_task(&pool, task_id, sub_task_id, request).await {
        Ok(Some(sub_task)) => {
            let payload = serde_json::to_value(&sub_task).unwrap_or_default();
            ws::push(
                "subtask.updated",
                serde_json::json!({
                    "task_id": task_id,
                    "sub_task": payload,
                }),
            );
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub task updated successfully",
                "sub_task": payload
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update sub task",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn delete_sub_task(task_id: PathParam<u64>, sub_task_id: PathParam<u64>, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();
    let sub_task_id: u64 = sub_task_id.into_inner();

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

    match SubTaskService::delete_sub_task(&pool, task_id, sub_task_id).await {
        Ok(true) => {
            ws::push(
                "subtask.deleted",
                serde_json::json!({
                    "task_id": task_id,
                    "sub_task_id": sub_task_id,
                }),
            );
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Sub task deleted successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Sub task not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to delete sub task",
                "message": e.to_string()
            })));
        }
    }
}
