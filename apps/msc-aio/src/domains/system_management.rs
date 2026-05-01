use dioxus::prelude::*;
use dioxus_components::{DataTable, MetricStrip, SectionHeader, StatTile, Surface, SurfaceHeader};

#[component]
pub fn SystemUsers() -> Element {
    rsx! {
        SectionHeader {
            title: "系统管理".to_string(),
            subtitle: "系统域：用户、菜单、角色、部门分模块维护。".to_string(),
            eyebrow: "System".to_string()
        }
        UsersScene {}
    }
}

#[component]
pub fn SystemMenus() -> Element {
    rsx! {
        SectionHeader {
            title: "系统管理".to_string(),
            subtitle: "菜单管理单独成模块，不与用户角色混杂。".to_string(),
            eyebrow: "System".to_string()
        }
        MenusScene {}
    }
}

#[component]
pub fn SystemRoles() -> Element {
    rsx! {
        SectionHeader {
            title: "系统管理".to_string(),
            subtitle: "角色管理聚焦权限域，与部门管理解耦。".to_string(),
            eyebrow: "System".to_string()
        }
        RolesScene {}
    }
}

#[component]
pub fn SystemDepartments() -> Element {
    rsx! {
        SectionHeader {
            title: "系统管理".to_string(),
            subtitle: "部门组织结构独立建模，支撑用户归属。".to_string(),
            eyebrow: "System".to_string()
        }
        DepartmentsScene {}
    }
}

#[component]
pub fn SystemDictionaries() -> Element {
    rsx! {
        SectionHeader {
            title: "系统管理".to_string(),
            subtitle: "字典分组与字典项独立维护，支撑表单枚举、状态机和值域配置。".to_string(),
            eyebrow: "System".to_string()
        }
        DictionariesScene {}
    }
}

#[component]
pub fn UsersScene() -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "用户列表".to_string(),
                subtitle: "后续可接分页、筛选、批量禁用。".to_string()
            }
            DataTable {
                columns: vec!["用户".to_string(), "部门".to_string(), "角色".to_string(), "状态".to_string()],
                tr { td { "zjarlin" } td { "平台工程" } td { "管理员" } td { "启用" } }
                tr { td { "luna" } td { "风控产品" } td { "审核员" } td { "启用" } }
                tr { td { "mika" } td { "履约运营" } td { "操作员" } td { "锁定" } }
            }
        }
    }
}

#[component]
pub fn MenusScene() -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "菜单树".to_string(),
                subtitle: "保持后台常规比例，导航本体优先。".to_string()
            }
            DataTable {
                columns: vec!["菜单名".to_string(), "路由".to_string(), "父级".to_string(), "排序".to_string()],
                tr { td { "Skills" } td { "/agents" } td { "后台" } td { "30" } }
                tr { td { "系统管理 / Agent 节点" } td { "/system/agent-nodes" } td { "系统管理" } td { "48" } }
                tr { td { "知识库 / 笔记" } td { "/knowledge/notes" } td { "知识库" } td { "10" } }
                tr { td { "系统管理 / 用户" } td { "/system/users" } td { "系统管理" } td { "10" } }
                tr { td { "系统管理 / 字典管理" } td { "/system/dictionaries" } td { "系统管理" } td { "45" } }
                tr { td { "系统管理 / 系统设置" } td { "/system/settings" } td { "系统管理" } td { "50" } }
            }
        }
    }
}

#[component]
pub fn RolesScene() -> Element {
    rsx! {
        MetricStrip { columns: 3,
            StatTile { label: "角色总数".to_string(), value: "7".to_string(), detail: "含系统预置角色".to_string() }
            StatTile { label: "自定义角色".to_string(), value: "3".to_string(), detail: "按业务线拆分".to_string() }
            StatTile { label: "待审核变更".to_string(), value: "1".to_string(), detail: "涉及菜单权限收敛".to_string() }
        }
        Surface {
            SurfaceHeader {
                title: "角色清单".to_string(),
                subtitle: "角色与关键权限摘要。".to_string()
            }
            DataTable {
                columns: vec!["角色".to_string(), "权限范围".to_string(), "成员数".to_string(), "更新时间".to_string()],
                tr { td { "管理员" } td { "全域" } td { "2" } td { "今天" } }
                tr { td { "审核员" } td { "审计 + 发布审批" } td { "4" } td { "昨天" } }
                tr { td { "操作员" } td { "场景录入 + 查询" } td { "11" } td { "3 天前" } }
            }
        }
    }
}

#[component]
pub fn DepartmentsScene() -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: "部门结构".to_string(),
                subtitle: "组织关系作为用户与角色的基础维度。".to_string()
            }
            DataTable {
                columns: vec!["部门".to_string(), "上级部门".to_string(), "负责人".to_string(), "成员数".to_string()],
                tr { td { "平台工程" } td { "技术中心" } td { "zjarlin" } td { "8" } }
                tr { td { "风控产品" } td { "产品中心" } td { "luna" } td { "5" } }
                tr { td { "履约运营" } td { "运营中心" } td { "mika" } td { "6" } }
            }
        }
    }
}

#[component]
pub fn DictionariesScene() -> Element {
    rsx! {
        MetricStrip { columns: 3,
            StatTile {
                label: "字典分类".to_string(),
                value: "12".to_string(),
                detail: "覆盖状态、级别、来源等基础枚举".to_string()
            }
            StatTile {
                label: "启用条目".to_string(),
                value: "148".to_string(),
                detail: "用于表单选项和策略判定".to_string()
            }
            StatTile {
                label: "待发布变更".to_string(),
                value: "2".to_string(),
                detail: "含一个新增枚举、一个排序调整".to_string()
            }
        }

        Surface {
            SurfaceHeader {
                title: "字典分组".to_string(),
                subtitle: "先管分组边界，再管每个分组内的值域。".to_string()
            }
            DataTable {
                columns: vec!["字典名".to_string(), "编码".to_string(), "用途".to_string(), "条目数".to_string(), "状态".to_string()],
                tr { td { "用户状态" } td { "user_status" } td { "用户启停、锁定" } td { "3" } td { "启用" } }
                tr { td { "发布渠道" } td { "publish_channel" } td { "知识资产分发来源" } td { "5" } td { "启用" } }
                tr { td { "风险等级" } td { "risk_level" } td { "审批与审计分级" } td { "4" } td { "草稿" } }
            }
        }

        Surface {
            SurfaceHeader {
                title: "条目预览".to_string(),
                subtitle: "当前展示一个分组下的值、标签、默认态和排序。".to_string()
            }
            DataTable {
                columns: vec!["所属字典".to_string(), "键值".to_string(), "显示文案".to_string(), "默认".to_string(), "排序".to_string()],
                tr { td { "用户状态" } td { "enabled" } td { "启用" } td { "是" } td { "10" } }
                tr { td { "用户状态" } td { "disabled" } td { "停用" } td { "否" } td { "20" } }
                tr { td { "用户状态" } td { "locked" } td { "锁定" } td { "否" } td { "30" } }
            }
        }
    }
}
