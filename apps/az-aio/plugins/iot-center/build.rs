use std::env;

use anyhow::{Context, Result};
use az_dict_spec::api::{DictionaryItemSpec, DictionarySpec, RawValueKind};
use az_micro_dict::api::{DictBuildGenerator, DictionaryContribution, StaticDictionaryContributor};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let output_dir = env::var_os("OUT_DIR").context("构建物联网字典时缺少 OUT_DIR")?;
    let contributor = StaticDictionaryContributor::new(vec![DictionaryContribution::new(
        "IotOnlineStatus",
        online_status_dictionary(),
    )]);

    DictBuildGenerator::new()
        .add_contributor(contributor)
        .generate_to(output_dir)
        .context("生成物联网状态字典失败")?;
    Ok(())
}

fn online_status_dictionary() -> DictionarySpec {
    DictionarySpec {
        code: "iot_online_status".to_string(),
        name: "设备在线状态".to_string(),
        description: Some("综合连接、心跳与业务数据活跃度计算的设备状态".to_string()),
        scope: "iot".to_string(),
        raw_value_kind: RawValueKind::String,
        open_enum: false,
        unknown_variant: None,
        sort_index: 10,
        items: vec![
            status_item("online", "在线", "连接、心跳与业务数据均处于正常窗口", 10),
            status_item(
                "heartbeat_lost",
                "心跳丢失",
                "连接仍存在，但心跳已经超过允许窗口",
                20,
            ),
            status_item(
                "data_anomaly",
                "数据异常",
                "连接和心跳正常，但业务数据已经超过允许窗口",
                30,
            ),
            status_item("offline", "离线", "设备连接已断开", 40),
            status_item("unknown", "未知", "设备状态所需信息不完整", 50),
        ],
    }
}

fn status_item(code: &str, label: &str, description: &str, sort_index: i64) -> DictionaryItemSpec {
    DictionaryItemSpec {
        code: code.to_string(),
        label: label.to_string(),
        description: Some(description.to_string()),
        raw_int_value: None,
        raw_text_value: Some(code.to_string()),
        sort_index,
        enabled: true,
        meta: None,
    }
}
