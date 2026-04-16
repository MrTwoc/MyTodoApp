# Tasks 页面重新设计方案

## 一、项目现状分析

### 1.1 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端框架 | Leptos (Dioxus) | Rust WASM 单页应用 |
| 后端 | Rust + Axum | REST API + WebSocket |
| 数据库 | PostgreSQL | 任务、子任务表已存在 |
| 样式 | 自定义 CSS | 全局 CSS 变量主题 |

### 1.2 现有任务相关模型

#### Task (backend/src/models/task.rs)

```rust
pub struct Task {
    pub task_id: u64,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_keywords: HashSet<String>,  // 标签
    pub task_priority: u8,             // 0-10
    pub task_difficulty: u8,             // 0-10
    pub task_deadline: Option<i64>,
    pub task_complete_time: Option<i64>,
    pub task_status: TaskStatus,         // Active/Completed/Paused
    pub task_create_time: i64,
    pub task_leader_id: u64,
    pub task_team_id: Option<u64>,
    pub task_update_time: Option<i64>,
    pub is_favorite: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<i64>,
}

pub enum TaskStatus {
    Active,    // 进行中
    Completed, // 已完成
    Paused,    // 已暂停
}
```

#### SubTask (后端已实现)

```rust
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
```

### 1.3 后端 API 现状

| API | 状态 | 说明 |
|-----|------|------|
| 子任务 CRUD | ✅ 已实现 | db_sub_task.rs |
| 任务状态更新 | ✅ 已实现 | 可直接使用 |
| 任务筛选 | ⚠️ 部分实现 | 需要扩展 |
| 编辑历史 | ❌ 未实现 | 需要新增 |

---

## 二、设计稿功能对照

### 2.1 布局结构

| 设计稿元素 | 现有实现 | 修改方案 |
|-----------|---------|--------|
| 导航轨道 (64px) | ❌ 无 | 新建 NavRail 组件 |
| 筛选面板 (250px) | ❌ 无 | 新建 FilterPanel 组件 |
| 主内容区 | ✅ kanban | 重构样式 |
| 统计条 | ❌ 无 | 新建 StatsBar 组件 |

### 2.2 Kanban 列

| 设计稿列 | 现有列 | 状态 |
|---------|--------|------|
| 待办 (To Do) | Active | ✅ 复用 |
| 进行中 (In Progress) | Paused / Active | 需新增 "InProgress" 状态 |
| 审核中 (Review) | ❌ 无 | 需新增 "Review" 状态 |
| 已完成 (Done) | Completed | ✅ 复用 |

**后端修改**: 需要添加 `TaskStatus::InReview` 枚举值

### 2.3 任务卡片

| 设计稿功能 | 现有实现 | 状态 |
|---------|--------|------|
| 优先级指示条 | 文本标签 | 样式改为指示条 |
| 难度点 (1-3) | 数值 (0-10) | 需转换显示 |
| 标签 (design/dev/research/bug/docs) | task_keywords | ✅ 已有 |
| 团队标签 | task_team_id | ✅ 已有 |
| 子任务进度条 | ❌ 无 | 需查询子任务 |
| 开始/截止时间 | task_deadline | 需新增 start_date |

**后端修改**: 需要添加 `task_start_date` 字段

### 2.4 筛选功能

| 设计稿筛选 | 现有筛选 | 状态 |
|-----------|---------|------|
| 优先级 (高/中/低) | task_priority 范围 | ✅ 复用 |
| 难度 (简单/中等/困难) | task_difficulty 范围 | ✅ 复用 |
| 类型 (团队/个人) | task_team_id | ✅ 复用 |
| 标签 | task_keywords | ✅ 复用 |
| 搜索 | search_query | ✅ 复用 |

### 2.5 任务详情模态框

| 设计稿功能 | 现有实现 | 状态 |
|-----------|---------|------|
| 查看模式 | 独���页面 | 改为模态框 |
| 编辑模式 | 页面内编辑 | 改为模态框 |
| 子任务管理 | ❌ 无 | 后端已有，前端未用 |
| 编辑历史 | ❌ 无 | 需要新增 |

---

## 三、修改步骤详细方案

### 步骤 0: 数据库迁移 (后端)

> **预览方式**: 无需预览，直接执行 SQL

