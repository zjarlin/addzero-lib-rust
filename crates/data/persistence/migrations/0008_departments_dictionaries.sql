-- 部门管理
CREATE TABLE IF NOT EXISTS sys_department (
    id SERIAL PRIMARY KEY,
    parent_id INTEGER REFERENCES sys_department(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 字典分组
CREATE TABLE IF NOT EXISTS sys_dict_group (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 字典项
CREATE TABLE IF NOT EXISTS sys_dict_item (
    id SERIAL PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES sys_dict_group(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    value TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed 部门
INSERT INTO sys_department (id, parent_id, name, sort_order) VALUES
    (1, NULL, '总部', 10),
    (2, 1, '技术部', 10),
    (3, 1, '运营部', 20)
ON CONFLICT (id) DO NOTHING;
SELECT setval('sys_department_id_seq', (SELECT COALESCE(MAX(id), 1) FROM sys_department));

-- Seed 字典示例
INSERT INTO sys_dict_group (name, description) VALUES
    ('用户状态', '用户账户状态枚举')
ON CONFLICT (name) DO NOTHING;

INSERT INTO sys_dict_item (group_id, label, value, sort_order)
SELECT g.id, v.label, v.value, v.sort_order
FROM sys_dict_group g
CROSS JOIN (VALUES
    ('启用', 'enabled', 10),
    ('停用', 'disabled', 20),
    ('锁定', 'locked', 30)
) AS v(label, value, sort_order)
WHERE g.name = '用户状态'
ON CONFLICT DO NOTHING;
