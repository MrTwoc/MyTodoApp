# 小组功能 (SubTeam) 完整实现

## 概述

本教程介绍如何在项目中实现小组功能。小组（SubTeam）是团队（Team）下的更小分支，允许将团队成员分成不同的组。

### 功能需求

1. 小组是团队中的更小分支
2. 可以把团队分为不同的组
3. 必须为团队成员才能加入某个小组
4. 团队成员中级别大于特定level才可以创建小组

---

## 第一部分：数据库设计

### 1.1 数据库迁移文件

创建 `migrations/007_sub_team.sql`：

```sql
-- ============================================
-- 子团队表 (sub_teams)
-- ============================================
CREATE TABLE IF NOT EXISTS sub_teams (
    sub_team_id BIGSERIAL PRIMARY KEY,
    sub_team_name TEXT NOT NULL,
    sub_team_leader_id BIGINT NOT NULL,
    team_id BIGINT NOT NULL,
    sub_team_create_time BIGINT NOT NULL,
    sub_team_description TEXT,
    CONSTRAINT chk_sub_team_create_time CHECK (sub_team_create_time > 0)
);

-- ============================================
-- 子团队成员表 (sub_team_members)
-- ============================================
CREATE TABLE IF NOT EXISTS sub_team_members (
    sub_team_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    level SMALLINT NOT NULL DEFAULT 1,
    join_time BIGINT NOT NULL,
    PRIMARY KEY (sub_team_id, user_id),
    CONSTRAINT chk_sub_team_member_level CHECK (level >= 1 AND level <= 255),
    CONSTRAINT chk_sub_team_member_join_time CHECK (join_time > 0)
);

-- 外键约束
ALTER TABLE sub_teams
ADD CONSTRAINT fk_sub_teams_team
FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE;

ALTER TABLE sub_teams
ADD CONSTRAINT fk_sub_teams_leader
FOREIGN KEY (sub_team_leader_id) REFERENCES users(user_id) ON DELETE CASCADE;

ALTER TABLE sub_team_members
ADD CONSTRAINT fk_sub_team_members_sub_team
FOREIGN KEY (sub_team_id) REFERENCES sub_teams(sub_team_id) ON DELETE CASCADE;

ALTER TABLE sub_team_members
ADD CONSTRAINT fk_sub_team_members_user
FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 索引
CREATE INDEX IF NOT EXISTS idx_sub_teams_team ON sub_teams(team_id);
CREATE INDEX IF NOT EXISTS idx_sub_teams_leader ON sub_teams(sub_team_leader_id);
CREATE INDEX IF NOT EXISTS idx_sub_team_members_sub_team ON sub_team_members(sub_team_id);
CREATE INDEX IF NOT EXISTS idx_sub_team_members_user ON sub_team_members(user_id);
```

---

## 第二部分：后端模型层

### 2.1 定义数据结构

在 `models/team.rs` 中添加：

```rust
// 子团队结构体
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SubTeam {
    pub sub_team_id: u64,           // 子团队ID，雪花ID
    pub sub_team_name: String,     // 子团队名称
    pub sub_team_leader_id: u64,    // 子团队负责人ID
    pub sub_team_members: Vec<Member>,  // 子团队成员列表
    pub sub_team_create_time: i64,  // 子团队创建时间
    pub sub_team_description: Option<String>,  // 子团队描述
    pub team_id: u64,               // 所属团队ID
}

// 统一成员结构体
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub team_id: Option<u64>,
    pub sub_team_id: Option<u64>,
    pub user_id: u64,
    pub username: Option<String>,
    pub level: u8,
    pub join_time: i64,
}
```

---

## 第三部分：数据库访问层

### 3.1 子团队数据库操作

创建 `db/db_sub_team.rs`：

