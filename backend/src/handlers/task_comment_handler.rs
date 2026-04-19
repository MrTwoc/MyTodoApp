use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::db_task_comment::DbTaskComment;
use crate::db::pool::DbPool;

#[endpoint]
pub async fn create_comment(req: &mut Request, depot: &mut Depot, res: &mut Response) {
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

    let task_id: u64 = match req.param::<u64>("task_id") {
        Some(id) => id,
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid task_id"
            })));
            return;
        }
    };

    let body: serde_json::Value = match req.parse_json().await {
        Ok(body) => body,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": e.to_string()
            })));
            return;
        }
    };

    let content = match body.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Content is required"
            })));
            return;
        }
    };

    if content.trim().is_empty() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "error": "Content cannot be empty"
        })));
        return;
    }

    let parent_id = body
        .get("parent_id")
        .and_then(|v| v.as_u64())
        .or_else(|| body.get("parent_id").and_then(|v| v.as_i64()).map(|v| v as u64));

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match DbTaskComment::create_comment(pool, task_id, user_id, &content, parent_id).await {
        Ok(comment) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Comment created successfully",
                "comment": serde_json::to_value(&comment).unwrap_or_default()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create comment",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn get_comments(task_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    let task_id: u64 = task_id.into_inner();

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    match DbTaskComment::get_comments_by_task_id(pool, task_id, Some(100), None).await {
        Ok(comments) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "comments": comments.iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect::<Vec<_>>()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch comments",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn update_comment(
    comment_id: PathParam<u64>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let comment_id: u64 = comment_id.into_inner();

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

    let body: serde_json::Value = match req.parse_json().await {
        Ok(body) => body,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body",
                "message": e.to_string()
            })));
            return;
        }
    };

    let content = match body.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Content is required"
            })));
            return;
        }
    };

    if content.trim().is_empty() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "error": "Content cannot be empty"
        })));
        return;
    }

    let pool = depot
        .get::<DbPool>("db_pool")
        .expect("DbPool not found in depot");

    // 验证评论所有权
    match DbTaskComment::get_comment_by_id(pool, comment_id).await {
        Ok(Some(comment)) => {
            if comment.user_id != user_id {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You can only edit your own comments"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Comment not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch comment",
                "message": e.to_string()
            })));
            return;
        }
    }

    match DbTaskComment::update_comment(pool, comment_id, &content).await {
        Ok(Some(comment)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Comment updated successfully",
                "comment": serde_json::to_value(&comment).unwrap_or_default()
            })));
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Comment not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to update comment",
                "message": e.to_string()
            })));
        }
    }
}

#[endpoint]
pub async fn delete_comment(
    comment_id: PathParam<u64>,
    depot: &mut Depot,
    res: &mut Response,
) {
    let comment_id: u64 = comment_id.into_inner();

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

    // 验证评论所有权
    match DbTaskComment::get_comment_by_id(pool, comment_id).await {
        Ok(Some(comment)) => {
            if comment.user_id != user_id {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You can only delete your own comments"
                })));
                return;
            }
        }
        Ok(None) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Comment not found"
            })));
            return;
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to fetch comment",
                "message": e.to_string()
            })));
            return;
        }
    }

    match DbTaskComment::delete_comment(pool, comment_id).await {
        Ok(true) => {
            res.status_code(StatusCode::OK);
            res.render(Json(serde_json::json!({
                "message": "Comment deleted successfully"
            })));
        }
        Ok(false) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(serde_json::json!({
                "error": "Comment not found"
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Failed to delete comment",
                "message": e.to_string()
            })));
        }
    }
}
