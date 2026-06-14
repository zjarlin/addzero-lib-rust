/// Force-link plugin crates so inventory sections are merged.
///
/// Each plugin crate registers via `register_native_plugin!` which calls
/// `inventory::submit!`. Without an explicit symbol reference from the
/// host binary, the linker may discard plugin rlibs.
pub fn link_plugins() {
    // asset-hub
    {
        extern crate asset_hub;
        let _ = &asset_hub::plugin::AssetHubPlugin;
    }
    // config-center
    {
        extern crate config_center;
        let _ = &config_center::plugin::ConfigCenterPlugin;
    }
    // drive-center
    {
        extern crate drive_center;
        let _ = &drive_center::plugin::DriveCenterPlugin;
    }
    // edge-gateway
    {
        extern crate edge_gateway;
        let _ = &edge_gateway::plugin::EdgeGatewayPlugin;
    }
    // software-center
    {
        extern crate software_center;
        let _ = &software_center::plugin::SoftwareCenterPlugin;
    }
}
