use crate::api::{ApiClient, ApiResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub comment_id: u64,
    pub task_id: u64,
    pub user_id: u64,
    pub content: String,
    pub parent_id: Option<u64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommentRequest {
    pub content: String,
}

pub async fn create_comment(
    client: &ApiClient,
    task_id: u64,
    req: &CreateCommentRequest,
) -> ApiResult<TaskComment> {
    let path = format!("/api/tasks/{}/comments", task_id);
    #[derive(Deserialize)]
    struct CommentResponse {
        comment: TaskComment,
    }
    let resp: CommentResponse = client.post(&path, req).await?;
    Ok(resp.comment)
}

pub async fn get_task_comments(client: &ApiClient, task_id: u64) -> ApiResult<Vec<TaskComment>> {
    let path = format!("/api/tasks/{}/comments", task_id);
    #[derive(Deserialize)]
    struct CommentsResponse {
        comments: Vec<TaskComment>,
    }
    let resp: CommentsResponse = client.get(&path).await?;
    Ok(resp.comments)
}

pub async fn update_comment(
    client: &ApiClient,
    comment_id: u64,
    req: &UpdateCommentRequest,
) -> ApiResult<TaskComment> {
    let path = format!("/api/comments/{}", comment_id);
    #[derive(Deserialize)]
    struct CommentResponse {
        comment: TaskComment,
    }
    let resp: CommentResponse = client.put(&path, req).await?;
    Ok(resp.comment)
}

pub async fn delete_comment(client: &ApiClient, comment_id: u64) -> ApiResult<()> {
    let path = format!("/api/comments/{}", comment_id);
    #[derive(Deserialize)]
    struct DeleteResponse {
        message: String,
    }
    let _: DeleteResponse = client.delete(&path).await?;
    Ok(())
}