```sql
-- 1. 添加任务开始时间字段
ALTER TABLE tasks ADD COLUMN task_start_date BIGINT;

-- 2. 添加 Review 状态 (PostgreSQL 枚举)
CREATE TYPE task_status_new AS ENUM ('Active', 'Paused', 'Completed', 'InReview');
ALTER TABLE tasks ALTER COLUMN task_status TYPE TASK_STATUS_NEW USING task_status::TEXT::TASK_STATUS_NEW;

-- 3. 创建任务编辑历史表
CREATE TABLE task_edit_history (
    id BIGINT PRIMARY KEY,
    task_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    user_name VARCHAR(100),
    user_avatar VARCHAR(255),
    edit_time BIGINT NOT NULL,
    action VARCHAR(20),  -- 'create', 'edit', 'move'
    field_name VARCHAR(50),
    old_value TEXT,
    new_value TEXT
);

-- 4. 创建子任务进度视图 (可选，优化查询)
CREATE VIEW task_subtask_progress AS
SELECT
    t.task_id,
    COUNT(st.sub_task_id) as total,
    COUNT(CASE WHEN st.sub_task_status = 'Completed' THEN 1 END) as done
FROM tasks t
LEFT JOIN sub_tasks st ON t.task_id = st.task_id
GROUP BY t.task_id;
```

---

### 步骤 1: 添加后端枚举和 API

> **预览方式**: 启动后端服务，检查 API 响应

**文件修改**:

1. `backend/src/models/task.rs` - 添加 `InReview` 状态
2. `backend/src/db/db_task.rs` - 添加状态转换逻辑
3. `backend/src/handlers/task_handler.rs` - 添加编辑历史 API

```rust
// backend/src/models/task.rs - 添加
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Active,
    InReview,  // 新增
    Completed,
    Paused,
}
```

**完成标志**: `GET /api/tasks` 返回的 task_status 包含 "InReview"

---

### 步骤 2: 前端样式初始化

> **预览方式**: 打开 Tasks 页面，检查控制台无报错

**文件修改**:

1. `frontend/src/styles/global.css` - 添加设计稿 CSS 变量

```css
/* 新增设计稿 CSS 变量 */
:root {
    /* 主色调 - 橙色强调 */
    --accent: #FF6B35;
    --accent-light: #FF8F5E;
    --accent-dim: rgba(255, 107, 53, 0.12);

    /* 背景 */
    --bg: #111111;
    --surface: #1A1A1A;
    --card: #1F1F1F;
    --card-hover: #272727;

    /* 边框 */
    --border: #2A2A2A;
    --border-light: #3A3A3A;

    /* 优先级颜色 */
    --priority-high: #EF4444;
    --priority-medium: #F59E0B;
    --priority-low: #10B981;

    /* 难度颜色 */
    --diff-easy: #34D399;
    --diff-medium: #FBBF24;
    --diff-hard: #F87171;

    /* 布局尺寸 */
    --nav-rail-width: 64px;
    --filter-panel-width: 250px;
}
```

**完成标志**: 页面加载无 CSS 错误

---

### 步骤 3: 统计条组件

> **预览方式**: Tasks 页面顶部显示 5 个统计卡片

**文件**: `frontend/src/components/stats_bar.rs` (新建)

```rust
#[component]
pub fn StatsBar() -> impl IntoView {
    let task_store = use_task_store();

    view! {
        <div class="stats-bar">
            <div class="stat-card">
                <div class="stat-icon" style="background: rgba(255,107,53,0.1); color: var(--accent);">
                    <i class="fa-solid fa-list-check"></i>
                </div>
                <div>
                    <div class="stat-value">{total}</div>
                    <div class="stat-label">全部</div>
                </div>
            </div>
            <!-- 其他 4 个统计卡片类似 -->
        </div>
    }
}
```

**完成标志**: 显示 5 个统计卡片，数字正确

---

### 步骤 4: 导航轨道组件

> **预览方式**: 左侧 64px 宽导航栏

**文件**: `frontend/src/components/nav_rail.rs` (新建)

```rust
#[component]
pub fn NavRail() -> impl IntoView {
    view! {
        <nav class="nav-rail">
            <div class="nav-rail-logo">
                <i class="fa-solid fa-layer-group"></i>
            </div>
            <button class="nav-rail-item active" data-nav="tasks">
                <i class="fa-solid fa-list-check"></i>
                <span class="rail-tooltip">Tasks</span>
            </button>
            <!-- 其他导航项 -->
        </nav>
    }
}
```

**完成标志**: 左侧显示垂直导航栏，悬停显示提示

---

### 步骤 5: 筛选面板组件

> **预览方式**: 导航栏右侧 250px 宽面板

**文件**: `frontend/src/components/filter_panel.rs` (新建)

