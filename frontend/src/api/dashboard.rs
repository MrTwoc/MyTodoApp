use crate::api::{ApiClient, ApiResult};
use crate::store::task_store::Task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardTaskStats {
    pub total: u32,
    pub active: u32,
    pub completed: u32,
    pub paused: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTaskStat {
    pub team_id: u64,
    pub team_name: String,
    pub total: u32,
    pub active: u32,
    pub completed: u32,
    pub paused: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTaskStatsResponse {
    pub teams: Vec<TeamTaskStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub username: Option<String>,
    pub personal_tasks: DashboardTaskStats,
    pub team_tasks: DashboardTaskStats,
    pub recent_personal_tasks: Vec<Task>,
    pub recent_team_tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTasksResponse {
    pub personal_tasks: DashboardTaskStats,
    pub team_tasks: DashboardTaskStats,
}

pub async fn get_overview(client: &ApiClient) -> ApiResult<DashboardOverview> {
    client.get("/api/dashboard").await
}

pub async fn get_task_overview(client: &ApiClient) -> ApiResult<DashboardTasksResponse> {
    client.get("/api/dashboard/tasks").await
}

pub async fn get_team_overview(client: &ApiClient) -> ApiResult<TeamTaskStatsResponse> {
    client.get("/api/dashboard/teams").await
}
