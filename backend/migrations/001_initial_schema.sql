-- 001_initial_schema.sql
-- PostgreSQL 初始数据库结构设置
-- 初始化时间：2026-03-22

-- ============================================
-- 1. 用户表 (users)
-- ============================================
CREATE TABLE IF NOT EXISTS users (
    user_id BIGSERIAL PRIMARY KEY,
    user_username TEXT NOT NULL UNIQUE,
    user_password TEXT NOT NULL,
    user_email TEXT NOT NULL UNIQUE,
    user_reg_time BIGINT NOT NULL,
    user_phone TEXT NOT NULL UNIQUE,
    user_teams JSONB NOT NULL DEFAULT '[]',
    user_last_login_time BIGINT,
    user_description TEXT,
    user_avatar TEXT,
    user_status TEXT NOT NULL DEFAULT 'Active',
    user_settings JSONB NOT NULL DEFAULT '{"mode":"Single","theme":"Dark"}',
    CONSTRAINT chk_user_status CHECK (user_status IN ('Active', 'Inactive')),
    CONSTRAINT chk_user_reg_time CHECK (user_reg_time > 0)
);

-- ============================================
-- 2. 任务表 (tasks)
-- ============================================
CREATE TABLE IF NOT EXISTS tasks (
    task_id BIGSERIAL PRIMARY KEY,
    task_name TEXT NOT NULL,
    task_description TEXT,
    task_keywords JSONB NOT NULL DEFAULT '[]',
    task_priority INTEGER NOT NULL DEFAULT 0,
    task_deadline BIGINT,
    task_complete_time BIGINT,
    task_status TEXT NOT NULL DEFAULT 'Active',
    task_create_time BIGINT NOT NULL,
    task_leader_id BIGINT NOT NULL,
    task_team_id BIGINT,
    task_update_time BIGINT,
    CONSTRAINT chk_task_status CHECK (task_status IN ('Active', 'Completed', 'Paused')),
    CONSTRAINT chk_task_priority CHECK (task_priority >= 0 AND task_priority <= 255),
    CONSTRAINT chk_task_create_time CHECK (task_create_time > 0)
);

-- ============================================
-- 3. 团队表 (teams)
-- ============================================
CREATE TABLE IF NOT EXISTS teams (
    team_id BIGSERIAL PRIMARY KEY,
    team_name TEXT NOT NULL,
    team_leader_id BIGINT NOT NULL,
    team_create_time BIGINT NOT NULL,
    sub_team_ids JSONB NOT NULL DEFAULT '[]',
    team_settings JSONB NOT NULL DEFAULT '{"team_description":null,"team_visibility":"Private","team_status":"Active","team_avatar":null,"team_member_limit":100}',
    CONSTRAINT chk_team_create_time CHECK (team_create_time > 0)
);

-- ============================================
-- 4. 子团队表 (sub_teams)
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
-- 5. 团队成员表 (team_members)
-- ============================================
CREATE TABLE IF NOT EXISTS team_members (
    team_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    join_time BIGINT NOT NULL,
    PRIMARY KEY (team_id, user_id),
    CONSTRAINT chk_team_member_level CHECK (level >= 1 AND level <= 255),
    CONSTRAINT chk_team_member_join_time CHECK (join_time > 0)
);

-- ============================================
-- 5.1 子团队成员表 (sub_team_members)
-- ============================================
CREATE TABLE IF NOT EXISTS sub_team_members (
    sub_team_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    join_time BIGINT NOT NULL,
    PRIMARY KEY (sub_team_id, user_id),
    CONSTRAINT chk_sub_team_member_level CHECK (level >= 1 AND level <= 255),
    CONSTRAINT chk_sub_team_member_join_time CHECK (join_time > 0)
);

-- ============================================
-- 6. 团队邀请表 (team_invites)
-- ============================================
CREATE TABLE IF NOT EXISTS team_invites (
    team_id BIGINT NOT NULL,
    inviter_id BIGINT NOT NULL,
    invitee_ids JSONB NOT NULL DEFAULT '[]',
    create_time BIGINT NOT NULL,
    expire_time BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    CONSTRAINT chk_invite_status CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    CONSTRAINT chk_invite_create_time CHECK (create_time > 0),
    CONSTRAINT chk_invite_expire_time CHECK (expire_time > 0)
);

-- ============================================
-- 7. 团队加入申请表 (join_requests)
-- ============================================
CREATE TABLE IF NOT EXISTS join_requests (
    request_id BIGSERIAL PRIMARY KEY,
    team_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    request_time BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    review_time BIGINT,
    reviewer_id BIGINT,
    review_message TEXT,
    CONSTRAINT chk_request_status CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    CONSTRAINT chk_request_time CHECK (request_time > 0)
);

-- ============================================
-- 8. 用户日志表 (user_logs)
-- ============================================
CREATE TABLE IF NOT EXISTS user_logs (
    log_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at BIGINT NOT NULL
);

