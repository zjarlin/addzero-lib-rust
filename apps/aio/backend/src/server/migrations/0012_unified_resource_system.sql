-- 统一资源表（替代原有的 admin_resources）
CREATE TABLE IF NOT EXISTS unified_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(50) NOT NULL, -- 'skill', 'note', 'config', 'template', 'dotfile'
    category VARCHAR(100),
    source_path TEXT, -- 原始文件/目录路径
    metadata JSONB DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_unified_resources_type ON unified_resources(resource_type);
CREATE INDEX IF NOT EXISTS idx_unified_resources_category ON unified_resources(category);
CREATE INDEX IF NOT EXISTS idx_unified_resources_tags ON unified_resources USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_unified_resources_active ON unified_resources(is_active);

-- 资源部署目标表（支持多工具/多目录部署）
CREATE TABLE IF NOT EXISTS resource_deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES unified_resources(id) ON DELETE CASCADE,
    target_tool VARCHAR(50) NOT NULL, -- 'codex', 'claude', 'vscode', 'cursor', 'custom'
    target_path TEXT NOT NULL,
    deployment_config JSONB DEFAULT '{}',
    is_enabled BOOLEAN DEFAULT true,
    last_deployed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_resource_deployments_resource ON resource_deployments(resource_id);
CREATE INDEX IF NOT EXISTS idx_resource_deployments_tool ON resource_deployments(target_tool);
CREATE INDEX IF NOT EXISTS idx_resource_deployments_enabled ON resource_deployments(is_enabled);

-- 资源扫描源配置表（配置从哪些目录扫描不同类型的资源）
CREATE TABLE IF NOT EXISTS resource_scan_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    scan_path TEXT NOT NULL,
    tool_name VARCHAR(50), -- 关联的工具（codex, claude等）
    scan_pattern TEXT, -- glob 模式，如 "**/*.md", "**/SKILL.md"
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    last_scanned_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_resource_scan_sources_type ON resource_scan_sources(resource_type);
CREATE INDEX IF NOT EXISTS idx_resource_scan_sources_tool ON resource_scan_sources(tool_name);
CREATE INDEX IF NOT EXISTS idx_resource_scan_sources_active ON resource_scan_sources(is_active);

-- 资源分类表
CREATE TABLE IF NOT EXISTS resource_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    resource_type VARCHAR(50) NOT NULL,
    parent_id UUID REFERENCES resource_categories(id) ON DELETE CASCADE,
    icon VARCHAR(100),
    sort_order INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_resource_categories_type ON resource_categories(resource_type);
CREATE INDEX IF NOT EXISTS idx_resource_categories_parent ON resource_categories(parent_id);

-- 插入默认扫描源配置
INSERT INTO resource_scan_sources (name, resource_type, scan_path, tool_name, scan_pattern, metadata, is_active) VALUES
('Codex Skills', 'skill', '~/.codex/skills', 'codex', '**/SKILL.md', '{"format": "codex"}'::jsonb, true),
('Claude Skills', 'skill', '~/.claude/skills', 'claude', '**/*.md', '{"format": "claude"}'::jsonb, true),
('Knowledge Notes', 'note', '~/Documents/notes', NULL, '**/*.md', '{"format": "markdown"}'::jsonb, true),
('Dotfiles Configs', 'config', '~/.dotfiles', NULL, '**/*.yaml', '{"format": "yaml"}'::jsonb, false)
ON CONFLICT DO NOTHING;

-- 插入默认分类
INSERT INTO resource_categories (name, resource_type, parent_id, icon, sort_order, metadata) VALUES
('Development', 'skill', NULL, 'Code', 1, '{}'::jsonb),
('AI/ML', 'skill', NULL, 'Brain', 2, '{}'::jsonb),
('Productivity', 'skill', NULL, 'Zap', 3, '{}'::jsonb),
('System', 'config', NULL, 'Settings', 1, '{}'::jsonb),
('Personal', 'note', NULL, 'FileText', 1, '{}'::jsonb),
('Work', 'note', NULL, 'Briefcase', 2, '{}'::jsonb)
ON CONFLICT (name) DO NOTHING;
