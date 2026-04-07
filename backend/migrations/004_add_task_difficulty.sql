-- 004_add_task_difficulty.sql
-- Add task difficulty field to tasks table
-- Date: 2026-04-07

ALTER TABLE tasks 
ADD COLUMN IF NOT EXISTS task_difficulty SMALLINT NOT NULL DEFAULT 0;

ALTER TABLE tasks 
ADD CONSTRAINT chk_task_difficulty CHECK (task_difficulty >= 0 AND task_difficulty <= 10);

CREATE INDEX IF NOT EXISTS idx_tasks_difficulty ON tasks(task_difficulty);