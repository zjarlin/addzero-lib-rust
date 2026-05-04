-- 菜单树表
CREATE TABLE IF NOT EXISTS admin_menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_path VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    icon VARCHAR(100),
    parent_id UUID REFERENCES admin_menus(id) ON DELETE CASCADE,
    sort_order INTEGER DEFAULT 0,
    visible BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_admin_menus_parent ON admin_menus(parent_id);
CREATE INDEX idx_admin_menus_route ON admin_menus(route_path);
CREATE INDEX idx_admin_menus_visible ON admin_menus(visible);

-- 权限表
CREATE TABLE IF NOT EXISTS admin_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    category VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_admin_permissions_name ON admin_permissions(name);
CREATE INDEX idx_admin_permissions_category ON admin_permissions(category);

-- 菜单权限关联表
CREATE TABLE IF NOT EXISTS admin_menu_permissions (
    menu_id UUID NOT NULL REFERENCES admin_menus(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES admin_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (menu_id, permission_id)
);

-- 资源表（技能/dotfiles）
CREATE TABLE IF NOT EXISTS admin_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    source_type VARCHAR(50) NOT NULL, -- 'codex', 'claude', 'custom', 'dotfile'
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_admin_resources_name ON admin_resources(name);
CREATE INDEX idx_admin_resources_category ON admin_resources(category);
CREATE INDEX idx_admin_resources_source_type ON admin_resources(source_type);

-- 部署路径表
CREATE TABLE IF NOT EXISTS admin_deployment_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES admin_resources(id) ON DELETE CASCADE,
    tool_name VARCHAR(50) NOT NULL, -- 'codex', 'claude', 'vscode', etc.
    path TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_admin_deployment_paths_resource ON admin_deployment_paths(resource_id);
CREATE INDEX idx_admin_deployment_paths_tool ON admin_deployment_paths(tool_name);

-- 资源权限关联表
CREATE TABLE IF NOT EXISTS admin_resource_permissions (
    resource_id UUID NOT NULL REFERENCES admin_resources(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES admin_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (resource_id, permission_id)
);

-- 插入默认权限
INSERT INTO admin_permissions (name, description, category) VALUES
('menu.view', 'View menu', 'menu'),
('menu.edit', 'Edit menu', 'menu'),
('menu.delete', 'Delete menu', 'menu'),
('skill.view', 'View skills', 'skill'),
('skill.create', 'Create skills', 'skill'),
('skill.edit', 'Edit skills', 'skill'),
('skill.delete', 'Delete skills', 'skill'),
('skill.deploy', 'Deploy skills', 'skill'),
('resource.view', 'View resources', 'resource'),
('resource.manage', 'Manage resources', 'resource')
ON CONFLICT (name) DO NOTHING;

-- 插入默认菜单根节点
INSERT INTO admin_menus (route_path, title, icon, parent_id, sort_order, visible, metadata) VALUES
('/admin', 'Admin', 'LayoutDashboard', NULL, 0, true, '{"category": "system"}'::jsonb),
('/admin/skills', 'Skills', 'Brain', NULL, 1, true, '{"category": "management"}'::jsonb),
('/admin/resources', 'Resources', 'FolderOpen', NULL, 2, true, '{"category": "management"}'::jsonb),
('/admin/settings', 'Settings', 'Settings', NULL, 3, true, '{"category": "system"}'::jsonb)
ON CONFLICT (route_path) DO NOTHING;
