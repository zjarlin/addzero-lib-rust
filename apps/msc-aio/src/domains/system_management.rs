use dioxus::prelude::*;
use dioxus_components::{
    DataTable, MetricStrip, SectionHeader, StatTile, Surface, SurfaceHeader, WorkbenchButton,
};

use crate::services::{
    DepartmentDto, DepartmentUpsertDto, DictGroupDto, DictGroupUpsertDto, DictItemDto,
    DictItemUpsertDto, MenuDto, MenuUpsertDto, RoleDto, RoleUpsertDto, UserUpsertDto,
    UserWithRolesDto,
};
use crate::state::AppServices;

// ─── Top-level page components ──────────────────────────────────────────────

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

// ─── Users ──────────────────────────────────────────────────────────────────

#[component]
fn UsersScene() -> Element {
    let sys = use_context::<AppServices>().system.clone();
    let mut users_resource = use_resource(move || {
        let sys = sys.clone();
        async move { sys.list_users().await }
    });

    let sys2 = use_context::<AppServices>().system.clone();
    let mut roles_resource = use_resource(move || {
        let sys = sys2.clone();
        async move { sys.list_roles().await }
    });

    let sys3 = use_context::<AppServices>().system.clone();
    let mut menus_resource = use_resource(move || {
        let sys = sys3.clone();
        async move { sys.list_menus().await }
    });

    let mut editing = use_signal::<Option<UserUpsertDto>>(|| None);
    let mut editing_id = use_signal::<Option<i32>>(|| None);
    let mut form_username = use_signal(String::new);
    let mut form_nickname = use_signal(String::new);
    let mut form_password = use_signal(String::new);
    let mut form_status = use_signal(|| "enabled".to_string());
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut show_role_dialog = use_signal::<Option<i32>>(|| None);
    let mut selected_role_ids = use_signal::<Vec<i32>>(Vec::new);
    let mut confirm_delete_user = use_signal::<Option<i32>>(|| None);
    let mut show_perms_dialog = use_signal::<Option<(i32, String, Vec<i32>)>>(|| None);
    let mut perms_loading = use_signal(|| false);
    let mut user_menu_names = use_signal::<Vec<String>>(Vec::new);

    let _reload = move || {
        users_resource.restart();
    };

    rsx! {
        Surface {
            SurfaceHeader {
                title: "用户列表".to_string(),
                subtitle: "管理系统用户，分配角色。".to_string()
            }
            div { style: "margin-bottom: 12px;",
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "新建用户" },
                    onclick: move |_| {
                        editing.set(Some(UserUpsertDto { username: String::new(), password: String::new(), nickname: String::new(), status: "enabled".to_string() }));
                        editing_id.set(None);
                        form_username.set(String::new());
                        form_nickname.set(String::new());
                        form_password.set(String::new());
                        form_status.set("enabled".to_string());
                        error_msg.set(None);
                    }
                }
            }
            // 编辑表单
            if editing.read().is_some() {
                Surface {
                    SurfaceHeader {
                        title: if editing_id.read().is_some() { "编辑用户".to_string() } else { "新建用户".to_string() },
                        subtitle: "填写用户信息。".to_string()
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;padding:8px 0;",
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "用户名" }
                            input {
                                r#type: "text",
                                value: "{form_username}",
                                oninput: move |e| form_username.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "昵称" }
                            input {
                                r#type: "text",
                                value: "{form_nickname}",
                                oninput: move |e| form_nickname.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "密码" }
                            input {
                                r#type: "password",
                                value: "{form_password}",
                                oninput: move |e| form_password.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "状态" }
                            select {
                                value: "{form_status}",
                                onchange: move |e| form_status.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;",
                                option { value: "enabled", "启用" }
                                option { value: "disabled", "停用" }
                                option { value: "locked", "锁定" }
                            }
                        }
                        if let Some(err) = error_msg.read().as_ref() {
                            div { style: "color:red;font-size:13px;", "{err}" }
                        }
                        div { style: "display:flex;gap:8px;padding-top:4px;",
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "保存" },
                                onclick: {
                                    let sys = use_context::<AppServices>().system.clone();
                                    move |_| {
                                        let sys = sys.clone();
                                        let u = form_username.read().trim().to_string();
                                        let p = form_password.read().trim().to_string();
                                        let n = form_nickname.read().trim().to_string();
                                        let s = form_status.read().clone();
                                        let eid = *editing_id.read();
                                        if u.is_empty() {
                                            error_msg.set(Some("用户名不能为空".into()));
                                            return;
                                        }
                                        if eid.is_none() && p.is_empty() {
                                            error_msg.set(Some("密码不能为空".into()));
                                            return;
                                        }
                                        error_msg.set(None);
                                        spawn(async move {
                                            let input = UserUpsertDto { username: u, password: p, nickname: n, status: s };
                                            let res = match eid {
                                                Some(id) => sys.update_user(id, input).await,
                                                None => sys.create_user(input).await,
                                            };
                                            match res {
                                                Ok(_) => {
                                                    editing.set(None);
                                                    editing_id.set(None);
                                                    users_resource.restart();
                                                }
                                                Err(e) => error_msg.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                }
                            }
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                onclick: move |_| {
                                    editing.set(None);
                                    editing_id.set(None);
                                    error_msg.set(None);
                                }
                            }
                        }
                    }
                }
            }
            // 角色分配对话框
            if let Some(uid) = *show_role_dialog.read() {
                Surface {
                    SurfaceHeader {
                        title: format!("分配角色 (用户 #{uid})"),
                        subtitle: "勾选要分配给该用户的角色。".to_string()
                    }
                    {
                        match roles_resource.read().as_ref() {
                            Some(Ok(roles)) => rsx! {
                                div { style: "display:flex;flex-direction:column;gap:4px;padding:8px 0;",
                                    for role in roles {
                                        div { style: "display:flex;align-items:center;gap:8px;",
                                            input {
                                                r#type: "checkbox",
                                                checked: selected_role_ids.read().contains(&role.id),
                                                onchange: {
                                                    let rid = role.id;
                                                    move |e: Event<FormData>| {
                                                        let mut ids = selected_role_ids.read().clone();
                                                        if e.value() == "true" {
                                                            if !ids.contains(&rid) { ids.push(rid); }
                                                        } else {
                                                            ids.retain(|x| *x != rid);
                                                        }
                                                        selected_role_ids.set(ids);
                                                    }
                                                }
                                            }
                                            span { "{role.name}" }
                                            span { style: "color:var(--text-secondary,#888);font-size:12px;", "{role.description}" }
                                        }
                                    }
                                }
                                div { style: "display:flex;gap:8px;padding-top:4px;",
                                    WorkbenchButton {
                                        class: "workbench-button".to_string(), children: rsx! { "保存" },
                                        onclick: {
                                            let sys = use_context::<AppServices>().system.clone();
                                            move |_| {
                                                let sys = sys.clone();
                                                let ids = selected_role_ids.read().clone();
                                                spawn(async move {
                                                    if sys.authorize_user_roles(uid, ids).await.is_ok() {
                                                        show_role_dialog.set(None);
                                                        users_resource.restart();
                                                    }
                                                });
                                            }
                                        }
                                    }
                                    WorkbenchButton {
                                        class: "workbench-button".to_string(), children: rsx! { "取消" },
                                        onclick: move |_| show_role_dialog.set(None)
                                    }
                                }
                            },
                            _ => rsx! { div { style: "padding:8px;", "加载中…" } }
                        }
                    }
                }
            }
            // 用户表格
            {
                match users_resource.read().as_ref() {
                    Some(Ok(users)) => {
                        let rows: Vec<UserWithRolesDto> = users.clone();
                        rsx! {
                            DataTable {
                                columns: vec![
                                    "用户".to_string(),
                                    "昵称".to_string(),
                                    "角色".to_string(),
                                    "状态".to_string(),
                                    "操作".to_string()
                                ],
                                for item in rows {
                                    tr {
                                        key: "{item.user.id}",
                                        td { "{item.user.username}" }
                                        td { "{item.user.nickname}" }
                                        td { "{item.role_names.join(\", \")}" }
                                        td {
                                            {
                                                match item.user.status.as_str() {
                                                    "enabled" => "✅ 启用",
                                                    "disabled" => "⏸ 停用",
                                                    "locked" => "🔒 锁定",
                                                    _ => &item.user.status,
                                                }
                                            }
                                        }
                                        td { style: "display:flex;gap:4px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "编辑" },
                                                onclick: {
                                                    let u = item.user.clone();
                                                    move |_| {
                                                        form_username.set(u.username.clone());
                                                        form_nickname.set(u.nickname.clone());
                                                        form_password.set(String::new());
                                                        form_status.set(u.status.clone());
                                                        editing_id.set(Some(u.id));
                                                        editing.set(Some(UserUpsertDto { username: u.username.clone(), password: String::new(), nickname: u.nickname.clone(), status: u.status.clone() }));
                                                        error_msg.set(None);
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "角色" },
                                                onclick: {
                                                    let uid = item.user.id;
                                                    let rids = item.role_ids.clone();
                                                    move |_| {
                                                        selected_role_ids.set(rids.clone());
                                                        show_role_dialog.set(Some(uid));
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "权限" },
                                                onclick: {
                                                    let sys = use_context::<AppServices>().system.clone();
                                                    let uid = item.user.id;
                                                    let uname = item.user.username.clone();
                                                    move |_| {
                                                        let sys = sys.clone();
                                                        let uname = uname.clone();
                                                        perms_loading.set(true);
                                                        show_perms_dialog.set(Some((uid, uname.clone(), Vec::new())));
                                                        spawn(async move {
                                                            match sys.get_user_effective_menu_ids(uid).await {
                                                                Ok(ids) => {
                                                                    show_perms_dialog.set(Some((uid, uname.clone(), ids)));
                                                                }
                                                                Err(_) => {
                                                                    show_perms_dialog.set(Some((uid, uname.clone(), Vec::new())));
                                                                }
                                                            }
                                                            perms_loading.set(false);
                                                        });
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "删除" },
                                                onclick: {
                                                    let uid = item.user.id;
                                                    move |_| {
                                                        confirm_delete_user.set(Some(uid));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { style: "color:red;padding:8px;", "加载失败: {e}" } },
                    None => rsx! { div { style: "padding:8px;", "加载中…" } },
                }
            }
            // 删除确认
            if let Some(uid) = *confirm_delete_user.read() {
                Surface {
                    SurfaceHeader {
                        title: "确认删除".to_string(),
                        subtitle: format!("确定要删除用户 #{uid} 吗？此操作不可撤销。")
                    }
                    div { style: "display:flex;gap:8px;padding:8px 0;",
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "确认删除" },
                            onclick: {
                                let sys = use_context::<AppServices>().system.clone();
                                move |_| {
                                    let sys = sys.clone();
                                    spawn(async move {
                                        match sys.delete_user(uid).await {
                                            Ok(_) => {
                                                confirm_delete_user.set(None);
                                                users_resource.restart();
                                            }
                                            Err(e) => { error_msg.set(Some(format!("删除失败: {e}"))); }
                                        }
                                    });
                                }
                            }
                        }
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "取消" },
                            onclick: move |_| confirm_delete_user.set(None)
                        }
                    }
                }
            }
            // 用户有效权限查看
            if let Some((uid, ref uname, ref menu_ids)) = show_perms_dialog.read().clone() {
                Surface {
                    SurfaceHeader {
                        title: format!("有效权限 — {uname}"),
                        subtitle: format!("用户 #{uid} 通过角色关联获得的菜单权限。")
                    }
                    if *perms_loading.read() {
                        div { style: "padding:8px;", "加载中…" }
                    } else if menu_ids.is_empty() {
                        div { style: "padding:8px;color:var(--text-secondary,#888);", "该用户没有分配任何角色或角色未授权菜单。" }
                    } else {
                        {
                            match menus_resource.read().as_ref() {
                                Some(Ok(all_menus)) => {
                                    let names: Vec<String> = menu_ids.iter().filter_map(|mid| {
                                        all_menus.iter().find(|m| m.id == *mid).map(|m| {
                                            let badge = match m.menu_type.as_str() {
                                                "dir" => "📁",
                                                "button" => "🔘",
                                                _ => "📄",
                                            };
                                            format!("{badge} {} ({})", m.name, if m.permission_code.is_empty() { "—" } else { &m.permission_code })
                                        })
                                    }).collect();
                                    rsx! {
                                        div { style: "padding:8px 0;font-size:13px;color:var(--text-secondary,#888);",
                                            "共 {names.len()} 项菜单权限"
                                        }
                                        div { style: "display:flex;flex-direction:column;gap:2px;padding:4px 0;max-height:320px;overflow-y:auto;",
                                            for name in names {
                                                div { style: "padding:2px 8px;font-size:13px;border-bottom:1px solid var(--border,#eee);", "{name}" }
                                            }
                                        }
                                        div { style: "padding-top:8px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "关闭" },
                                                onclick: move |_| show_perms_dialog.set(None)
                                            }
                                        }
                                    }
                                }
                                _ => rsx! { div { style: "padding:8px;", "加载菜单数据中…" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Roles (with menu authorization) ────────────────────────────────────────

#[component]
fn RolesScene() -> Element {
    let sys = use_context::<AppServices>().system.clone();
    let mut roles_resource = use_resource(move || {
        let sys = sys.clone();
        async move { sys.list_roles().await }
    });

    let sys2 = use_context::<AppServices>().system.clone();
    let mut menus_resource = use_resource(move || {
        let sys = sys2.clone();
        async move { sys.list_menus().await }
    });

    let mut editing = use_signal::<Option<RoleUpsertDto>>(|| None);
    let mut editing_id = use_signal::<Option<i32>>(|| None);
    let mut form_name = use_signal(String::new);
    let mut form_desc = use_signal(String::new);
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut show_auth_dialog = use_signal::<Option<(i32, String)>>(|| None);
    let mut selected_menu_ids = use_signal::<Vec<i32>>(Vec::new);
    let mut auth_loading = use_signal(|| false);
    let mut confirm_delete_role = use_signal::<Option<i32>>(|| None);

    rsx! {
        // 指标
        MetricStrip { columns: 3,
            StatTile { label: "角色总数".to_string(), value: match roles_resource.read().as_ref() { Some(Ok(roles)) => roles.len().to_string(), _ => "…".to_string() }, detail: "含系统预置角色".to_string() }
            StatTile { label: "自定义角色".to_string(), value: match roles_resource.read().as_ref() { Some(Ok(roles)) => roles.iter().filter(|r| !r.is_system).count().to_string(), _ => "…".to_string() }, detail: "按业务线拆分".to_string() }
            StatTile { label: "菜单总数".to_string(), value: match menus_resource.read().as_ref() { Some(Ok(menus)) => menus.len().to_string(), _ => "…".to_string() }, detail: "可分配给角色的菜单项".to_string() }
        }
        Surface {
            SurfaceHeader {
                title: "角色清单".to_string(),
                subtitle: "角色与关键权限摘要。点击「授权」分配菜单权限。".to_string()
            }
            div { style: "margin-bottom: 12px;",
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "新建角色" },
                    onclick: move |_| {
                        form_name.set(String::new());
                        form_desc.set(String::new());
                        editing_id.set(None);
                        editing.set(Some(RoleUpsertDto { name: String::new(), description: String::new() }));
                        error_msg.set(None);
                    }
                }
            }
            // 编辑表单
            if editing.read().is_some() {
                Surface {
                    SurfaceHeader {
                        title: if editing_id.read().is_some() { "编辑角色".to_string() } else { "新建角色".to_string() },
                        subtitle: "填写角色信息。".to_string()
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;padding:8px 0;",
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "角色名" }
                            input {
                                r#type: "text",
                                value: "{form_name}",
                                oninput: move |e| form_name.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "描述" }
                            input {
                                r#type: "text",
                                value: "{form_desc}",
                                oninput: move |e| form_desc.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        if let Some(err) = error_msg.read().as_ref() {
                            div { style: "color:red;font-size:13px;", "{err}" }
                        }
                        div { style: "display:flex;gap:8px;padding-top:4px;",
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "保存" },
                                onclick: {
                                    let sys = use_context::<AppServices>().system.clone();
                                    move |_| {
                                        let sys = sys.clone();
                                        let n = form_name.read().trim().to_string();
                                        let d = form_desc.read().trim().to_string();
                                        let eid = *editing_id.read();
                                        if n.is_empty() {
                                            error_msg.set(Some("角色名不能为空".into()));
                                            return;
                                        }
                                        error_msg.set(None);
                                        spawn(async move {
                                            let input = RoleUpsertDto { name: n, description: d };
                                            let res = match eid {
                                                Some(id) => sys.update_role(id, input).await,
                                                None => sys.create_role(input).await,
                                            };
                                            match res {
                                                Ok(_) => {
                                                    editing.set(None);
                                                    editing_id.set(None);
                                                    roles_resource.restart();
                                                }
                                                Err(e) => error_msg.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                }
                            }
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                onclick: move |_| {
                                    editing.set(None);
                                    editing_id.set(None);
                                    error_msg.set(None);
                                }
                            }
                        }
                    }
                }
            }
            // 角色-菜单授权对话框
            if let Some((rid, rname)) = show_auth_dialog.read().clone() {
                Surface {
                    SurfaceHeader {
                        title: format!("菜单授权 — {rname}"),
                        subtitle: format!("角色 #{rid} · 勾选该角色可访问的菜单。")
                    }
                    if *auth_loading.read() {
                        div { style: "padding:8px;", "加载中…" }
                    } else {
                        // 菜单树 checkbox
                        {
                            match menus_resource.read().as_ref() {
                                Some(Ok(menus)) => {
                                    let roots: Vec<MenuDto> = menus.iter().filter(|m| m.parent_id.is_none()).cloned().collect();
                                    let all_ids: Vec<i32> = menus.iter().map(|m| m.id).collect();
                                    rsx! {
                                        div { style: "display:flex;gap:8px;padding:8px 0;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(),
                                                children: rsx! { "全选" },
                                                onclick: {
                                                    let ids = all_ids.clone();
                                                    move |_| selected_menu_ids.set(ids.clone())
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(),
                                                children: rsx! { "全不选" },
                                                onclick: move |_| selected_menu_ids.set(Vec::new())
                                            }
                                        }
                                        div { style: "padding:4px 0;",
                                            for root in &roots {
                                                MenuCheckbox {
                                                    menu: root.clone(),
                                                    all_menus: menus.clone(),
                                                    selected: selected_menu_ids.read().clone(),
                                                    on_toggle: {
                                                        let all_m = menus.clone();
                                                        move |(clicked_id, now_checked): (i32, bool)| {
                                                            let mut ids = selected_menu_ids.read().clone();
                                                            let descendants = collect_descendant_ids(clicked_id, &all_m);
                                                            if now_checked {
                                                                for did in &descendants {
                                                                    if !ids.contains(did) { ids.push(*did); }
                                                                }
                                                            } else {
                                                                ids.retain(|x| !descendants.contains(x));
                                                            }
                                                            selected_menu_ids.set(ids);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { style: "display:flex;gap:8px;padding-top:8px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "保存授权" },
                                                onclick: {
                                                    let sys = use_context::<AppServices>().system.clone();
                                                    move |_| {
                                                        let sys = sys.clone();
                                                        let ids = selected_menu_ids.read().clone();
                                                        spawn(async move {
                                                            if sys.authorize_role_menus(rid, ids).await.is_ok() {
                                                                show_auth_dialog.set(None);
                                                                roles_resource.restart();
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                                onclick: move |_| show_auth_dialog.set(None)
                                            }
                                        }
                                    }
                                }
                                _ => rsx! { div { style: "padding:8px;", "加载菜单中…" } }
                            }
                        }
                    }
                }
            }
            // 角色表格
            {
                match roles_resource.read().as_ref() {
                    Some(Ok(roles)) => {
                        let rows: Vec<RoleDto> = roles.clone();
                        rsx! {
                            DataTable {
                                columns: vec!["角色".to_string(), "描述".to_string(), "菜单数".to_string(), "类型".to_string(), "操作".to_string()],
                                for role in rows {
                                    tr {
                                        key: "{role.id}",
                                        td { "{role.name}" }
                                        td { "{role.description}" }
                                        td { "{role.menu_count}" }
                                        td { if role.is_system { "系统" } else { "自定义" } }
                                        td { style: "display:flex;gap:4px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "授权" },
                                                onclick: {
                                                    let sys = use_context::<AppServices>().system.clone();
                                                    let rid = role.id;
                                                    let rname = role.name.clone();
                                                    let is_sys = role.is_system;
                                                    move |_| {
                                                        if is_sys { return; }
                                                        let sys = sys.clone();
                                                        auth_loading.set(true);
                                                        show_auth_dialog.set(Some((rid, rname.clone())));
                                                        spawn(async move {
                                                            match sys.get_role(rid).await {
                                                                Ok(detail) => { selected_menu_ids.set(detail.menu_ids); }
                                                                Err(_) => { selected_menu_ids.set(Vec::new()); }
                                                            }
                                                            auth_loading.set(false);
                                                        });
                                                    }
                                                }
                                            }
                                            if !role.is_system {
                                                WorkbenchButton {
                                                    class: "workbench-button".to_string(), children: rsx! { "编辑" },
                                                    onclick: {
                                                        let r = role.clone();
                                                        move |_| {
                                                            form_name.set(r.name.clone());
                                                            form_desc.set(r.description.clone());
                                                            editing_id.set(Some(r.id));
                                                            editing.set(Some(RoleUpsertDto { name: r.name.clone(), description: r.description.clone() }));
                                                            error_msg.set(None);
                                                        }
                                                    }
                                                }
                                                WorkbenchButton {
                                                    class: "workbench-button".to_string(), children: rsx! { "删除" },
                                                    onclick: {
                                                        let rid = role.id;
                                                        move |_| {
                                                            confirm_delete_role.set(Some(rid));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { style: "color:red;padding:8px;", "加载失败: {e}" } },
                    None => rsx! { div { style: "padding:8px;", "加载中…" } },
                }
            }
            // 角色删除确认
            if let Some(rid) = *confirm_delete_role.read() {
                Surface {
                    SurfaceHeader {
                        title: "确认删除".to_string(),
                        subtitle: format!("确定要删除角色 #{rid} 吗？此操作会同时清除该角色的菜单授权关系，不可撤销。")
                    }
                    div { style: "display:flex;gap:8px;padding:8px 0;",
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "确认删除" },
                            onclick: {
                                let sys = use_context::<AppServices>().system.clone();
                                move |_| {
                                    let sys = sys.clone();
                                    spawn(async move {
                                        match sys.delete_role(rid).await {
                                            Ok(_) => {
                                                confirm_delete_role.set(None);
                                                roles_resource.restart();
                                            }
                                            Err(e) => { error_msg.set(Some(format!("删除失败: {e}"))); }
                                        }
                                    });
                                }
                            }
                        }
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "取消" },
                            onclick: move |_| confirm_delete_role.set(None)
                        }
                    }
                }
            }
        }
    }
}

/// 收集指定菜单及其所有后代 ID
fn collect_descendant_ids(menu_id: i32, all_menus: &[MenuDto]) -> Vec<i32> {
    let mut ids = vec![menu_id];
    for m in all_menus.iter().filter(|m| m.parent_id == Some(menu_id)) {
        ids.extend(collect_descendant_ids(m.id, all_menus));
    }
    ids
}

/// 判断节点是否"全选"（所有后代都在 selected 中）
fn is_all_selected(menu_id: i32, all_menus: &[MenuDto], selected: &[i32]) -> bool {
    let descendants = collect_descendant_ids(menu_id, all_menus);
    descendants.iter().all(|id| selected.contains(id))
}

/// 判断节点是否"部分选中"（有后代在 selected 中但不是全部）
fn is_indeterminate(menu_id: i32, all_menus: &[MenuDto], selected: &[i32]) -> bool {
    let descendants = collect_descendant_ids(menu_id, all_menus);
    let any_selected = descendants.iter().any(|id| selected.contains(id));
    any_selected && !is_all_selected(menu_id, all_menus, selected)
}

/// 递归菜单 checkbox 组件（父子级联）
#[component]
fn MenuCheckbox(
    menu: MenuDto,
    all_menus: Vec<MenuDto>,
    selected: Vec<i32>,
    on_toggle: EventHandler<(i32, bool)>,
) -> Element {
    let children: Vec<MenuDto> = all_menus
        .iter()
        .filter(|m| m.parent_id == Some(menu.id))
        .cloned()
        .collect();
    let has_children = !children.is_empty();
    let is_leaf_checked = selected.contains(&menu.id);
    let node_checked = if has_children {
        is_all_selected(menu.id, &all_menus, &selected)
    } else {
        is_leaf_checked
    };
    let _indeterminate = has_children && is_indeterminate(menu.id, &all_menus, &selected);
    let type_badge = match menu.menu_type.as_str() {
        "dir" => "📁",
        "button" => "🔘",
        _ => "📄",
    };

    rsx! {
        div { style: "padding-left: 16px;",
            div { style: "display:flex;align-items:center;gap:6px;padding:2px 0;",
                input {
                    r#type: "checkbox",
                    checked: "{node_checked}",
                    // Note: Dioxus doesn't support indeterminate directly, visual hint via style
                    onchange: move |e: Event<FormData>| {
                        let now_checked = e.value() == "true";
                        on_toggle.call((menu.id, now_checked));
                    }
                }
                span { "{type_badge}" }
                span { style: "font-size:14px;", "{menu.name}" }
                if !menu.permission_code.is_empty() {
                    span { style: "color:var(--text-tertiary,#aaa);font-size:11px;font-family:monospace;", "{menu.permission_code}" }
                }
            }
            for child in children {
                MenuCheckbox {
                    key: "{child.id}",
                    menu: child,
                    all_menus: all_menus.clone(),
                    selected: selected.clone(),
                    on_toggle: on_toggle
                }
            }
        }
    }
}

// ─── Menus ──────────────────────────────────────────────────────────────────

#[component]
fn MenusScene() -> Element {
    let sys = use_context::<AppServices>().system.clone();
    let mut menus_resource = use_resource(move || {
        let sys = sys.clone();
        async move { sys.list_menus().await }
    });

    let mut editing = use_signal::<Option<MenuUpsertDto>>(|| None);
    let mut editing_id = use_signal::<Option<i32>>(|| None);
    let mut form_name = use_signal(String::new);
    let mut form_route = use_signal(String::new);
    let mut form_icon = use_signal(String::new);
    let mut form_sort = use_signal(|| "0".to_string());
    let mut form_parent = use_signal(|| "0".to_string());
    let mut form_visible = use_signal(|| true);
    let mut form_permission = use_signal(String::new);
    let mut form_menu_type = use_signal(|| "menu".to_string());
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut confirm_delete_menu = use_signal::<Option<i32>>(|| None);

    let _reload = move || {
        menus_resource.restart();
    };

    rsx! {
        Surface {
            SurfaceHeader {
                title: "菜单树".to_string(),
                subtitle: "管理后台导航菜单，支持树形嵌套、权限标识与类型分级。".to_string()
            }
            div { style: "margin-bottom: 12px;",
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "新建菜单" },
                    onclick: move |_| {
                        form_name.set(String::new());
                        form_route.set(String::new());
                        form_icon.set(String::new());
                        form_sort.set("0".to_string());
                        form_parent.set("0".to_string());
                        form_visible.set(true);
                        form_permission.set(String::new());
                        form_menu_type.set("menu".to_string());
                        editing_id.set(None);
                        editing.set(Some(MenuUpsertDto::default()));
                        error_msg.set(None);
                    }
                }
            }
            // 编辑表单
            if editing.read().is_some() {
                Surface {
                    SurfaceHeader {
                        title: if editing_id.read().is_some() { "编辑菜单".to_string() } else { "新建菜单".to_string() },
                        subtitle: "填写菜单信息。".to_string()
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;padding:8px 0;",
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "菜单名" }
                            input {
                                r#type: "text",
                                value: "{form_name}",
                                oninput: move |e| form_name.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "路由" }
                            input {
                                r#type: "text",
                                value: "{form_route}",
                                oninput: move |e| form_route.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "图标" }
                            input {
                                r#type: "text",
                                value: "{form_icon}",
                                oninput: move |e| form_icon.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "排序" }
                            input {
                                r#type: "number",
                                value: "{form_sort}",
                                oninput: move |e| form_sort.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "父级" }
                            select {
                                value: "{form_parent}",
                                onchange: move |e| form_parent.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;",
                                option { value: "0", "（顶级）" }
                                {
                                    match menus_resource.read().as_ref() {
                                        Some(Ok(menus)) => {
                                            let eid = *editing_id.read();
                                            let options: Vec<MenuDto> = menus.iter().filter(|m| eid.map_or(true, |id| m.id != id)).cloned().collect();
                                            rsx! {
                                                for m in options {
                                                    option { value: "{m.id}", "{m.name}" }
                                                }
                                            }
                                        }
                                        _ => rsx! {}
                                    }
                                }
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "类型" }
                            select {
                                value: "{form_menu_type}",
                                onchange: move |e| form_menu_type.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;",
                                option { value: "dir", "目录" }
                                option { value: "menu", "菜单" }
                                option { value: "button", "按钮" }
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "权限标识" }
                            input {
                                r#type: "text",
                                value: "{form_permission}",
                                oninput: move |e| form_permission.set(e.value()),
                                placeholder: "例如 system:user:list".to_string(),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "可见" }
                            input {
                                r#type: "checkbox",
                                checked: *form_visible.read(),
                                onchange: move |e: Event<FormData>| form_visible.set(e.value() == "true")
                            }
                        }
                        if let Some(err) = error_msg.read().as_ref() {
                            div { style: "color:red;font-size:13px;", "{err}" }
                        }
                        div { style: "display:flex;gap:8px;padding-top:4px;",
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "保存" },
                                onclick: {
                                    let sys = use_context::<AppServices>().system.clone();
                                    move |_| {
                                        let sys = sys.clone();
                                        let n = form_name.read().trim().to_string();
                                        let r = form_route.read().trim().to_string();
                                        let ic = form_icon.read().trim().to_string();
                                        let sort: i32 = form_sort.read().parse().unwrap_or(0);
                                        let pid_raw: i32 = form_parent.read().parse().unwrap_or(0);
                                        let pid = if pid_raw == 0 { None } else { Some(pid_raw) };
                                        let vis = *form_visible.read();
                                        let pc = form_permission.read().trim().to_string();
                                        let mt = form_menu_type.read().clone();
                                        let eid = *editing_id.read();
                                        if n.is_empty() {
                                            error_msg.set(Some("菜单名不能为空".into()));
                                            return;
                                        }
                                        error_msg.set(None);
                                        spawn(async move {
                                            let input = MenuUpsertDto { name: n, route: r, icon: ic, sort_order: sort, parent_id: pid, visible: vis, permission_code: pc, menu_type: mt };
                                            let res = match eid {
                                                Some(id) => sys.update_menu(id, input).await,
                                                None => sys.create_menu(input).await,
                                            };
                                            match res {
                                                Ok(_) => {
                                                    editing.set(None);
                                                    editing_id.set(None);
                                                    menus_resource.restart();
                                                }
                                                Err(e) => error_msg.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                }
                            }
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                onclick: move |_| {
                                    editing.set(None);
                                    editing_id.set(None);
                                    error_msg.set(None);
                                }
                            }
                        }
                    }
                }
            }
            // 菜单表格
            {
                match menus_resource.read().as_ref() {
                    Some(Ok(menus)) => {
                        let rows: Vec<MenuDto> = menus.clone();
                        let parent_map: Vec<(i32, String)> = menus.iter().map(|m| (m.id, m.name.clone())).collect();
                        rsx! {
                            DataTable {
                                columns: vec!["菜单名".to_string(), "类型".to_string(), "权限标识".to_string(), "路由".to_string(), "父级".to_string(), "排序".to_string(), "可见".to_string(), "操作".to_string()],
                                for menu in rows {
                                    tr {
                                        key: "{menu.id}",
                                        td { style: if menu.parent_id.is_some() { "padding-left:24px;" } else { "" }, "{menu.name}" }
                                        td {
                                            {
                                                match menu.menu_type.as_str() {
                                                    "dir" => "📁 目录",
                                                    "button" => "🔘 按钮",
                                                    _ => "📄 菜单",
                                                }
                                            }
                                        }
                                        td { style: "color:var(--text-secondary,#888);font-size:12px;", { if menu.permission_code.is_empty() { "—".to_string() } else { menu.permission_code.clone() } } }
                                        td { style: "color:var(--text-secondary,#888);", { if menu.route.is_empty() { "—".to_string() } else { menu.route.clone() } } }
                                        td {
                                            {
                                                match menu.parent_id {
                                                    Some(pid) => parent_map.iter().find(|(id,_)| *id == pid).map(|(_,n)| n.as_str()).unwrap_or("?"),
                                                    None => "—",
                                                }
                                            }
                                        }
                                        td { "{menu.sort_order}" }
                                        td { if menu.visible { "✅" } else { "—" } }
                                        td { style: "display:flex;gap:4px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "编辑" },
                                                onclick: {
                                                    let m = menu.clone();
                                                    move |_| {
                                                        form_name.set(m.name.clone());
                                                        form_route.set(m.route.clone());
                                                        form_icon.set(m.icon.clone());
                                                        form_sort.set(m.sort_order.to_string());
                                                        form_parent.set(m.parent_id.map_or("0".to_string(), |v| v.to_string()));
                                                        form_visible.set(m.visible);
                                                        form_permission.set(m.permission_code.clone());
                                                        form_menu_type.set(m.menu_type.clone());
                                                        editing_id.set(Some(m.id));
                                                        editing.set(Some(MenuUpsertDto { name: m.name.clone(), route: m.route.clone(), icon: m.icon.clone(), sort_order: m.sort_order, parent_id: m.parent_id, visible: m.visible, permission_code: m.permission_code.clone(), menu_type: m.menu_type.clone() }));
                                                        error_msg.set(None);
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "删除" },
                                                onclick: {
                                                    let mid = menu.id;
                                                    move |_| {
                                                        confirm_delete_menu.set(Some(mid));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { style: "color:red;padding:8px;", "加载失败: {e}" } },
                    None => rsx! { div { style: "padding:8px;", "加载中…" } },
                }
            }
            // 菜单删除确认（级联）
            if let Some(mid) = *confirm_delete_menu.read() {
                Surface {
                    SurfaceHeader {
                        title: "确认删除".to_string(),
                        subtitle: format!("确定要删除菜单 #{mid} 吗？若该菜单包含子项，将一并删除，同时清除所有角色对该菜单的授权关系，此操作不可撤销。")
                    }
                    div { style: "display:flex;gap:8px;padding:8px 0;",
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "确认删除" },
                            onclick: {
                                let sys = use_context::<AppServices>().system.clone();
                                move |_| {
                                    let sys = sys.clone();
                                    spawn(async move {
                                        match sys.delete_menu(mid).await {
                                            Ok(_) => {
                                                confirm_delete_menu.set(None);
                                                menus_resource.restart();
                                            }
                                            Err(e) => { error_msg.set(Some(format!("删除失败: {e}"))); }
                                        }
                                    });
                                }
                            }
                        }
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "取消" },
                            onclick: move |_| confirm_delete_menu.set(None)
                        }
                    }
                }
            }
        }
    }
}

// ─── Departments ───────────────────────────────────────────────────────────

#[component]
fn DepartmentsScene() -> Element {
    let sys = use_context::<AppServices>().system.clone();
    let mut dept_resource = use_resource(move || {
        let sys = sys.clone();
        async move { sys.list_departments().await }
    });

    let mut editing = use_signal::<Option<DepartmentUpsertDto>>(|| None);
    let mut editing_id = use_signal::<Option<i32>>(|| None);
    let mut form_name = use_signal(String::new);
    let mut form_sort = use_signal(|| "0".to_string());
    let mut form_parent = use_signal(|| "0".to_string());
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut confirm_delete_dept = use_signal::<Option<i32>>(|| None);

    rsx! {
        Surface {
            SurfaceHeader {
                title: "部门结构".to_string(),
                subtitle: "组织关系作为用户与角色的基础维度。".to_string()
            }
            div { style: "margin-bottom: 12px;",
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "新建部门" },
                    onclick: move |_| {
                        form_name.set(String::new());
                        form_sort.set("0".to_string());
                        form_parent.set("0".to_string());
                        editing_id.set(None);
                        editing.set(Some(DepartmentUpsertDto { parent_id: None, name: String::new(), sort_order: 0 }));
                        error_msg.set(None);
                    }
                }
            }
            if editing.read().is_some() {
                Surface {
                    SurfaceHeader {
                        title: if editing_id.read().is_some() { "编辑部门".to_string() } else { "新建部门".to_string() },
                        subtitle: "填写部门信息。".to_string()
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;padding:8px 0;",
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "部门名" }
                            input {
                                r#type: "text",
                                value: "{form_name}",
                                oninput: move |e| form_name.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "排序" }
                            input {
                                r#type: "number",
                                value: "{form_sort}",
                                oninput: move |e| form_sort.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "上级" }
                            select {
                                value: "{form_parent}",
                                onchange: move |e| form_parent.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;",
                                option { value: "0", "（顶级）" }
                                {
                                    match dept_resource.read().as_ref() {
                                        Some(Ok(depts)) => {
                                            let eid = *editing_id.read();
                                            let opts: Vec<DepartmentDto> = depts.iter().filter(|d| eid.map_or(true, |id| d.id != id)).cloned().collect();
                                            rsx! {
                                                for d in opts {
                                                    option { value: "{d.id}", "{d.name}" }
                                                }
                                            }
                                        }
                                        _ => rsx! {}
                                    }
                                }
                            }
                        }
                        if let Some(err) = error_msg.read().as_ref() {
                            div { style: "color:red;font-size:13px;", "{err}" }
                        }
                        div { style: "display:flex;gap:8px;padding-top:4px;",
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "保存" },
                                onclick: {
                                    let sys = use_context::<AppServices>().system.clone();
                                    move |_| {
                                        let sys = sys.clone();
                                        let n = form_name.read().trim().to_string();
                                        let sort: i32 = form_sort.read().parse().unwrap_or(0);
                                        let pid_raw: i32 = form_parent.read().parse().unwrap_or(0);
                                        let pid = if pid_raw == 0 { None } else { Some(pid_raw) };
                                        let eid = *editing_id.read();
                                        if n.is_empty() {
                                            error_msg.set(Some("部门名不能为空".into()));
                                            return;
                                        }
                                        error_msg.set(None);
                                        spawn(async move {
                                            let input = DepartmentUpsertDto { name: n, sort_order: sort, parent_id: pid };
                                            let res = match eid {
                                                Some(id) => sys.update_department(id, input).await,
                                                None => sys.create_department(input).await,
                                            };
                                            match res {
                                                Ok(_) => {
                                                    editing.set(None);
                                                    editing_id.set(None);
                                                    dept_resource.restart();
                                                }
                                                Err(e) => error_msg.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                }
                            }
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                onclick: move |_| {
                                    editing.set(None);
                                    editing_id.set(None);
                                    error_msg.set(None);
                                }
                            }
                        }
                    }
                }
            }
            {
                match dept_resource.read().as_ref() {
                    Some(Ok(depts)) => {
                        let rows: Vec<DepartmentDto> = depts.clone();
                        let parent_map: Vec<(i32, String)> = depts.iter().map(|d| (d.id, d.name.clone())).collect();
                        rsx! {
                            DataTable {
                                columns: vec!["部门名".to_string(), "上级".to_string(), "排序".to_string(), "操作".to_string()],
                                for dept in rows {
                                    tr {
                                        key: "{dept.id}",
                                        td { style: if dept.parent_id.is_some() { "padding-left:24px;" } else { "" }, "{dept.name}" }
                                        td {
                                            {
                                                match dept.parent_id {
                                                    Some(pid) => parent_map.iter().find(|(id,_)| *id == pid).map(|(_,n)| n.as_str()).unwrap_or("?"),
                                                    None => "—",
                                                }
                                            }
                                        }
                                        td { "{dept.sort_order}" }
                                        td { style: "display:flex;gap:4px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "编辑" },
                                                onclick: {
                                                    let d = dept.clone();
                                                    move |_| {
                                                        form_name.set(d.name.clone());
                                                        form_sort.set(d.sort_order.to_string());
                                                        form_parent.set(d.parent_id.map_or("0".to_string(), |v| v.to_string()));
                                                        editing_id.set(Some(d.id));
                                                        editing.set(Some(DepartmentUpsertDto { name: d.name.clone(), sort_order: d.sort_order, parent_id: d.parent_id }));
                                                        error_msg.set(None);
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "删除" },
                                                onclick: {
                                                    let did = dept.id;
                                                    move |_| {
                                                        confirm_delete_dept.set(Some(did));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { style: "color:red;padding:8px;", "加载失败: {e}" } },
                    None => rsx! { div { style: "padding:8px;", "加载中…" } },
                }
            }
            // 部门删除确认
            if let Some(did) = *confirm_delete_dept.read() {
                Surface {
                    SurfaceHeader {
                        title: "确认删除".to_string(),
                        subtitle: format!("确定要删除部门 #{did} 吗？子部门将一并删除，此操作不可撤销。")
                    }
                    div { style: "display:flex;gap:8px;padding:8px 0;",
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "确认删除" },
                            onclick: {
                                let sys = use_context::<AppServices>().system.clone();
                                move |_| {
                                    let sys = sys.clone();
                                    spawn(async move {
                                        match sys.delete_department(did).await {
                                            Ok(_) => {
                                                confirm_delete_dept.set(None);
                                                dept_resource.restart();
                                            }
                                            Err(e) => { error_msg.set(Some(format!("删除失败: {e}"))); }
                                        }
                                    });
                                }
                            }
                        }
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "取消" },
                            onclick: move |_| confirm_delete_dept.set(None)
                        }
                    }
                }
            }
        }
    }
}

// ─── Dictionaries ──────────────────────────────────────────────────────────

#[component]
fn DictionariesScene() -> Element {
    let sys = use_context::<AppServices>().system.clone();
    let mut groups_resource = use_resource(move || {
        let sys = sys.clone();
        async move { sys.list_dict_groups().await }
    });

    let mut selected_group = use_signal::<Option<DictGroupDto>>(|| None);

    // Group form
    let mut editing_group = use_signal::<Option<DictGroupUpsertDto>>(|| None);
    let mut editing_group_id = use_signal::<Option<i32>>(|| None);
    let mut form_gname = use_signal(String::new);
    let mut form_gdesc = use_signal(String::new);
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut confirm_delete_group = use_signal::<Option<i32>>(|| None);

    rsx! {
        Surface {
            SurfaceHeader {
                title: "字典分组".to_string(),
                subtitle: "字典分组与字典项独立维护，支撑表单枚举、状态机和值域配置。".to_string()
            }
            div { style: "margin-bottom: 12px;",
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "新建字典组" },
                    onclick: move |_| {
                        form_gname.set(String::new());
                        form_gdesc.set(String::new());
                        editing_group_id.set(None);
                        editing_group.set(Some(DictGroupUpsertDto { name: String::new(), description: String::new() }));
                        error_msg.set(None);
                    }
                }
            }
            if editing_group.read().is_some() {
                Surface {
                    SurfaceHeader {
                        title: if editing_group_id.read().is_some() { "编辑字典组".to_string() } else { "新建字典组".to_string() },
                        subtitle: "填写字典分组信息。".to_string()
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;padding:8px 0;",
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "名称" }
                            input {
                                r#type: "text",
                                value: "{form_gname}",
                                oninput: move |e| form_gname.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "描述" }
                            input {
                                r#type: "text",
                                value: "{form_gdesc}",
                                oninput: move |e| form_gdesc.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        if let Some(err) = error_msg.read().as_ref() {
                            div { style: "color:red;font-size:13px;", "{err}" }
                        }
                        div { style: "display:flex;gap:8px;padding-top:4px;",
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "保存" },
                                onclick: {
                                    let sys = use_context::<AppServices>().system.clone();
                                    move |_| {
                                        let sys = sys.clone();
                                        let n = form_gname.read().trim().to_string();
                                        let d = form_gdesc.read().trim().to_string();
                                        let eid = *editing_group_id.read();
                                        if n.is_empty() {
                                            error_msg.set(Some("名称不能为空".into()));
                                            return;
                                        }
                                        error_msg.set(None);
                                        spawn(async move {
                                            let input = DictGroupUpsertDto { name: n, description: d };
                                            let res = match eid {
                                                Some(id) => sys.update_dict_group(id, input).await,
                                                None => sys.create_dict_group(input).await,
                                            };
                                            match res {
                                                Ok(_) => {
                                                    editing_group.set(None);
                                                    editing_group_id.set(None);
                                                    groups_resource.restart();
                                                }
                                                Err(e) => error_msg.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                }
                            }
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                onclick: move |_| {
                                    editing_group.set(None);
                                    editing_group_id.set(None);
                                    error_msg.set(None);
                                }
                            }
                        }
                    }
                }
            }
            {
                match groups_resource.read().as_ref() {
                    Some(Ok(groups)) => {
                        let rows: Vec<DictGroupDto> = groups.clone();
                        rsx! {
                            DataTable {
                                columns: vec!["名称".to_string(), "描述".to_string(), "字典项数".to_string(), "操作".to_string()],
                                for g in rows {
                                    tr {
                                        key: "{g.id}",
                                        td { "{g.name}" }
                                        td { "{g.description}" }
                                        td { "{g.item_count}" }
                                        td { style: "display:flex;gap:4px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "字典项" },
                                                onclick: {
                                                    let g2 = g.clone();
                                                    move |_| selected_group.set(Some(g2.clone()))
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "编辑" },
                                                onclick: {
                                                    let g2 = g.clone();
                                                    move |_| {
                                                        form_gname.set(g2.name.clone());
                                                        form_gdesc.set(g2.description.clone());
                                                        editing_group_id.set(Some(g2.id));
                                                        editing_group.set(Some(DictGroupUpsertDto { name: g2.name.clone(), description: g2.description.clone() }));
                                                        error_msg.set(None);
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "删除" },
                                                onclick: {
                                                    let gid = g.id;
                                                    move |_| {
                                                        confirm_delete_group.set(Some(gid));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { style: "color:red;padding:8px;", "加载失败: {e}" } },
                    None => rsx! { div { style: "padding:8px;", "加载中…" } },
                }
            }
            // 字典组删除确认
            if let Some(gid) = *confirm_delete_group.read() {
                Surface {
                    SurfaceHeader {
                        title: "确认删除".to_string(),
                        subtitle: format!("确定要删除字典组 #{gid} 吗？该组下所有字典项将一并删除，此操作不可撤销。")
                    }
                    div { style: "display:flex;gap:8px;padding:8px 0;",
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "确认删除" },
                            onclick: {
                                let sys = use_context::<AppServices>().system.clone();
                                move |_| {
                                    let sys = sys.clone();
                                    spawn(async move {
                                        match sys.delete_dict_group(gid).await {
                                            Ok(_) => {
                                                confirm_delete_group.set(None);
                                                groups_resource.restart();
                                                selected_group.set(None);
                                            }
                                            Err(e) => { error_msg.set(Some(format!("删除失败: {e}"))); }
                                        }
                                    });
                                }
                            }
                        }
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "取消" },
                            onclick: move |_| confirm_delete_group.set(None)
                        }
                    }
                }
            }
        }
        // Selected group items
        if let Some(group) = selected_group.read().clone() {
            DictItemsPanel {
                group: group,
                on_close: move |_| selected_group.set(None),
            }
        }
    }
}

#[component]
fn DictItemsPanel(group: DictGroupDto, on_close: EventHandler<()>) -> Element {
    let sys = use_context::<AppServices>().system.clone();
    let gid = group.id;
    let mut items_resource = use_resource(move || {
        let sys = sys.clone();
        async move { sys.list_dict_items(gid).await }
    });

    let mut editing = use_signal::<Option<DictItemUpsertDto>>(|| None);
    let mut editing_id = use_signal::<Option<i32>>(|| None);
    let mut form_label = use_signal(String::new);
    let mut form_value = use_signal(String::new);
    let mut form_sort = use_signal(|| "0".to_string());
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut confirm_delete_item = use_signal::<Option<i32>>(|| None);

    rsx! {
        Surface {
            SurfaceHeader {
                title: format!("字典项 — {}", group.name),
                subtitle: format!("管理 {} 分组下的所有字典项。", group.name)
            }
            div { style: "display:flex;gap:8px;margin-bottom:12px;",
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "新建字典项" },
                    onclick: move |_| {
                        form_label.set(String::new());
                        form_value.set(String::new());
                        form_sort.set("0".to_string());
                        editing_id.set(None);
                        editing.set(Some(DictItemUpsertDto { group_id: group.id, label: String::new(), value: String::new(), sort_order: 0 }));
                        error_msg.set(None);
                    }
                }
                WorkbenchButton {
                    class: "workbench-button".to_string(), children: rsx! { "关闭" },
                    onclick: move |_| on_close.call(())
                }
            }
            if editing.read().is_some() {
                Surface {
                    SurfaceHeader {
                        title: if editing_id.read().is_some() { "编辑字典项".to_string() } else { "新建字典项".to_string() },
                        subtitle: "填写字典项信息。".to_string()
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;padding:8px 0;",
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "标签" }
                            input {
                                r#type: "text",
                                value: "{form_label}",
                                oninput: move |e| form_label.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "值" }
                            input {
                                r#type: "text",
                                value: "{form_value}",
                                oninput: move |e| form_value.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        div { style: "display:flex;align-items:center;gap:8px;",
                            label { style: "width:80px;text-align:right;", "排序" }
                            input {
                                r#type: "number",
                                value: "{form_sort}",
                                oninput: move |e| form_sort.set(e.value()),
                                style: "flex:1;padding:4px 8px;border:1px solid var(--border,#ccc);border-radius:4px;"
                            }
                        }
                        if let Some(err) = error_msg.read().as_ref() {
                            div { style: "color:red;font-size:13px;", "{err}" }
                        }
                        div { style: "display:flex;gap:8px;padding-top:4px;",
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "保存" },
                                onclick: {
                                    let sys = use_context::<AppServices>().system.clone();
                                    move |_| {
                                        let sys = sys.clone();
                                        let l = form_label.read().trim().to_string();
                                        let v = form_value.read().trim().to_string();
                                        let s: i32 = form_sort.read().parse().unwrap_or(0);
                                        let eid = *editing_id.read();
                                        if l.is_empty() {
                                            error_msg.set(Some("标签不能为空".into()));
                                            return;
                                        }
                                        error_msg.set(None);
                                        spawn(async move {
                                            let input = DictItemUpsertDto { group_id: gid, label: l, value: v, sort_order: s };
                                            let res = match eid {
                                                Some(id) => sys.update_dict_item(id, input).await,
                                                None => sys.create_dict_item(input).await,
                                            };
                                            match res {
                                                Ok(_) => {
                                                    editing.set(None);
                                                    editing_id.set(None);
                                                    items_resource.restart();
                                                }
                                                Err(e) => error_msg.set(Some(e.to_string())),
                                            }
                                        });
                                    }
                                }
                            }
                            WorkbenchButton {
                                class: "workbench-button".to_string(), children: rsx! { "取消" },
                                onclick: move |_| {
                                    editing.set(None);
                                    editing_id.set(None);
                                    error_msg.set(None);
                                }
                            }
                        }
                    }
                }
            }
            {
                match items_resource.read().as_ref() {
                    Some(Ok(items)) => {
                        let rows: Vec<DictItemDto> = items.clone();
                        rsx! {
                            DataTable {
                                columns: vec!["标签".to_string(), "值".to_string(), "排序".to_string(), "操作".to_string()],
                                for item in rows {
                                    tr {
                                        key: "{item.id}",
                                        td { "{item.label}" }
                                        td { style: "color:var(--text-secondary,#888);", "{item.value}" }
                                        td { "{item.sort_order}" }
                                        td { style: "display:flex;gap:4px;",
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "编辑" },
                                                onclick: {
                                                    let it = item.clone();
                                                    move |_| {
                                                        form_label.set(it.label.clone());
                                                        form_value.set(it.value.clone());
                                                        form_sort.set(it.sort_order.to_string());
                                                        editing_id.set(Some(it.id));
                                                        editing.set(Some(DictItemUpsertDto { group_id: gid, label: it.label.clone(), value: it.value.clone(), sort_order: it.sort_order }));
                                                        error_msg.set(None);
                                                    }
                                                }
                                            }
                                            WorkbenchButton {
                                                class: "workbench-button".to_string(), children: rsx! { "删除" },
                                                onclick: {
                                                    let iid = item.id;
                                                    move |_| {
                                                        confirm_delete_item.set(Some(iid));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { style: "color:red;padding:8px;", "加载失败: {e}" } },
                    None => rsx! { div { style: "padding:8px;", "加载中…" } },
                }
            }
            // 字典项删除确认
            if let Some(iid) = *confirm_delete_item.read() {
                Surface {
                    SurfaceHeader {
                        title: "确认删除".to_string(),
                        subtitle: format!("确定要删除字典项 #{iid} 吗？此操作不可撤销。")
                    }
                    div { style: "display:flex;gap:8px;padding:8px 0;",
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "确认删除" },
                            onclick: {
                                let sys = use_context::<AppServices>().system.clone();
                                move |_| {
                                    let sys = sys.clone();
                                    spawn(async move {
                                        match sys.delete_dict_item(iid).await {
                                            Ok(_) => {
                                                confirm_delete_item.set(None);
                                                items_resource.restart();
                                            }
                                            Err(e) => { error_msg.set(Some(format!("删除失败: {e}"))); }
                                        }
                                    });
                                }
                            }
                        }
                        WorkbenchButton {
                            class: "workbench-button".to_string(), children: rsx! { "取消" },
                            onclick: move |_| confirm_delete_item.set(None)
                        }
                    }
                }
            }
        }
    }
}
