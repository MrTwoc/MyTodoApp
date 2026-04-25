-- 迁移脚本: 给 tasks 表添加 task_group_id 字段
-- 执行时间: 2026-04-25
-- 描述: 支持将团队任务指派给指定小组

-- 1. 添加 task_group_id 字段 (条件执行)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM information_schema.columns
        WHERE table_name = 'tasks' AND column_name = 'task_group_id'
    ) THEN
        ALTER TABLE tasks ADD COLUMN task_group_id BIGINT;
    END IF;
END $$;

-- 2. 添加外键约束 (如果不存在)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM pg_constraint
        WHERE conname = 'fk_tasks_task_group_id'
    ) THEN
        ALTER TABLE tasks
        ADD CONSTRAINT fk_tasks_task_group_id
        FOREIGN KEY (task_group_id) REFERENCES groups(group_id)
        ON DELETE SET NULL;
    END IF;
END $$;

-- 3. 创建索引
CREATE INDEX IF NOT EXISTS idx_tasks_task_group_id ON tasks(task_group_id);
CREATE INDEX IF NOT EXISTS idx_tasks_task_team_id ON tasks(task_team_id);
