use crate::api::{ApiClient, ApiResult};
use crate::store::task_store::TaskStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubTask {
    pub sub_task_id: u64,
    pub task_id: u64,
    pub sub_task_name: String,
    pub sub_task_description: Option<String>,
    pub sub_task_status: TaskStatus,
    pub sub_task_create_time: i64,
    pub sub_task_update_time: Option<i64>,
    pub sub_task_complete_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubTaskRequest {
    pub sub_task_name: String,
    pub sub_task_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_task_status: Option<TaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_task_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_task_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_task_status: Option<TaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskListResponse {
    pub sub_tasks: Vec<SubTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskResponse {
    pub sub_task: SubTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}

pub async fn create_sub_task(
    client: &ApiClient,
    task_id: u64,
    req: &CreateSubTaskRequest,
) -> ApiResult<SubTask> {
    let path = format!("/api/tasks/{}/subtasks", task_id);
    let resp: SubTaskResponse = client.post(&path, req).await?;
    Ok(resp.sub_task)
}

pub async fn list_sub_tasks(client: &ApiClient, task_id: u64) -> ApiResult<Vec<SubTask>> {
    let path = format!("/api/tasks/{}/subtasks", task_id);
    let resp: SubTaskListResponse = client.get(&path).await?;
    Ok(resp.sub_tasks)
}

pub async fn update_sub_task(
    client: &ApiClient,
    task_id: u64,
    sub_task_id: u64,
    req: &UpdateSubTaskRequest,
) -> ApiResult<SubTask> {
    let path = format!("/api/tasks/{}/subtasks/{}", task_id, sub_task_id);
    let resp: SubTaskResponse = client.put(&path, req).await?;
    Ok(resp.sub_task)
}

pub async fn delete_sub_task(
    client: &ApiClient,
    task_id: u64,
    sub_task_id: u64,
) -> ApiResult<()> {
    let path = format!("/api/tasks/{}/subtasks/{}", task_id, sub_task_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}