-- ============================================
-- 9. 任务日志表 (task_logs)
-- ============================================
CREATE TABLE IF NOT EXISTS task_logs (
    log_id BIGSERIAL PRIMARY KEY,
    task_id BIGINT NOT NULL,
    operator_id BIGINT NOT NULL,
    action TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    details TEXT,
    created_at BIGINT NOT NULL
);

-- ============================================
-- 10. 团队日志表 (team_logs)
-- ============================================
CREATE TABLE IF NOT EXISTS team_logs (
    log_id BIGSERIAL PRIMARY KEY,
    team_id BIGINT NOT NULL,
    operator_id BIGINT NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id BIGINT,
    details TEXT,
    created_at BIGINT NOT NULL,
    ip_address TEXT
);

-- ============================================
-- 外键约束（PostgreSQL需要显式添加）
-- ============================================

-- 团队表外键
ALTER TABLE teams 
ADD CONSTRAINT fk_teams_leader 
FOREIGN KEY (team_leader_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 子团队表外键
ALTER TABLE sub_teams 
ADD CONSTRAINT fk_sub_teams_team 
FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE;

ALTER TABLE sub_teams 
ADD CONSTRAINT fk_sub_teams_leader 
FOREIGN KEY (sub_team_leader_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 团队成员表外键
ALTER TABLE team_members 
ADD CONSTRAINT fk_team_members_team 
FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE;

ALTER TABLE team_members 
ADD CONSTRAINT fk_team_members_user 
FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 子团队成员表外键
ALTER TABLE sub_team_members 
ADD CONSTRAINT fk_sub_team_members_sub_team 
FOREIGN KEY (sub_team_id) REFERENCES sub_teams(sub_team_id) ON DELETE CASCADE;

ALTER TABLE sub_team_members 
ADD CONSTRAINT fk_sub_team_members_user 
FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 团队邀请表外键
ALTER TABLE team_invites 
ADD CONSTRAINT fk_team_invites_team 
FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE;

ALTER TABLE team_invites 
ADD CONSTRAINT fk_team_invites_inviter 
FOREIGN KEY (inviter_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 团队加入申请表外键
ALTER TABLE join_requests 
ADD CONSTRAINT fk_join_requests_team 
FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE;

ALTER TABLE join_requests 
ADD CONSTRAINT fk_join_requests_user 
FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

ALTER TABLE join_requests 
ADD CONSTRAINT fk_join_requests_reviewer 
FOREIGN KEY (reviewer_id) REFERENCES users(user_id) ON DELETE SET NULL;

-- 日志表外键
ALTER TABLE user_logs 
ADD CONSTRAINT fk_user_logs_user 
FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

ALTER TABLE task_logs 
ADD CONSTRAINT fk_task_logs_task 
FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE;

ALTER TABLE task_logs 
ADD CONSTRAINT fk_task_logs_operator 
FOREIGN KEY (operator_id) REFERENCES users(user_id) ON DELETE CASCADE;

ALTER TABLE team_logs 
ADD CONSTRAINT fk_team_logs_team 
FOREIGN KEY (team_id) REFERENCES teams(team_id) ON DELETE CASCADE;

ALTER TABLE team_logs
ADD CONSTRAINT fk_team_logs_operator
FOREIGN KEY (operator_id) REFERENCES users(user_id) ON DELETE CASCADE;

-- 任务表外键
ALTER TABLE tasks
ADD CONSTRAINT fk_tasks_leader
FOREIGN KEY (task_leader_id) REFERENCES users(user_id) ON DELETE CASCADE;

ALTER TABLE tasks
ADD CONSTRAINT fk_tasks_team
FOREIGN KEY (task_team_id) REFERENCES teams(team_id) ON DELETE SET NULL;

-- ============================================
-- 索引创建
-- ============================================

-- 用户表索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(user_email);
CREATE INDEX IF NOT EXISTS idx_users_phone ON users(user_phone);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(user_status);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(user_username);
CREATE INDEX IF NOT EXISTS idx_user_logs_user_id ON user_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_logs_created_at ON user_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_user_logs_user_created ON user_logs(user_id, created_at DESC);

-- JSONB 字段 GIN 索引
CREATE INDEX IF NOT EXISTS idx_users_teams_gin ON users USING GIN (user_teams);
CREATE INDEX IF NOT EXISTS idx_tasks_keywords_gin ON tasks USING GIN (task_keywords);

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

-- 团队成员表索引
CREATE INDEX IF NOT EXISTS idx_team_members_team ON team_members(team_id);
CREATE INDEX IF NOT EXISTS idx_team_members_user ON team_members(user_id);
CREATE INDEX IF NOT EXISTS idx_sub_team_members_sub_team ON sub_team_members(sub_team_id);
CREATE INDEX IF NOT EXISTS idx_sub_team_members_user ON sub_team_members(user_id);

-- 日志表索引
CREATE INDEX IF NOT EXISTS idx_task_logs_task ON task_logs(task_id);
CREATE INDEX IF NOT EXISTS idx_task_logs_operator ON task_logs(operator_id);
CREATE INDEX IF NOT EXISTS idx_team_logs_team ON team_logs(team_id);
CREATE INDEX IF NOT EXISTS idx_team_logs_operator ON team_logs(operator_id);