```rust
use crate::models::team::SubTeam;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbSubTeam;

impl DbSubTeam {
    // 创建子团队
    pub async fn create_sub_team(
        pool: &PgPool,
        sub_team_name: &str,
        sub_team_leader_id: u64,
        team_id: u64,
        sub_team_description: Option<&str>,
    ) -> Result<SubTeam> {
        let sub_team_id = crate::utils::id_generator::generate_team_id();
        let sub_team_create_time = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO sub_teams (sub_team_id, sub_team_name, sub_team_leader_id, team_id, sub_team_create_time, sub_team_description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING sub_team_id, sub_team_name, sub_team_leader_id, team_id, sub_team_create_time, sub_team_description
            "#,
        )
        .bind(sub_team_id as i64)
        .bind(sub_team_name)
        .bind(sub_team_leader_id as i64)
        .bind(team_id as i64)
        .bind(sub_team_create_time)
        .bind(sub_team_description)
        .fetch_one(pool)
        .await?;

        Ok(Self::row_to_sub_team(result)?)
    }

    // 根据ID获取子团队
    pub async fn get_sub_team_by_id(pool: &PgPool, sub_team_id: u64) -> Result<Option<SubTeam>> {
        // ... 实现代码
    }

    // 列出子团队
    pub async fn list_sub_teams(
        pool: &PgPool,
        team_id: Option<u64>,
        sub_team_leader_id: Option<u64>,
    ) -> Result<Vec<SubTeam>> {
        // ... 实现代码
    }

    // 更新子团队
    pub async fn update_sub_team(...) -> Result<Option<SubTeam>> {
        // ... 实现代码
    }

    // 删除子团队
    pub async fn delete_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<bool> {
        // ... 实现代码
    }
}
```

### 3.2 子团队成员数据库操作

创建 `db/db_sub_team_member.rs`：

```rust
use crate::models::team::Member;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbSubTeamMember;

impl DbSubTeamMember {
    // 添加子团队成员
    pub async fn add_sub_team_member(
        pool: &PgPool,
        sub_team_id: u64,
        user_id: u64,
        level: u8,
    ) -> Result<bool> {
        // ... 实现代码
    }

    // 移除子团队成员
    pub async fn remove_sub_team_member(...) -> Result<bool> {
        // ... 实现代码
    }

    // 获取子团队成员列表
    pub async fn get_sub_team_members(pool: &PgPool, sub_team_id: u64) -> Result<Vec<Member>> {
        // ... 实现代码
    }

    // 更新成员级别
    pub async fn update_member_level(...) -> Result<bool> {
        // ... 实现代码
    }

    // 检查是否是成员
    pub async fn is_member(pool: &PgPool, sub_team_id: u64, user_id: u64) -> Result<bool> {
        // ... 实现代码
    }

    // 获取成员级别
    pub async fn get_member_level(...) -> Result<Option<u8>> {
        // ... 实现代码
    }
}
```

---

## 第四部分：服务层

### 4.1 子团队服务

创建 `services/sub_team_service.rs`：

```rust
use crate::db::db_sub_team::DbSubTeam;
use crate::db::db_sub_team_member::DbSubTeamMember;
use crate::db::db_team::DbTeam;
use crate::models::team::{Member, SubTeam, Team};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubTeamRequest {
    pub sub_team_name: String,
    pub sub_team_leader_id: Option<u64>,
    pub sub_team_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubTeamRequest {
    pub sub_team_name: Option<String>,
    pub sub_team_leader_id: Option<u64>,
    pub sub_team_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddSubTeamMemberRequest {
    pub user_id: u64,
    pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubTeamMemberRequest {
    pub level: u8,
}

pub struct SubTeamService;

impl SubTeamService {
    // 创建子团队
    pub async fn create_sub_team(
        pool: &PgPool,
        team_id: u64,
        user_id: u64,
        request: CreateSubTeamRequest,
    ) -> Result<SubTeam> {
        let leader_id = request.sub_team_leader_id.unwrap_or(user_id);
        let sub_team = DbSubTeam::create_sub_team(
            pool,
            &request.sub_team_name,
            leader_id,
            team_id,
            request.sub_team_description.as_deref(),
        )
        .await?;

        // 将子团队ID添加到父团队的sub_team_ids中
        add_sub_team_to_parent(pool, team_id, sub_team.sub_team_id).await?;
        Ok(sub_team)
    }

    // 获取子团队
    pub async fn get_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<Option<SubTeam>> {
        DbSubTeam::get_sub_team_by_id(pool, sub_team_id).await
    }

    // 列出子团队
    pub async fn list_sub_teams(pool: &PgPool, team_id: u64) -> Result<Vec<SubTeam>> {
        DbSubTeam::list_sub_teams(pool, Some(team_id), None).await
    }

    // 更新子团队
    pub async fn update_sub_team(...) -> Result<Option<SubTeam>> {
        // ... 实现代码
    }

    // 删除子团队
    pub async fn delete_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<bool> {
        // ... 实现代码
    }

    // 获取子团队所属的团队
    pub async fn get_team_of_sub_team(pool: &PgPool, sub_team_id: u64) -> Result<Option<Team>> {
        // ... 实现代码
    }

    // 列出成员
    pub async fn list_members(pool: &PgPool, sub_team_id: u64) -> Result<Vec<Member>> {
        DbSubTeamMember::get_sub_team_members(pool, sub_team_id).await
    }

    // 添加成员
    pub async fn add_member(...) -> Result<bool> {
        DbSubTeamMember::add_sub_team_member(pool, sub_team_id, request.user_id, request.level).await
    }

    // 移除成员
    pub async fn remove_member(...) -> Result<bool> {
        DbSubTeamMember::remove_sub_team_member(pool, sub_team_id, user_id).await
    }

    // 更新成员级别
    pub async fn update_member_level(...) -> Result<bool> {
        DbSubTeamMember::update_member_level(pool, sub_team_id, user_id, request.level).await
    }
}
```

