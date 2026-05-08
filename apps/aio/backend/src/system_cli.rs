use anyhow::Result;

use crate::cli::{
    DepartmentSubcommand, DictGroupSubcommand, DictItemSubcommand, IdArgs, MenuCreateArgs,
    MenuSubcommand, MenuUpdateArgs, RoleCli, RoleSubcommand, SystemCli, SystemDocsArgs,
    SystemModuleKind, SystemSubcommand, UserCli, UserSubcommand,
};
use crate::services::system_management::{
    DepartmentUpsertDto, DictGroupUpsertDto, DictItemUpsertDto, MenuUpsertDto, RoleUpsertDto,
    UserUpsertDto, authorize_role_menus_on_server, authorize_user_roles_on_server,
    create_department_on_server, create_dict_group_on_server, create_dict_item_on_server,
    create_menu_on_server, create_role_on_server, create_user_on_server,
    delete_department_on_server, delete_dict_group_on_server, delete_dict_item_on_server,
    delete_menu_on_server, delete_role_on_server, delete_user_on_server, get_role_on_server,
    get_user_effective_menu_ids_on_server, get_user_on_server, list_departments_on_server,
    list_dict_groups_on_server, list_dict_items_on_server, list_menus_on_server,
    list_roles_on_server, list_users_on_server, update_department_on_server,
    update_dict_group_on_server, update_dict_item_on_server, update_menu_on_server,
    update_role_on_server, update_user_on_server,
};

pub async fn run_system_cli(cli: SystemCli) -> Result<()> {
    match cli.command {
        SystemSubcommand::Docs(args) => print_docs(args)?,
        SystemSubcommand::User(cli) => run_user_cli(cli).await?,
        SystemSubcommand::Role(cli) => run_role_cli(cli).await?,
        SystemSubcommand::Menu(cli) => run_menu_cli(cli).await?,
        SystemSubcommand::Department(cli) => run_department_cli(cli).await?,
        SystemSubcommand::DictGroup(cli) => run_dict_group_cli(cli).await?,
        SystemSubcommand::DictItem(cli) => run_dict_item_cli(cli).await?,
    }
    Ok(())
}

fn print_docs(args: SystemDocsArgs) -> Result<()> {
    let text = match args.module {
        Some(module) => module_doc(module),
        None => overview_doc(),
    };
    println!("{text}");
    Ok(())
}

fn overview_doc() -> String {
    [
        "# AIO System Agent CLI",
        "",
        "这是一组给 agent 调用的系统治理命令，不面向 MCP，也不要求人手点表单。",
        "设计原则：菜单是渐进式披露入口，每个模块同时携带局部提示词、CLI 操作面和边界说明。",
        "",
        "可用模块：",
        "- users: 用户管理",
        "- roles: 角色管理",
        "- navigation: 菜单挂载 / 导航治理",
        "- organization: 部门与组织上下文",
        "- metadata: 字典与元数据",
        "- security: 安全边界提示词与治理约束",
        "",
        "查看模块提示词：",
        "```bash",
        "aio system docs --module users",
        "```",
        "",
        "示例命令：",
        "```bash",
        "aio system user list",
        "aio system role create --name operator --description '日常运维'",
        "aio system dict-group list",
        "```",
    ]
    .join("\n")
}

