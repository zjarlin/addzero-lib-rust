#![forbid(unsafe_code)]

pub fn ensure_linked() {
    asset_hub::plugin::ensure_linked();
    az_aio_plugin_admin_bridge::api::ensure_linked();
    az_aio_plugin_catalog::api::ensure_linked();
    az_aio_plugin_core_nav::api::ensure_linked();
    az_aio_plugin_git_clis::plugin::ensure_linked();
    az_aio_plugin_git_envs::api::ensure_linked();
    az_aio_plugin_git_notes::api::ensure_linked();
    az_aio_plugin_git_skills::api::ensure_linked();
    az_aio_plugin_lowcode::descriptor::ensure_linked();
    az_aio_plugin_projects::api::ensure_linked();
    az_aio_plugin_search::api::ensure_linked();
    az_aio_plugin_settings::api::ensure_linked();
    az_aio_plugin_sync::descriptor::ensure_linked();
    config_center::plugin::ensure_linked();
    drive_center::plugin::ensure_linked();
    edge_gateway::plugin::ensure_linked();
    software_center::plugin::ensure_linked();
    az_system_starters::api::link_all();
}
