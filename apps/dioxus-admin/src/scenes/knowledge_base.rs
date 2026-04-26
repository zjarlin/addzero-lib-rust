use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, Field, ListItem, ResponsiveGrid, Stack, StatTile, Surface, SurfaceHeader,
};

#[component]
pub fn NotesScene() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库 / 笔记".to_string(),
            subtitle: "沉淀操作手册、复盘记录和高频 SOP。".to_string(),
        }
        Surface {
            SurfaceHeader {
                title: "笔记索引".to_string(),
                subtitle: "按主题归档，支持后续接全文检索。".to_string()
            }
            Stack {
                ListItem { title: "Agent 运维手册".to_string(), detail: "关键词: skill, sync, pg".to_string(), meta: "2 小时前更新".to_string() }
                ListItem { title: "部署复盘 2026-04".to_string(), detail: "故障窗口、回滚路径、检查单".to_string(), meta: "昨天".to_string() }
                ListItem { title: "会话策略模板".to_string(), detail: "登录态、角色域、审计字段".to_string(), meta: "3 天前".to_string() }
            }
        }
        Surface {
            SurfaceHeader {
                title: "新增笔记".to_string(),
                subtitle: "默认双栏录入，减少纵向滚动。".to_string()
            }
            ResponsiveGrid { columns: 2,
                Field { label: "标题".to_string(), value: "例如：配置变更 SOP".to_string() }
                Field { label: "标签".to_string(), value: "ops, agent, runbook".to_string() }
            }
        }
    }
}

#[component]
pub fn SoftwareScene() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库 / 软件".to_string(),
            subtitle: "管理工具清单、安装版本和依赖约束。".to_string(),
        }
        ResponsiveGrid { columns: 3,
            StatTile { label: "已登记软件".to_string(), value: "42".to_string(), detail: "含 CLI / GUI / 服务".to_string() }
            StatTile { label: "待升级".to_string(), value: "6".to_string(), detail: "已超过安全基线".to_string() }
            StatTile { label: "高风险依赖".to_string(), value: "2".to_string(), detail: "等待兼容性验证".to_string() }
        }
        Surface {
            SurfaceHeader {
                title: "软件台账".to_string(),
                subtitle: "按用途和运行环境做分组。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "软件" } th { "版本" } th { "用途" } th { "状态" } } }
                tbody {
                    tr { td { "Cursor" } td { "0.52.x" } td { "编码与代理协作" } td { "稳定" } }
                    tr { td { "Docker Desktop" } td { "4.40.x" } td { "本地容器编排" } td { "待升级" } }
                    tr { td { "cloudflared" } td { "2026.4" } td { "隧道与公网访问" } td { "稳定" } }
                }
            }
        }
    }
}

#[component]
pub fn ConfigFilesScene() -> Element {
    rsx! {
        ContentHeader {
            title: "知识库 / 系统配置文件".to_string(),
            subtitle: "集中登记关键配置路径与变更策略。".to_string(),
        }
        Surface {
            SurfaceHeader {
                title: "配置目录".to_string(),
                subtitle: "路径、负责人、最后变更时间一屏可见。".to_string()
            }
            table { class: "data-table",
                thead { tr { th { "路径" } th { "类别" } th { "负责人" } th { "备注" } } }
                tbody {
                    tr { td { "~/.cloudflared/" } td { "网络代理" } td { "ops" } td { "隧道 ingress 模板" } }
                    tr { td { "~/.agents/skills/" } td { "Agent 规则" } td { "ai-platform" } td { "skill 源文件目录" } }
                    tr { td { "/etc/hosts" } td { "系统网络" } td { "infra" } td { "本地域名映射" } }
                }
            }
        }
        Surface {
            SurfaceHeader {
                title: "配置登记".to_string(),
                subtitle: "新增配置时写入路径和回滚策略。".to_string()
            }
            ResponsiveGrid { columns: 2,
                Field { label: "配置路径".to_string(), value: "/path/to/config".to_string() }
                Field { label: "回滚方式".to_string(), value: "git revert + service reload".to_string() }
            }
        }
    }
}
