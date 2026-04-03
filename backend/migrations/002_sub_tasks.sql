-- 002_sub_tasks.sql
-- 子任务表和索引

CREATE TABLE IF NOT EXISTS sub_tasks (
    sub_task_id BIGSERIAL PRIMARY KEY,
    task_id BIGINT NOT NULL,
    sub_task_name TEXT NOT NULL,
    sub_task_description TEXT,
    sub_task_status TEXT NOT NULL DEFAULT 'Active',
    sub_task_create_time BIGINT NOT NULL,
    sub_task_update_time BIGINT,
    sub_task_complete_time BIGINT,
    CONSTRAINT chk_sub_task_status CHECK (sub_task_status IN ('Active', 'Completed', 'Paused')),
    CONSTRAINT chk_sub_task_create_time CHECK (sub_task_create_time > 0),
    CONSTRAINT chk_sub_task_update_time CHECK (sub_task_update_time IS NULL OR sub_task_update_time >= sub_task_create_time),
    CONSTRAINT chk_sub_task_complete_time CHECK (sub_task_complete_time IS NULL OR sub_task_complete_time >= sub_task_create_time),
    CONSTRAINT fk_sub_tasks_task FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sub_tasks_task_id ON sub_tasks(task_id);
CREATE INDEX IF NOT EXISTS idx_sub_tasks_status ON sub_tasks(sub_task_status);