fn module_doc(module: SystemModuleKind) -> String {
    match module {
        SystemModuleKind::Users => [
            "# 模块: 用户管理",
            "",
            "提示词：把用户管理模块整理成角色边界、状态流转、命令执行顺序和风险点。",
            "渐进式披露：",
            "1. 先列用户与角色现状。",
            "2. 再决定新建/更新/授权/删除。",
            "3. 最后输出验收与回滚检查。",
            "",
            "CLI：",
            "- `aio system user list`",
            "- `aio system user get --id 1`",
            "- `aio system user create --username demo --password secret --nickname 演示 --status active --role-id 2`",
            "- `aio system user update --id 3 --username demo --nickname 新名字 --status disabled --password new-secret`",
            "- `aio system user authorize-roles --id 3 --role-id 1 --role-id 2`",
            "- `aio system user effective-menus --id 3`",
            "- `aio system user delete --id 3`",
        ]
        .join("\n"),
        SystemModuleKind::Roles => [
            "# 模块: 角色管理",
            "",
            "提示词：把角色治理整理成权限矩阵、菜单授权覆盖和高风险授权边界。",
            "渐进式披露：",
            "1. 先列角色和菜单数。",
            "2. 再查看单角色菜单绑定。",
            "3. 最后执行授权或删除。",
            "",
            "CLI：",
            "- `aio system role list`",
            "- `aio system role get --id 2`",
            "- `aio system role create --name reviewer --description 审核员 --menu-id 5 --menu-id 8`",
            "- `aio system role update --id 2 --name reviewer --description 审核员`",
            "- `aio system role authorize-menus --id 2 --menu-id 5 --menu-id 8`",
            "- `aio system role delete --id 2`",
        ]
        .join("\n"),
        SystemModuleKind::Navigation => [
            "# 模块: 菜单挂载 / 导航治理",
            "",
            "提示词：把导航管理整理成二维上下文树、菜单可见性和权限码治理。",
            "渐进式披露：",
            "1. 先列当前菜单树。",
            "2. 再做挂载或排序调整。",
            "3. 最后检查 permission_code 和可见性。",
            "",
            "CLI：",
            "- `aio system menu list`",
            "- `aio system menu create --name 用户管理 --route /system/users --icon users --parent-id 6 --sort-order 10 --visible true --permission-code sys:user:view --menu-type page`",
            "- `aio system menu update --id 11 --name 用户管理 --route /system/users --icon users --parent-id 6 --sort-order 10 --visible true --permission-code sys:user:view --menu-type page`",
            "- `aio system menu delete --id 11`",
        ]
        .join("\n"),
        SystemModuleKind::Organization => [
            "# 模块: 组织上下文",
            "",
            "提示词：把组织上下文整理成层级关系、命名规范、隔离边界和同步顺序。",
            "CLI：",
            "- `aio system department list`",
            "- `aio system department create --name 华东区 --parent-id 1 --sort-order 20`",
            "- `aio system department update --id 4 --name 华东一区 --parent-id 1 --sort-order 30`",
            "- `aio system department delete --id 4`",
        ]
        .join("\n"),
        SystemModuleKind::Metadata => [
            "# 模块: 字典与元数据",
            "",
            "提示词：把字典与元数据整理成 group/item 结构、枚举约束和发布影响面。",
            "CLI：",
            "- `aio system dict-group list`",
            "- `aio system dict-group create --name user_status --description 用户状态`",
            "- `aio system dict-group update --id 2 --name user_status --description 用户状态字典`",
            "- `aio system dict-group delete --id 2`",
            "- `aio system dict-item list --group-id 2`",
            "- `aio system dict-item create --group-id 2 --label 启用 --value active --sort-order 10`",
            "- `aio system dict-item update --id 5 --group-id 2 --label 停用 --value disabled --sort-order 20`",
            "- `aio system dict-item delete --id 5`",
        ]
        .join("\n"),
        SystemModuleKind::Security => [
            "# 模块: 安全边界",
            "",
            "提示词：整理脚本权限、外部调用审批、凭证托管和审计轨迹。",
            "说明：该模块当前以治理提示词为主，不直接暴露危险执行命令。",
            "建议：先用 `aio system docs --module users|roles|navigation` 拿实际操作面，再由 agent 输出执行计划。",
        ]
        .join("\n"),
    }
}

async fn run_user_cli(cli: UserCli) -> Result<()> {
    match cli.command {
        UserSubcommand::List => print_json(&list_users_on_server().await?)?,
        UserSubcommand::Get(IdArgs { id }) => print_json(&get_user_on_server(id).await?)?,
        UserSubcommand::Create(args) => {
            let user = create_user_on_server(UserUpsertDto {
                username: args.username,
                password: args.password,
                nickname: args.nickname,
                status: args.status,
            })
            .await?;
            if !args.role_ids.is_empty() {
                authorize_user_roles_on_server(user.id, args.role_ids).await?;
            }
            print_json(&get_user_on_server(user.id).await?)?;
        }
        UserSubcommand::Update(args) => {
            let user = update_user_on_server(
                args.id,
                UserUpsertDto {
                    username: args.username,
                    password: args.password.unwrap_or_default(),
                    nickname: args.nickname,
                    status: args.status,
                },
            )
            .await?;
            print_json(&get_user_on_server(user.id).await?)?;
        }
        UserSubcommand::Delete(IdArgs { id }) => {
            delete_user_on_server(id).await?;
            println!("{{\"deleted\":true,\"id\":{id}}}");
        }
        UserSubcommand::AuthorizeRoles(args) => {
            authorize_user_roles_on_server(args.id, args.role_ids).await?;
            print_json(&get_user_on_server(args.id).await?)?;
        }
        UserSubcommand::EffectiveMenus(IdArgs { id }) => {
            print_json(&get_user_effective_menu_ids_on_server(id).await?)?;
        }
    }
    Ok(())
}

async fn run_role_cli(cli: RoleCli) -> Result<()> {
    match cli.command {
        RoleSubcommand::List => print_json(&list_roles_on_server().await?)?,
        RoleSubcommand::Get(IdArgs { id }) => print_json(&get_role_on_server(id).await?)?,
        RoleSubcommand::Create(args) => {
            let role = create_role_on_server(RoleUpsertDto {
                name: args.name,
                description: args.description,
            })
            .await?;
            if !args.menu_ids.is_empty() {
                authorize_role_menus_on_server(role.id, args.menu_ids).await?;
            }
            print_json(&get_role_on_server(role.id).await?)?;
        }
        RoleSubcommand::Update(args) => {
            let role = update_role_on_server(
                args.id,
                RoleUpsertDto {
                    name: args.name,
                    description: args.description,
                },
            )
            .await?;
            print_json(&get_role_on_server(role.id).await?)?;
        }
        RoleSubcommand::Delete(IdArgs { id }) => {
            delete_role_on_server(id).await?;
            println!("{{\"deleted\":true,\"id\":{id}}}");
        }
        RoleSubcommand::AuthorizeMenus(args) => {
            authorize_role_menus_on_server(args.id, args.menu_ids).await?;
            print_json(&get_role_on_server(args.id).await?)?;
        }
    }
    Ok(())
}

