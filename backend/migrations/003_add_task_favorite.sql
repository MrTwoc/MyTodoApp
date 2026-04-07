-- 003_add_task_favorite.sql
-- 添加任务收藏功能
-- 时间：2026-04-07

ALTER TABLE tasks ADD COLUMN is_favorite BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_tasks_favorite ON tasks(is_favorite);