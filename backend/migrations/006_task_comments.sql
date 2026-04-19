-- 006_task_comments.sql
-- 任务评论表
-- 时间：2026-04-19

CREATE TABLE IF NOT EXISTS task_comments (
    comment_id BIGSERIAL PRIMARY KEY,
    task_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    parent_id BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT,
    CONSTRAINT chk_task_comment_created_at CHECK (created_at > 0),
    CONSTRAINT fk_task_comments_task FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE,
    CONSTRAINT fk_task_comments_user FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CONSTRAINT fk_task_comments_parent FOREIGN KEY (parent_id) REFERENCES task_comments(comment_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_task_comments_task_id ON task_comments(task_id);
CREATE INDEX IF NOT EXISTS idx_task_comments_user_id ON task_comments(user_id);
CREATE INDEX IF NOT EXISTS idx_task_comments_parent_id ON task_comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_task_comments_created_at ON task_comments(created_at);
