-- 迁移：菜单权限增强
-- 为 sys_menu 增加 permission_code（权限标识）和 menu_type（菜单类型：目录/菜单/按钮）

-- 1. 新增字段
ALTER TABLE sys_menu
    ADD COLUMN IF NOT EXISTS permission_code VARCHAR(128) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS menu_type VARCHAR(16) NOT NULL DEFAULT 'menu';

-- 2. 补充已有菜单的 menu_type（有 route 的视为 menu，无 route 且有 children 的视为 dir，叶子无 route 的视为 button）
UPDATE sys_menu SET menu_type = 'dir'
WHERE route = '' AND id IN (SELECT DISTINCT parent_id FROM sys_menu WHERE parent_id IS NOT NULL);

UPDATE sys_menu SET menu_type = 'button'
WHERE route = '' AND menu_type = 'menu';

-- 3. 为顶级菜单补 permission_code（按 name 生成，首版手工填）
-- 生产环境请根据实际菜单逐条设置，这里给出示例
UPDATE sys_menu SET permission_code = 'system'         WHERE name = '系统管理' AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:user'    WHERE name = '用户'     AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:menu'    WHERE name = '菜单'     AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:role'    WHERE name = '角色'     AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:dept'    WHERE name = '部门'     AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:dict'    WHERE name = '字典管理' AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:agent'   WHERE name = 'Agent 节点' AND permission_code = '';
UPDATE sys_menu SET permission_code = 'system:setting' WHERE name = '系统设置' AND permission_code = '';
UPDATE sys_menu SET permission_code = 'knowledge'       WHERE name = '知识库'   AND permission_code = '';
UPDATE sys_menu SET permission_code = 'knowledge:note'  WHERE name = '笔记'     AND permission_code = '';
UPDATE sys_menu SET permission_code = 'knowledge:skill' WHERE name = 'Skills'   AND permission_code = '';
UPDATE sys_menu SET permission_code = 'knowledge:pkg'   WHERE name = '安装包'   AND permission_code = '';
UPDATE sys_menu SET permission_code = 'knowledge:cli'   WHERE name = 'CLI 市场' AND permission_code = '';
UPDATE sys_menu SET permission_code = 'knowledge:dl'    WHERE name = '下载站'   AND permission_code = '';
UPDATE sys_menu SET permission_code = 'audit'           WHERE name = '审计日志' AND permission_code = '';
UPDATE sys_menu SET permission_code = 'overview'        WHERE name = '智能体工作台'     AND permission_code = '';

-- 4. 索引
CREATE INDEX IF NOT EXISTS idx_sys_menu_permission_code ON sys_menu(permission_code) WHERE permission_code != '';
CREATE INDEX IF NOT EXISTS idx_sys_menu_type ON sys_menu(menu_type);