---

## 第五部分：处理器层 (Handler)

创建 `handlers/sub_team_handler.rs`：

```rust
use salvo::oapi::extract::PathParam;
use salvo::prelude::*;

use crate::db::pool::DbPool;
use crate::services::sub_team_service::{
    AddSubTeamMemberRequest, CreateSubTeamRequest, SubTeamService, UpdateSubTeamMemberRequest,
    UpdateSubTeamRequest,
};

#[endpoint]
pub async fn create_sub_team(
    team_id: PathParam<u64>,
    depot: &mut Depot,
    req: &mut Request,
    res: &mut Response,
) {
    let team_id: u64 = team_id.into_inner();
    let actor_id = match depot.get::<i64>("user_id").ok() {
        Some(id) => *id as u64,
        None => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Unauthorized"
            })));
            return;
        }
    };

    let request: CreateSubTeamRequest = match req.parse_json().await {
        Ok(r) => r,
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Invalid request body"
            })));
            return;
        }
    };

    let pool = depot.get::<DbPool>("db_pool").expect("DbPool not found in depot");

    match SubTeamService::create_sub_team(pool, team_id, actor_id, request).await {
        Ok(sub_team) => {
            res.status_code(StatusCode::CREATED);
            res.render(Json(serde_json::json!({
                "message": "Sub team created successfully",
                "sub_team": serde_json::to_value(&sub_team).unwrap_or_default()
            })));
        }
        Err(e) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(serde_json::json!({
                "error": "Failed to create sub team"
            })));
        }
    }
}

#[endpoint]
pub async fn list_sub_teams(team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    // ... 实现代码
}

#[endpoint]
pub async fn get_sub_team(sub_team_id: PathParam<u64>, depot: &mut Depot, res: &mut Response) {
    // ... 实现代码
}

#[endpoint]
pub async fn update_sub_team(...) {
    // ... 实现代码
}

#[endpoint]
pub async fn delete_sub_team(...) {
    // ... 实现代码
}

#[endpoint]
pub async fn list_sub_team_members(...) {
    // ... 实现代码
}

#[endpoint]
pub async fn add_sub_team_member(...) {
    // ... 实现代码
}

#[endpoint]
pub async fn remove_sub_team_member(...) {
    // ... 实现代码
}

#[endpoint]
pub async fn update_sub_team_member_level(...) {
    // ... 实现代码
}
```

---

## 第六部分：路由层

创建 `routes/sub_team_routes.rs`：

