#![forbid(unsafe_code)]

pub fn ensure_linked() {
    asset_hub::plugin::ensure_linked();
    config_center::plugin::ensure_linked();
    drive_center::plugin::ensure_linked();
    edge_gateway::plugin::ensure_linked();
    software_center::plugin::ensure_linked();
}

#[cfg(test)]
mod tests {
    use az_aio_plugin_api::NativePluginRegistration;

    #[test]
    fn bundled_plugins_are_discovered_by_inventory() {
        super::ensure_linked();
        let discovered = inventory::iter::<NativePluginRegistration>
            .into_iter()
            .map(|registration| (registration.constructor)().descriptor().id)
            .collect::<std::collections::BTreeSet<_>>();

        for expected in [
            "asset-hub",
            "config-center",
            "drive-center",
            "edge-gateway",
            "software-center",
        ] {
            assert!(discovered.contains(expected), "missing {expected}");
        }
    }
}
