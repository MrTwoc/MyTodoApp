-- 001_initial_schema.sql
-- 初始数据库结构设置 (SQLite 版本)
-- 包含所有核心表和索引
-- 初始化时间：2026-03-21 18:00:00

-- ============================================
-- 1. 用户表 (users)
-- ============================================
CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER PRIMARY KEY,
    user_username TEXT NOT NULL UNIQUE,
    user_password TEXT NOT NULL,
    user_email TEXT NOT NULL UNIQUE,
    user_reg_time INTEGER NOT NULL,
    user_phone TEXT NOT NULL UNIQUE,
    user_teams TEXT NOT NULL DEFAULT '[]',
    user_last_login_time INTEGER,
    user_description TEXT,
    user_avatar TEXT,
    user_status TEXT NOT NULL DEFAULT 'Active',
    user_settings TEXT NOT NULL DEFAULT '{"mode":"Single","theme":"Dark"}',
    CONSTRAINT chk_user_status CHECK (user_status IN ('Active', 'Inactive')),
    CONSTRAINT chk_user_reg_time CHECK (user_reg_time > 0)
);

-- ============================================
-- 2. 任务表 (tasks)
-- ============================================
CREATE TABLE IF NOT EXISTS tasks (
    task_id INTEGER PRIMARY KEY,
    task_name TEXT NOT NULL,
    task_description TEXT,
    task_keywords TEXT NOT NULL DEFAULT '[]',
    task_priority INTEGER NOT NULL DEFAULT 0,
    task_deadline INTEGER,
    task_complete_time INTEGER,
    task_status TEXT NOT NULL DEFAULT 'Active',
    task_create_time INTEGER NOT NULL,
    task_leader_id INTEGER NOT NULL,
    task_team_id INTEGER,
    task_update_time INTEGER,
    FOREIGN KEY (task_leader_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_task_status CHECK (task_status IN ('Active', 'Completed', 'Paused')),
    CONSTRAINT chk_task_priority CHECK (task_priority >= 0 AND task_priority <= 255),
    CONSTRAINT chk_task_create_time CHECK (task_create_time > 0)
);

-- ============================================
-- 3. 团队表 (teams)
-- ============================================
CREATE TABLE IF NOT EXISTS teams (
    team_id INTEGER PRIMARY KEY,
    team_name TEXT NOT NULL,
    team_leader_id INTEGER NOT NULL,
    team_create_time INTEGER NOT NULL,
    sub_team_ids TEXT NOT NULL DEFAULT '[]',
    team_settings TEXT NOT NULL DEFAULT '{"team_description":null,"team_visibility":"Private","team_status":"Active","team_avatar":null,"team_member_limit":100}',
    FOREIGN KEY (team_leader_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_team_create_time CHECK (team_create_time > 0)
);

-- ============================================
-- 4. 子团队表 (sub_teams)
-- ============================================
CREATE TABLE IF NOT EXISTS sub_teams (
    sub_team_id INTEGER PRIMARY KEY,
    sub_team_name TEXT NOT NULL,
    sub_team_leader_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    sub_team_create_time INTEGER NOT NULL,
    sub_team_description TEXT,
    FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE,
    FOREIGN KEY (sub_team_leader_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_sub_team_create_time CHECK (sub_team_create_time > 0)
);

-- ============================================
-- 5. 团队成员表 (team_members)
-- ============================================
CREATE TABLE IF NOT EXISTS team_members (
    team_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    join_time INTEGER NOT NULL,
    PRIMARY KEY (team_id, user_id),
    FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_team_member_level CHECK (level >= 1 AND level <= 255),
    CONSTRAINT chk_team_member_join_time CHECK (join_time > 0)
);

CREATE TABLE IF NOT EXISTS sub_team_members (
    sub_team_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    join_time INTEGER NOT NULL,
    PRIMARY KEY (sub_team_id, user_id),
    FOREIGN KEY (sub_team_id) REFERENCES sub_teams(sub_team_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_sub_team_member_level CHECK (level >= 1 AND level <= 255),
    CONSTRAINT chk_sub_team_member_join_time CHECK (join_time > 0)
);

-- ============================================
-- 6. 团队邀请表 (team_invites)
-- ============================================
CREATE TABLE IF NOT EXISTS team_invites (
    invite_id INTEGER PRIMARY KEY,
    team_id INTEGER NOT NULL,
    inviter_id INTEGER NOT NULL,
    invitee_id TEXT NOT NULL DEFAULT '[]',
    create_time INTEGER NOT NULL,
    expire_time INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE,
    FOREIGN KEY (inviter_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT chk_invite_status CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    CONSTRAINT chk_invite_create_time CHECK (create_time > 0),
    CONSTRAINT chk_invite_expire_time CHECK (expire_time > 0)
);

-- ============================================
-- 7. 团队加入申请表 (join_requests)
-- ============================================
CREATE TABLE IF NOT EXISTS join_requests (
    request_id INTEGER PRIMARY KEY,
    team_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    request_time INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    review_time INTEGER,
    reviewer_id INTEGER,
    review_message TEXT,
    FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    FOREIGN KEY (reviewer_id) REFERENCES users(user_id) ON DELETE SET NULL,
    CONSTRAINT chk_request_status CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    CONSTRAINT chk_request_time CHECK (request_time > 0)
);

-- ============================================
-- 8. 用户日志表 (user_logs)
-- ============================================
CREATE TABLE IF NOT EXISTS user_logs (
    log_id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- ============================================
-- 9. 任务日志表 (task_logs)
-- ============================================
CREATE TABLE IF NOT EXISTS task_logs (
    log_id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL,
    operator_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    details TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE,
    FOREIGN KEY (operator_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- ============================================
-- 10. 团队日志表 (team_logs)
-- ============================================
CREATE TABLE IF NOT EXISTS team_logs (
    log_id INTEGER PRIMARY KEY,
    team_id INTEGER NOT NULL,
    operator_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id INTEGER,
    details TEXT,
    created_at INTEGER NOT NULL,
    ip_address TEXT,
    FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE,
    FOREIGN KEY (operator_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- ============================================
-- 索引创建
-- ============================================

-- 用户表索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(user_email);
CREATE INDEX IF NOT EXISTS idx_users_phone ON users(user_phone);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(user_status);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(user_username);

-- 任务表索引
CREATE INDEX IF NOT EXISTS idx_tasks_leader ON tasks(task_leader_id);
CREATE INDEX IF NOT EXISTS idx_tasks_team ON tasks(task_team_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(task_status);
CREATE INDEX IF NOT EXISTS idx_tasks_deadline ON tasks(task_deadline);
CREATE INDEX IF NOT EXISTS idx_tasks_create_time ON tasks(task_create_time);
CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(task_priority);

-- 团队表索引
CREATE INDEX IF NOT EXISTS idx_teams_leader ON teams(team_leader_id);
CREATE INDEX IF NOT EXISTS idx_teams_create_time ON teams(team_create_time);
CREATE INDEX IF NOT EXISTS idx_teams_settings ON teams(team_settings);

-- 子团队表索引
CREATE INDEX IF NOT EXISTS idx_sub_teams_team ON sub_teams(team_id);
CREATE INDEX IF NOT EXISTS idx_sub_teams_leader ON sub_teams(sub_team_leader_id);

-- 子团队成员表索引
CREATE INDEX IF NOT EXISTS idx_sub_team_members_user ON sub_team_members(user_id);
CREATE INDEX IF NOT EXISTS idx_sub_team_members_level ON sub_team_members(level);
CREATE INDEX IF NOT EXISTS idx_sub_team_members_join_time ON sub_team_members(join_time);

-- 团队成员表索引
CREATE INDEX IF NOT EXISTS idx_team_members_user ON team_members(user_id);
CREATE INDEX IF NOT EXISTS idx_team_members_level ON team_members(level);
CREATE INDEX IF NOT EXISTS idx_team_members_join_time ON team_members(join_time);

-- 团队邀请表索引
CREATE INDEX IF NOT EXISTS idx_team_invites_team ON team_invites(team_id);
CREATE INDEX IF NOT EXISTS idx_team_invites_inviter ON team_invites(inviter_id);
CREATE INDEX IF NOT EXISTS idx_team_invites_status ON team_invites(status);
CREATE INDEX IF NOT EXISTS idx_team_invites_expire_time ON team_invites(expire_time);

-- 团队加入申请表索引
CREATE INDEX IF NOT EXISTS idx_join_requests_team ON join_requests(team_id);
CREATE INDEX IF NOT EXISTS idx_join_requests_user ON join_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_join_requests_status ON join_requests(status);
CREATE INDEX IF NOT EXISTS idx_join_requests_reviewer ON join_requests(reviewer_id);

-- 用户日志表索引
CREATE INDEX IF NOT EXISTS idx_user_logs_user ON user_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_logs_action ON user_logs(action);
CREATE INDEX IF NOT EXISTS idx_user_logs_created_at ON user_logs(created_at);

-- 任务日志表索引
CREATE INDEX IF NOT EXISTS idx_task_logs_task ON task_logs(task_id);
CREATE INDEX IF NOT EXISTS idx_task_logs_operator ON task_logs(operator_id);
CREATE INDEX IF NOT EXISTS idx_task_logs_action ON task_logs(action);
CREATE INDEX IF NOT EXISTS idx_task_logs_created_at ON task_logs(created_at);

-- 团队日志表索引
CREATE INDEX IF NOT EXISTS idx_team_logs_team ON team_logs(team_id);
CREATE INDEX IF NOT EXISTS idx_team_logs_operator ON team_logs(operator_id);
CREATE INDEX IF NOT EXISTS idx_team_logs_action ON team_logs(action);
CREATE INDEX IF NOT EXISTS idx_team_logs_target_type ON team_logs(target_type);
CREATE INDEX IF NOT EXISTS idx_team_logs_created_at ON team_logs(created_at);

-- ============================================
-- 注释说明
-- ============================================

-- 用户状态枚举值: Active, Inactive
-- 用户设置 (user_settings TEXT 存储 JSON):
--   mode: Single (单机模式) / Online (在线模式)
--   theme: Dark (暗色主题) / Light (亮色主题)

-- 任务状态枚举值: Active, Completed, Paused
-- 任务优先级: 0-255 (数字越大优先级越高)

-- 团队设置 (team_settings TEXT 存储 JSON):
--   team_description: 团队描述
--   team_visibility: Public (公开) / Private (私有)
--   team_status: Active (运行中) / Closed (已关闭)
--   team_avatar: 团队头像URL
--   team_member_limit: 团队成员上限 (1-65535)

-- 团队成员等级: 1-255 (数字越大权限越高)

-- 子团队无单独的状态，继承父团队状态

-- 团队邀请状态枚举值: Pending, Approved, Rejected
-- 团队加入申请状态枚举值: Pending, Approved, Rejected

-- 用户日志动作枚举值: Register, Login, Logout, PasswordChanged, EmailUpdated, PhoneUpdated, ProfileUpdated, AvatarUpdated

-- 任务日志动作枚举值: Created, Updated, Deleted, StatusChanged, PriorityChanged, DeadlineChanged, LeaderChanged, TeamChanged, CommentAdded, AttachmentAdded

-- 团队日志动作枚举值: MemberJoined, MemberLeft, MemberRemoved, MemberRoleChanged, TeamCreated, TeamUpdated, TeamClosed, SubTeamCreated, SubTeamDeleted, RequestApproved, RequestRejected, TaskCreated, TaskCompleted, TaskDeleted