```rust
use salvo::prelude::*;

use crate::handlers::sub_team_handler;
use crate::middleware;

pub fn sub_team_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/teams")
        .hoop(auth_middleware)
        .push(
            Router::with_path("{team_id}")
                .push(Router::with_path("subteams")
                    .post(sub_team_handler::create_sub_team)
                    .get(sub_team_handler::list_sub_teams)),
        )
}

pub fn sub_team_single_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("api/subteams")
        .hoop(auth_middleware)
        .push(
            Router::with_path("{sub_team_id}")
                .get(sub_team_handler::get_sub_team)
                .put(sub_team_handler::update_sub_team)
                .delete(sub_team_handler::delete_sub_team)
                .push(Router::with_path("members").get(sub_team_handler::list_sub_team_members))
                .push(
                    Router::with_path("members/{user_id}")
                        .delete(sub_team_handler::remove_sub_team_member),
                )
                .push(
                    Router::with_path("members").post(sub_team_handler::add_sub_team_member),
                )
                .push(
                    Router::with_path("members/{user_id}/role").put(
                        sub_team_handler::update_sub_team_member_level,
                    ),
                ),
        )
}
```

### 6.1 在 main.rs 中注册路由

```rust
mod routes;

fn main() {
    // ... 其他代码
    
    router!(
        team_router(),
        sub_team_router(),
        sub_team_single_router(),
        // ... 其他路由
    );
}
```

---

## 第七部分：前端 API

创建 `frontend/src/api/sub_team.rs`：

```rust
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

// 创建子团队
pub async fn create_sub_team(
    client: &ApiClient,
    team_id: u64,
    req: &CreateSubTeamRequest,
) -> ApiResult<SubTeam> {
    let path = format!("/api/teams/{}/subteams", team_id);
    let resp: SubTeamResponse = client.post(&path, req).await?;
    Ok(resp.sub_team)
}

// 列出子团队
pub async fn list_sub_teams(client: &ApiClient, team_id: u64) -> ApiResult<Vec<SubTeam>> {
    let path = format!("/api/teams/{}/subteams", team_id);
    let resp: SubTeamListResponse = client.get(&path).await?;
    Ok(resp.sub_teams)
}

// 获取单个子团队
pub async fn get_sub_team(client: &ApiClient, sub_team_id: u64) -> ApiResult<SubTeam> {
    let path = format!("/api/subteams/{}", sub_team_id);
    let resp: SubTeamResponse = client.get(&path).await?;
    Ok(resp.sub_team)
}

// 更新子团队
pub async fn update_sub_team(
    client: &ApiClient,
    sub_team_id: u64,
    req: &UpdateSubTeamRequest,
) -> ApiResult<SubTeam> {
    let path = format!("/api/subteams/{}", sub_team_id);
    let resp: SubTeamResponse = client.put(&path, req).await?;
    Ok(resp.sub_team)
}

// 删除子团队
pub async fn delete_sub_team(client: &ApiClient, sub_team_id: u64) -> ApiResult<()> {
    let path = format!("/api/subteams/{}", sub_team_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}

// 列出子团队成员
pub async fn list_sub_team_members(client: &ApiClient, sub_team_id: u64) -> ApiResult<Vec<TeamMember>> {
    let path = format!("/api/subteams/{}/members", sub_team_id);
    let resp: MembersResponse = client.get(&path).await?;
    Ok(resp.members)
}

// 添加子团队成员
pub async fn add_sub_team_member(
    client: &ApiClient,
    sub_team_id: u64,
    req: &AddMemberRequest,
) -> ApiResult<()> {
    let path = format!("/api/subteams/{}/members", sub_team_id);
    let _: MessageResponse = client.post(&path, req).await?;
    Ok(())
}

// 更新成员级别
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

// 移除子团队成员
pub async fn remove_sub_team_member(
    client: &ApiClient,
    sub_team_id: u64,
    user_id: u64,
) -> ApiResult<()> {
    let path = format!("/api/subteams/{}/members/{}", sub_team_id, user_id);
    let _: MessageResponse = client.delete(&path).await?;
    Ok(())
}
```

### 7.1 在 mod.rs 中导出

```rust
pub mod sub_team;
```

---

## 第八部分：前端组件和页面

### 8.1 创建子团队页面组件

创建 `frontend/src/pages/sub_team_page.rs`：