async fn run_menu_cli(cli: crate::cli::MenuCli) -> Result<()> {
    match cli.command {
        MenuSubcommand::List => print_json(&list_menus_on_server().await?)?,
        MenuSubcommand::Create(args) => {
            print_json(&create_menu_on_server(menu_input_from_create(args)).await?)?;
        }
        MenuSubcommand::Update(args) => {
            print_json(&update_menu_on_server(args.id, menu_input_from_update(args)).await?)?;
        }
        MenuSubcommand::Delete(IdArgs { id }) => {
            delete_menu_on_server(id).await?;
            println!("{{\"deleted\":true,\"id\":{id}}}");
        }
    }
    Ok(())
}

async fn run_department_cli(cli: crate::cli::DepartmentCli) -> Result<()> {
    match cli.command {
        DepartmentSubcommand::List => print_json(&list_departments_on_server().await?)?,
        DepartmentSubcommand::Create(args) => {
            print_json(
                &create_department_on_server(DepartmentUpsertDto {
                    parent_id: args.parent_id,
                    name: args.name,
                    sort_order: args.sort_order,
                })
                .await?,
            )?;
        }
        DepartmentSubcommand::Update(args) => {
            print_json(
                &update_department_on_server(
                    args.id,
                    DepartmentUpsertDto {
                        parent_id: args.parent_id,
                        name: args.name,
                        sort_order: args.sort_order,
                    },
                )
                .await?,
            )?;
        }
        DepartmentSubcommand::Delete(IdArgs { id }) => {
            delete_department_on_server(id).await?;
            println!("{{\"deleted\":true,\"id\":{id}}}");
        }
    }
    Ok(())
}

async fn run_dict_group_cli(cli: crate::cli::DictGroupCli) -> Result<()> {
    match cli.command {
        DictGroupSubcommand::List => print_json(&list_dict_groups_on_server().await?)?,
        DictGroupSubcommand::Create(args) => {
            print_json(
                &create_dict_group_on_server(DictGroupUpsertDto {
                    name: args.name,
                    description: args.description,
                })
                .await?,
            )?;
        }
        DictGroupSubcommand::Update(args) => {
            print_json(
                &update_dict_group_on_server(
                    args.id,
                    DictGroupUpsertDto {
                        name: args.name,
                        description: args.description,
                    },
                )
                .await?,
            )?;
        }
        DictGroupSubcommand::Delete(IdArgs { id }) => {
            delete_dict_group_on_server(id).await?;
            println!("{{\"deleted\":true,\"id\":{id}}}");
        }
    }
    Ok(())
}

async fn run_dict_item_cli(cli: crate::cli::DictItemCli) -> Result<()> {
    match cli.command {
        DictItemSubcommand::List(args) => {
            print_json(&list_dict_items_on_server(args.group_id).await?)?
        }
        DictItemSubcommand::Create(args) => {
            print_json(
                &create_dict_item_on_server(DictItemUpsertDto {
                    group_id: args.group_id,
                    label: args.label,
                    value: args.value,
                    sort_order: args.sort_order,
                })
                .await?,
            )?;
        }
        DictItemSubcommand::Update(args) => {
            print_json(
                &update_dict_item_on_server(
                    args.id,
                    DictItemUpsertDto {
                        group_id: args.group_id,
                        label: args.label,
                        value: args.value,
                        sort_order: args.sort_order,
                    },
                )
                .await?,
            )?;
        }
        DictItemSubcommand::Delete(IdArgs { id }) => {
            delete_dict_item_on_server(id).await?;
            println!("{{\"deleted\":true,\"id\":{id}}}");
        }
    }
    Ok(())
}

fn menu_input_from_create(args: MenuCreateArgs) -> MenuUpsertDto {
    MenuUpsertDto {
        parent_id: args.parent_id,
        name: args.name,
        route: args.route,
        icon: args.icon,
        sort_order: args.sort_order,
        visible: args.visible,
        permission_code: args.permission_code,
        menu_type: args.menu_type,
    }
}

fn menu_input_from_update(args: MenuUpdateArgs) -> MenuUpsertDto {
    MenuUpsertDto {
        parent_id: args.parent_id,
        name: args.name,
        route: args.route,
        icon: args.icon,
        sort_order: args.sort_order,
        visible: args.visible,
        permission_code: args.permission_code,
        menu_type: args.menu_type,
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
