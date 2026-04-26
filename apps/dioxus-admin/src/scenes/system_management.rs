use dioxus::prelude::*;
use dioxus_components::{ContentHeader, ResponsiveGrid, StatTile, Surface, SurfaceHeader};

#[component]
pub fn UsersScene() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理 / 用户管理".to_string(),
            subtitle: "管理账号状态、归属部门和最后登录。".to_string(),
        }
        Surface {
            SurfaceHeader {
                title: "用户列表".to_string(),
                subtitle: "后续可接分页、筛选、批量禁用。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "用户" } th { "部门" } th { "角色" } th { "状态" } } }
                tbody {
                    tr { td { "zjarlin" } td { "平台工程" } td { "管理员" } td { "启用" } }
                    tr { td { "luna" } td { "风控产品" } td { "审核员" } td { "启用" } }
                    tr { td { "mika" } td { "履约运营" } td { "操作员" } td { "锁定" } }
                }
            }
        }
    }
}

#[component]
pub fn MenusScene() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理 / 菜单管理".to_string(),
            subtitle: "配置导航树、访问路径与可见性。".to_string(),
        }
        Surface {
            SurfaceHeader {
                title: "菜单树".to_string(),
                subtitle: "保持后台常规比例，导航本体优先。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "菜单名" } th { "路由" } th { "父级" } th { "排序" } } }
                tbody {
                    tr { td { "Agent 技能" } td { "/agents" } td { "后台" } td { "30" } }
                    tr { td { "知识库 / 笔记" } td { "/knowledge/notes" } td { "知识库" } td { "10" } }
                    tr { td { "系统管理 / 用户" } td { "/system/users" } td { "系统管理" } td { "10" } }
                }
            }
        }
    }
}

#[component]
pub fn RolesScene() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理 / 角色管理".to_string(),
            subtitle: "定义角色与权限边界，支撑菜单与接口授权。".to_string(),
        }
        ResponsiveGrid { columns: 3,
            StatTile { label: "角色总数".to_string(), value: "7".to_string(), detail: "含系统预置角色".to_string() }
            StatTile { label: "自定义角色".to_string(), value: "3".to_string(), detail: "按业务线拆分".to_string() }
            StatTile { label: "待审核变更".to_string(), value: "1".to_string(), detail: "涉及菜单权限收敛".to_string() }
        }
        Surface {
            SurfaceHeader {
                title: "角色清单".to_string(),
                subtitle: "角色与关键权限摘要。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "角色" } th { "权限范围" } th { "成员数" } th { "更新时间" } } }
                tbody {
                    tr { td { "管理员" } td { "全域" } td { "2" } td { "今天" } }
                    tr { td { "审核员" } td { "审计 + 发布审批" } td { "4" } td { "昨天" } }
                    tr { td { "操作员" } td { "场景录入 + 查询" } td { "11" } td { "3 天前" } }
                }
            }
        }
    }
}

#[component]
pub fn DepartmentsScene() -> Element {
    rsx! {
        ContentHeader {
            title: "系统管理 / 部门管理".to_string(),
            subtitle: "维护组织树与部门负责人映射。".to_string(),
        }
        Surface {
            SurfaceHeader {
                title: "部门结构".to_string(),
                subtitle: "组织关系作为用户与角色的基础维度。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "部门" } th { "上级部门" } th { "负责人" } th { "成员数" } } }
                tbody {
                    tr { td { "平台工程" } td { "技术中心" } td { "zjarlin" } td { "8" } }
                    tr { td { "风控产品" } td { "产品中心" } td { "luna" } td { "5" } }
                    tr { td { "履约运营" } td { "运营中心" } td { "mika" } td { "6" } }
                }
            }
        }
    }
}
