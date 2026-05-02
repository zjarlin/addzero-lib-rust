-- System management: menus, roles, users and their associations.

CREATE TABLE IF NOT EXISTS sys_menu (
    id SERIAL PRIMARY KEY,
    parent_id INTEGER REFERENCES sys_menu(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    route TEXT NOT NULL DEFAULT '',
    icon TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sys_role (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sys_user (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    nickname TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'enabled',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sys_user_role (
    user_id INTEGER NOT NULL REFERENCES sys_user(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES sys_role(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS sys_role_menu (
    role_id INTEGER NOT NULL REFERENCES sys_role(id) ON DELETE CASCADE,
    menu_id INTEGER NOT NULL REFERENCES sys_menu(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, menu_id)
);

-- Seed: system admin role
INSERT INTO sys_role (name, description, is_system)
VALUES ('管理员', '拥有全部权限的系统内置角色', TRUE)
ON CONFLICT (name) DO NOTHING;

-- Seed: menu tree
INSERT INTO sys_menu (id, parent_id, name, route, icon, sort_order) VALUES
    (1, NULL, '总览', '/', 'dashboard', 10),
    (2, NULL, '知识库', '', 'book', 20),
    (3, 2, '笔记', '/knowledge/notes', 'note', 10),
    (4, 2, '安装包', '/knowledge/packages', 'package', 20),
    (5, 2, 'CLI Market', '/knowledge/cli-market', 'terminal', 30),
    (6, NULL, '系统管理', '', 'settings', 30),
    (7, 6, '用户', '/system/users', 'user', 10),
    (8, 6, '角色', '/system/roles', 'shield', 20),
    (9, 6, '菜单', '/system/menus', 'menu', 30),
    (10, 6, '部门', '/system/departments', 'building', 40),
    (11, 6, '字典', '/system/dictionaries', 'book-open', 50),
    (12, 6, 'Agent 节点', '/system/agent-nodes', 'server', 60),
    (13, 6, '系统设置', '/system/settings', 'cog', 70),
    (14, NULL, '审计日志', '/audit', 'clipboard', 40)
ON CONFLICT (id) DO NOTHING;

-- Reset sequence
SELECT setval('sys_menu_id_seq', (SELECT COALESCE(MAX(id), 1) FROM sys_menu));

-- Seed: admin user (plain password hash placeholder for dev)
INSERT INTO sys_user (username, password_hash, nickname)
VALUES ('admin', 'admin', '系统管理员')
ON CONFLICT (username) DO NOTHING;

-- Seed: bind admin role to all menus
INSERT INTO sys_role_menu (role_id, menu_id)
SELECT r.id, m.id FROM sys_role r CROSS JOIN sys_menu m WHERE r.name = '管理员'
ON CONFLICT DO NOTHING;

-- Seed: bind admin user to admin role
INSERT INTO sys_user_role (user_id, role_id)
SELECT u.id, r.id FROM sys_user u CROSS JOIN sys_role r WHERE u.username = 'admin' AND r.name = '管理员'
ON CONFLICT DO NOTHING;
