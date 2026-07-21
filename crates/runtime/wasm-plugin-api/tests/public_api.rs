use az_wasm_plugin_api::contract::{ExtensionPoint, PluginState};

#[test]
fn plugin_state_codes_follow_manifest_values() {
    assert_eq!(PluginState::Installed.code(), "installed");
    assert_eq!(PluginState::from_code("active"), Some(PluginState::Active));
}

#[test]
fn extension_points_keep_manifest_wire_values() {
    assert_eq!(
        serde_json::to_string(&ExtensionPoint::ScriptEngine).expect("serialize"),
        r#""script-engine""#
    );
    assert_eq!(
        serde_json::from_str::<ExtensionPoint>(r#""ui-contribution""#).expect("deserialize"),
        ExtensionPoint::UiContribution
    );
}
