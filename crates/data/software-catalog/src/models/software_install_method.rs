use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, toasty::Model)]
#[table = "admin_software_install_methods"]
pub struct SoftwareInstallMethodRecord {
    #[key]
    pub id: Uuid,
    #[index]
    pub software_id: Uuid,
    pub platform: String,
    pub installer_kind: String,
    pub label: String,
    pub package_id: String,
    pub asset_item_id: Option<String>,
    pub command_text: String,
    pub note: String,
    pub priority: i32,
}
