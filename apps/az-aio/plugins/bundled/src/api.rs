#![forbid(unsafe_code)]

pub fn ensure_linked() {
    asset_hub::plugin::ensure_linked();
    config_center::plugin::ensure_linked();
    drive_center::plugin::ensure_linked();
    edge_gateway::plugin::ensure_linked();
    software_center::plugin::ensure_linked();
}
