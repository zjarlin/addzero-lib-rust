pub fn link_all() {
    az_starter_identity::ensure_linked();
    az_starter_organization::ensure_linked();
    az_starter_dictionary::ensure_linked();
    az_starter_menu::ensure_linked();
    az_starter_audit::ensure_linked();
    az_starter_storage::ensure_linked();
}