```rust
use leptos::*;
use crate::api::sub_team::*;
use crate::store::team_store::TeamStore;

#[component]
pub fn SubTeamPage(team_id: u64) -> impl IntoView {
    let team_store = expect_context::<TeamStore>();
    let (sub_teams, set_sub_teams) = create_signal(vec![]);
    let (loading, set_loading) = create_signal(false);

    // 加载子团队列表
    let load_sub_teams = move |_| {
        set_loading(true);
        let client = // 获取API客户端;
        
        spawn(async move {
            match list_sub_teams(&client, team_id).await {
                Ok(teams) => set_sub_teams.set(teams),
                Err(e) => log!("Error: {:?}", e),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="sub-team-page">
            <h2>"小组管理"</h2>
            
            <button on:click=load_sub_teams>"刷新"</button>
            
            <div class="sub-team-list">
                <For each=sub_teams>
                    |sub_team| {
                        view! {
                            <div class="sub-team-card">
                                <h3>{sub_team.sub_team_name}</h3>
                                <p>"成员数: " {sub_team.sub_team_members.len()}</p>
                                <p>"创建时间: " {sub_team.sub_team_create_time}</p>
                            </div>
                        }
                    }
                </For>
            </div>
        </div>
    }
}
```

### 8.2 子团队详情组件

创建 `frontend/src/components/sub_team_detail.rs`：

```rust
use leptos::*;
use crate::api::sub_team::*;

#[component]
pub fn SubTeamDetail(sub_team_id: u64) -> impl IntoView {
    let (sub_team, set_sub_team) = create_signal(None);
    let (members, set_members) = create_signal(vec![]);

    // 加载子团队详情和成员
    let load_data = move |_| {
        let client = // 获取API客户端;
        
        spawn(async move {
            // 加载子团队信息
            if let Ok(st) = get_sub_team(&client, sub_team_id).await {
                set_sub_team.set(Some(st));
            }
            
            // 加载成员列表
            if let Ok(m) = list_sub_team_members(&client, sub_team_id).await {
                set_members.set(m);
            }
        });
    };

    view! {
        <div class="sub-team-detail">
            <Show when=move { sub_team().is_some() }>
                <h2>{move { sub_team().unwrap().sub_team_name }}</h2>
            </Show>
            
            <div class="members-section">
                <h3>"成员列表"</h3>
                <For each=members>
                    |member| {
                        view! {
                            <div class="member-item">
                                <span>{member.username}</span>
                                <span>" Level: " {member.level}</span>
                            </div>
                        }
                    }
                </For>
            </div>
        </div>
    }
}
```

---

## 第九部分：权限验证（待实现）

当前实现缺失的权限验证逻辑：

### 9.1 创建小组时的验证

在 `create_sub_team` handler 中添加：

```rust
// 验证用户是否是团队成员
let is_member = DbTeam::check_team_membership(pool, team_id, actor_id).await?;
if !is_member {
    res.status_code(StatusCode::FORBIDDEN);
    res.render(Json(serde_json::json!({
        "error": "Only team members can create sub teams"
    })));
    return;
}

// 获取用户在团队中的级别
let member_level = DbTeam::get_member_level(pool, team_id, actor_id).await?;
let required_level = 10; // 可配置的阈值
if member_level < required_level {
    res.status_code(StatusCode::FORBIDDEN);
    res.render(Json(serde_json::json!({
        "error": "Insufficient level to create sub team"
    })));
    return;
}
```

### 9.2 添加成员时的验证

在 `add_sub_team_member` handler 中添加类似验证。

---

## API 端点总结

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/teams/{team_id}/subteams | 创建小组 |
| GET | /api/teams/{team_id}/subteams | 列出团队下的所有小组 |
| GET | /api/subteams/{sub_team_id} | 获取小组详情 |
| PUT | /api/subteams/{sub_team_id} | 更新小组信息 |
| DELETE | /api/subteams/{sub_team_id} | 删除小组 |
| GET | /api/subteams/{sub_team_id}/members | 列出小组成员 |
| POST | /api/subteams/{sub_team_id}/members | 添加成员 |
| DELETE | /api/subteams/{sub_team_id}/members/{user_id} | 移除成员 |
| PUT | /api/subteams/{sub_team_id}/members/{user_id}/role | 更新成员级别 |

---

## 注意事项

1. **数据一致性**：删除小组时，会自动从父团队的 `sub_team_ids` 中移除
2. **外键约束**：数据库层已设置级联删除，子团队删除会自动清理关联的成员数据
3. **权限控制**：建议在 handler 层添加权限验证，确保只有团队成员才能创建小组，且级别满足要求