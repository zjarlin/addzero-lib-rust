use az_aio_plugin_api::NativePluginRegistration;

#[test]
fn bundled_plugins_are_discovered_by_inventory() {
    az_aio_plugin_bundled::api::ensure_linked();
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