```rust
#[component]
pub fn FilterPanel(
    #[prop(default = signal(false)) collapsed: Signal<bool>,
) -> impl IntoView {
    view! {
        <aside class="filter-panel" class:collapsed=move || collapsed.get()>
            <div class="filter-panel-header">
                <span>筛选与视图</span>
                <button class="filter-panel-toggle">
                    <i class="fa-solid fa-angles-left"></i>
                </button>
            </div>
            <div class="filter-nav">
                <div class="filter-nav-item active" data-view="board">
                    <i class="fa-solid fa-table-columns"></i> 看板视图
                </div>
                <div class="filter-nav-item" data-view="list">
                    <i class="fa-solid fa-list"></i> 列表视图
                </div>
            </div>
            <!-- 筛选器 -->
        </aside>
    }
}
```

**完成标志**: 显示筛选面板，可折叠

---

### 步骤 6: 重构 Kanban 看板

> **预览方式**: 4 列 Kanban 布局

**文件**: `frontend/src/components/kanban.rs`

**修改内容**:

1. 添加新列配置
2. 优先级指示条样式
3. 难度点显示
4. 子任务进度条

```rust
pub fn get_kanban_columns() -> Vec<KanbanColumnConfig> {
    vec![
        KanbanColumnConfig {
            status: TaskStatus::Active,
            title: "待办",
            color: "#FF6B35",
            icon: "fa-clipboard-list",
        },
        KanbanColumnConfig {
            status: TaskStatus::InReview,  // 新增
            title: "审核中",
            color: "#A78BFA",
            icon: "fa-eye",
        },
        KanbanColumnConfig {
            status: TaskStatus::Completed,
            title: "已完成",
            color: "#10B981",
            icon: "fa-circle-check",
        },
    ]
}
```

**完成标志**: 4 列 Kanban，任务正确分布

---

### 步骤 7: 任务卡片升级

> **预览方式**: 卡片显示优先级条、难度点、标签、子任务进度

**文件**: `frontend/src/components/kanban.rs` - KanbanCard

```rust
#[component]
pub fn KanbanCard(task: Task, subtasks: Vec<SubTask>) -> impl IntoView {
    // 优先级颜色映射
    let priority_color = match task.task_priority {
        0..=2 => "var(--priority-low)",
        3..=5 => "var(--priority-medium)",
        _ => "var(--priority-high)",
    };

    // 难度点
    let difficulty_dots = (0..task.task_difficulty.min(3))
        .map(|_| view! { <span class="diff-dot"></span> })
        .collect::<Vec<_>>();

    // 子任务进度
    let progress = if !subtasks.is_empty() {
        let done = subtasks.iter().filter(|s| s.sub_task_status == TaskStatus::Completed).count();
        let pct = (done * 100) / subtasks.len();
        view! {
            <div class="subtask-bar">
                <div class="subtask-bar-fill" style=format!("width: {}%", pct)></div>
            </div>
        }.into_any()
    } else {
        ().into_any()
    };
}
```

**完成标志**: 卡片显示设计稿中的所有元素

---

### 步骤 8: 任务详情模态框

> **预览方式**: 点击任务打开模态框

**文件**: `frontend/src/components/task_modal.rs` (新建)

```rust
#[component]
pub fn TaskModal(
    task: Task,
    subtasks: Vec<SubTask>,
    #[prop(default = signal(false)) editing: Signal<bool>,
    #[prop(default = signal(false)) history_open: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="modal-overlay">
            <div class="modal-content">
                <!-- 查看模式 -->
                <div class="modal-read-mode">
                    <!-- 标题、优先级、难度、标签、团队 -->
                    <!-- 时间信息 -->
                    <!-- 子任务列表 + 添加子任务 -->
                </div>

                <!-- 编辑模式 -->
                <div class="modal-edit-mode" class:hidden=move || !editing.get()>
                    <!-- 表单 -->
                </div>

                <!-- 历史面板 -->
                <div class="history-panel" class:hidden=move || !history_open.get()>
                    <!-- 编辑历史 -->
                </div>
            </div>
        </div>
    }
}
```

**完成标志**: 模态框正常显示，可切换编辑模式，可显示历史

---

### 步骤 9: 列表视图

> **预览方式**: 切换到列表视图

**文件**: `frontend/src/components/task_list.rs` (新建)

