use crate::api::{ApiClient, ApiResult};
use crate::store::team_store::TeamMember;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubTeam {
    pub sub_team_id: u64,
    pub sub_team_name: String,
    pub sub_team_leader_id: u64,
    pub sub_team_members: Vec<TeamMember>,
    pub sub_team_create_time: i64,
    pub sub_team_description: Option<String>,
    pub team_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubTeamRequest {
    pub sub_team_name: String,
    pub sub_team_leader_id: Option<u64>,
    pub sub_team_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubTeamRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_team_leader_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_team_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: u64,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberLevelRequest {
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTeamListResponse {
    pub sub_teams: Vec<SubTeam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTeamResponse {
    pub sub_team: SubTeam,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembersResponse {
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}

pub async fn create_sub_team(
    client: &ApiClient,
    team_id: u64,
    req: &CreateSubTeamRequest,
) -> ApiResult<SubTeam> {
    let path = format!("/api/teams/{}/subteams", team_id);
    let resp: SubTeamResponse = client.post(&path, req).await?;
    Ok(resp.sub_team)
}

pub async fn list_sub_teams(client: &ApiClient, team_id: u64) -> ApiResult<Vec<SubTeam>> {
    let path = format!("/api/teams/{}/subteams", team_id);
    let resp: SubTeamListResponse = client.get(&path).await?;
    Ok(resp.sub_teams)
}

pub async fn get_sub_team(client: &ApiClient, sub_team_id: u64) -> ApiResult<SubTeam> {
    let path = format!("/api/subteams/{}", sub_team_id);
    let resp: SubTeamResponse = client.get(&path).await?;
    Ok(resp.sub_team)
}

pub async fn update_sub_team(
    client: &ApiClient,
    sub_team_id: u64,
    req: &UpdateSubTeamRequest,
) -> ApiResult<SubTeam> {
    let path = format!("/api/subteams/{}", sub_team_id);
    let resp: SubTeamResponse = client.put(&path, req).await?;
    Ok(resp.sub_team)
}

pub async fn delete_sub_team(client: &ApiClient, sub_team_id: u64) -> ApiResult<()> {
    let path = format!("/api/subteams/{}", sub_team_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}

pub async fn list_sub_team_members(client: &ApiClient, sub_team_id: u64) -> ApiResult<Vec<TeamMember>> {
    let path = format!("/api/subteams/{}/members", sub_team_id);
    let resp: MembersResponse = client.get(&path).await?;
    Ok(resp.members)
}

pub async fn add_sub_team_member(
    client: &ApiClient,
    sub_team_id: u64,
    req: &AddMemberRequest,
) -> ApiResult<()> {
    let path = format!("/api/subteams/{}/members", sub_team_id);
    let _: MessageResponse = client.post(&path, req).await?;
    Ok(())
}

pub async fn update_sub_team_member_level(
    client: &ApiClient,
    sub_team_id: u64,
    user_id: u64,
    req: &UpdateMemberLevelRequest,
) -> ApiResult<()> {
    let path = format!("/api/subteams/{}/members/{}/role", sub_team_id, user_id);
    let _: MessageResponse = client.put(&path, req).await?;
    Ok(())
}

pub async fn remove_sub_team_member(
    client: &ApiClient,
    sub_team_id: u64,
    user_id: u64,
) -> ApiResult<()> {
    let path = format!("/api/subteams/{}/members/{}", sub_team_id, user_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}
