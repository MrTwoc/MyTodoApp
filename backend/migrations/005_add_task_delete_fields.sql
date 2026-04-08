-- 005_add_task_delete_fields.sql
-- PostgreSQL 迁移：添加任务删除相关字段
-- 时间：2026-04-08

-- ============================================
-- 为tasks表添加软删除字段
-- ============================================

-- 添加is_deleted字段（布尔型，默认为false）
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS is_deleted BOOLEAN NOT NULL DEFAULT false;

-- 添加deleted_at字段（时间戳，可为空）
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS deleted_at BIGINT;

-- 创建软删除相关索引
CREATE INDEX IF NOT EXISTS idx_tasks_is_deleted ON tasks(is_deleted);
CREATE INDEX IF NOT EXISTS idx_tasks_deleted_at ON tasks(deleted_at);