```rust
#[component]
pub fn TaskListView(tasks: Vec<Task>) -> impl IntoView {
    view! {
        <div class="list-view">
            <div class="list-header">
                <span></span>
                <span>名称</span>
                <span>优先级</span>
                <span>难度</span>
                <span>类型</span>
                <span>时间</span>
                <span>子任务</span>
                <span></span>
            </div>
            <!-- 任务行 -->
        </div>
    }
}
```

**完成标志**: 列表视图正常显示

---

### 步骤 10: 整合 TasksPage

> **预览方式**: 完整页面布局

**文件**: `frontend/src/pages/tasks.rs`

```rust
#[component]
pub fn TasksPage() -> impl IntoView {
    view! {
        <div class="app-layout">
            <NavRail />
            <FilterPanel collapsed=filter_collapsed />
            <div class="main-content">
                <StatsBar />
                <TopBar />  <!-- 搜索 + 新建任务按钮 -->
                {match view_mode.get() {
                    "board" => view! { <KanbanBoard tasks /> }.into_any(),
                    "list" => view! { <TaskListView tasks /> }.into_any(),
                    "starred" => view! { <StarredView tasks /> }.into_any(),
                }}
            </div>
            <TaskModal />
        </div>
    }
}
```

**完成标志**: 完整页面可用

---

## 四、后端未实现功能对照

| 设计稿功能 | 后端状态 | 待实现 |
|-----------|---------|--------|
| 任务开始时间 | ❌ 需添加字段 | task_start_date |
| 审核中状态 | ⚠️ 需扩展枚举 | TaskStatus::InReview |
| 编辑历史记录 | ❌ 需新增表 | task_edit_history |
| 子任务进度查询 | ⚠️ 已有子任务 | 需添加 API |

### 待新增 API

```rust
// 1. 更新任务状态 (包含 InReview)
PATCH /api/tasks/{id}/status
Body: { "status": "InReview" }

// 2. 获取子任务列表
GET /api/tasks/{id}/subtasks

// 3. 获取任务编辑历史
GET /api/tasks/{id}/history

// 4. 更新任务 (记录编辑历史)
PUT /api/tasks/{id}
Body: { "task_name": "...", "task_description": "...", ... }
```

---

## 五、文件修改清单

### 前端 (Leptos/Rust)

| 文件 | 操作 | 备注 |
|------|------|------|
| `frontend/src/styles/global.css` | 修改 | 添加设计稿 CSS 变量 |
| `frontend/src/components/nav_rail.rs` | 新建 | 导航轨道 |
| `frontend/src/components/filter_panel.rs` | 新建 | 筛选面板 |
| `frontend/src/components/stats_bar.rs` | 新建 | 统计条 |
| `frontend/src/components/kanban.rs` | 修改 | 重构看板和卡片 |
| `frontend/src/components/task_modal.rs` | 新建 | 详情模态框 |
| `frontend/src/components/task_list.rs` | 新建 | 列表视图 |
| `frontend/src/components/task_card.rs` | 修改 | 升级卡片样式 |
| `frontend/src/pages/tasks.rs` | 修改 | 整合新布局 |
| `frontend/src/store/task_store.rs` | 修改 | 添加新筛选器 |

### 后端 (Rust)

| 文件 | 操作 | 备注 |
|------|------|------|
| `backend/src/models/task.rs` | 修改 | 添加 InReview |
| `backend/src/db/db_task.rs` | 修改 | 状态处理 |
| `backend/src/handlers/task_handler.rs` | 修改 | 添加历史 API |
| `backend/src/services/task_service.rs` | 修改 | 业务逻辑 |

---

## 六、测试检查点

每个步骤完成后请检查:

| 步骤 | 检查点 |
|------|--------|
| 1 | 数据库迁移成功，无报错 |
| 2 | API 返回正确的状态枚举 |
| 3 | 页面加载无 CSS 错误 |
| 4 | 统计卡片显示正确的数字 |
| 5 | 导航栏显示且悬停有效 |
| 6 | 筛选面板可折叠 |
| 7 | Kanban 4 列正确显示 |
| 8 | 卡片显示所有设计元素 |
| 9 | 模态框打开/编辑/历史正常 |
| 10 | 列表视图正确显示 |
| 11 | 完整页面整合完成 |

---

## 七、预计工作量

| 阶段 | 预估工时 |
|------|---------|
| 后端修改 (状态/字段/历史) | 2-3 小时 |
| 前端组件开发 | 4-6 小时 |
| 整合调试 | 2-3 小时 |
| **总计** | **8-12 小时** |

---

*文档版本: v1.0*
*创建时间: 2026-04-16*