use crate::api::{ApiClient, ApiResult};
use crate::store::team_store::TeamMember;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    pub group_id: u64,
    pub group_name: String,
    pub group_leader_id: u64,
    pub group_members: Vec<TeamMember>,
    pub group_create_time: i64,
    pub group_description: Option<String>,
    pub team_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub group_leader_id: Option<u64>,
    pub group_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_leader_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_description: Option<String>,
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
pub struct GroupListResponse {
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    pub group: Group,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembersResponse {
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}

pub async fn create_group(
    client: &ApiClient,
    team_id: u64,
    req: &CreateGroupRequest,
) -> ApiResult<Group> {
    let path = format!("/api/teams/{}/groups", team_id);
    let resp: GroupResponse = client.post(&path, req).await?;
    Ok(resp.group)
}

pub async fn list_groups(client: &ApiClient, team_id: u64) -> ApiResult<Vec<Group>> {
    let path = format!("/api/teams/{}/groups", team_id);
    let resp: GroupListResponse = client.get(&path).await?;
    Ok(resp.groups)
}

pub async fn get_group(client: &ApiClient, group_id: u64) -> ApiResult<Group> {
    let path = format!("/api/groups/{}", group_id);
    let resp: GroupResponse = client.get(&path).await?;
    Ok(resp.group)
}

pub async fn update_group(
    client: &ApiClient,
    group_id: u64,
    req: &UpdateGroupRequest,
) -> ApiResult<Group> {
    let path = format!("/api/groups/{}", group_id);
    let resp: GroupResponse = client.put(&path, req).await?;
    Ok(resp.group)
}

pub async fn delete_group(client: &ApiClient, group_id: u64) -> ApiResult<()> {
    let path = format!("/api/groups/{}", group_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}

pub async fn list_group_members(client: &ApiClient, group_id: u64) -> ApiResult<Vec<TeamMember>> {
    let path = format!("/api/groups/{}/members", group_id);
    let resp: MembersResponse = client.get(&path).await?;
    Ok(resp.members)
}

pub async fn add_group_member(
    client: &ApiClient,
    group_id: u64,
    req: &AddMemberRequest,
) -> ApiResult<()> {
    let path = format!("/api/groups/{}/members", group_id);
    let _: MessageResponse = client.post(&path, req).await?;
    Ok(())
}

pub async fn update_group_member_level(
    client: &ApiClient,
    group_id: u64,
    user_id: u64,
    req: &UpdateMemberLevelRequest,
) -> ApiResult<()> {
    let path = format!("/api/groups/{}/members/{}/role", group_id, user_id);
    let _: MessageResponse = client.put(&path, req).await?;
    Ok(())
}

pub async fn remove_group_member(client: &ApiClient, group_id: u64, user_id: u64) -> ApiResult<()> {
    let path = format!("/api/groups/{}/members/{}", group_id, user_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}
