#![cfg(not(target_arch = "wasm32"))]

automod::dir!(pub(crate) "src/models");

pub(crate) fn software_catalog_models() -> toasty::ModelSet {
    toasty::models!(
        software_entry::SoftwareEntryRecord,
        software_install_method::SoftwareInstallMethodRecord
    )
}
