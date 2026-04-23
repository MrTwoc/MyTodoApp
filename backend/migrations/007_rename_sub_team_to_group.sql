-- 迁移脚本: 将 sub_team 相关表和字段重命名为 group
-- 执行时间: 2026-04-23
-- 描述: 将 sub_team 功能重命名为 group (小组)

-- 1. 重命名表 (条件执行)
DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'sub_teams') THEN
        ALTER TABLE sub_teams RENAME TO groups;
    END IF;
    IF EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'sub_team_members') THEN
        ALTER TABLE sub_team_members RENAME TO group_members;
    END IF;
END $$;

-- 2. 重命名 groups 表中的字段 (条件执行)
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'sub_team_id') THEN
        ALTER TABLE groups RENAME COLUMN sub_team_id TO group_id;
    END IF;
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'sub_team_name') THEN
        ALTER TABLE groups RENAME COLUMN sub_team_name TO group_name;
    END IF;
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'sub_team_leader_id') THEN
        ALTER TABLE groups RENAME COLUMN sub_team_leader_id TO group_leader_id;
    END IF;
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'sub_team_create_time') THEN
        ALTER TABLE groups RENAME COLUMN sub_team_create_time TO group_create_time;
    END IF;
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'sub_team_description') THEN
        ALTER TABLE groups RENAME COLUMN sub_team_description TO group_description;
    END IF;
END $$;

-- 3. 重命名 group_members 表中的字段 (条件执行)
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'group_members' AND column_name = 'sub_team_id') THEN
        ALTER TABLE group_members RENAME COLUMN sub_team_id TO group_id;
    END IF;
END $$;

-- 4. 更新 teams 表中的 group_ids 字段 (条件执行)
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'teams' AND column_name = 'sub_team_ids') THEN
        ALTER TABLE teams RENAME COLUMN sub_team_ids TO group_ids;
    END IF;
END $$;

-- 5. 创建索引 (使用 IF NOT EXISTS)
CREATE INDEX IF NOT EXISTS idx_groups_team_id ON groups(team_id);
CREATE INDEX IF NOT EXISTS idx_groups_group_leader_id ON groups(group_leader_id);
CREATE INDEX IF NOT EXISTS idx_group_members_group_id ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_group_members_user_id ON group_members(user_id);
