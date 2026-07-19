//! 物联网插件的构建期字典类型。

include!(concat!(env!("OUT_DIR"), "/az_micro_dict/enums.rs"));

impl serde::Serialize for IotOnlineStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.raw_value())
    }
}

impl<'de> serde::Deserialize<'de> for IotOnlineStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        let raw_value = Self::items()
            .iter()
            .find(|item| item.raw_value == value)
            .map(|item| item.raw_value);

        raw_value
            .and_then(Self::from_raw)
            .ok_or_else(|| serde::de::Error::custom(format!("未知的设备在线状态：{value}")))
    }
}

#[cfg(test)]
mod tests {
    use super::IotOnlineStatus;

    #[test]
    fn generated_status_exposes_dictionary_metadata() {
        // 页面文案必须来自生成字典，避免枚举与展示名称分别维护。
        assert_eq!(IotOnlineStatus::HeartbeatLost.label(), "心跳丢失");
        assert_eq!(IotOnlineStatus::HeartbeatLost.code(), "heartbeat_lost");
        assert_eq!(
            IotOnlineStatus::DataAnomaly.description(),
            "连接和心跳正常，但业务数据已经超过允许窗口"
        );
        assert_eq!(IotOnlineStatus::items().len(), 5);
    }

    #[test]
    fn status_serde_uses_dictionary_raw_value() -> anyhow::Result<()> {
        // API 值必须继续使用稳定的 snake_case 字典原始值。
        let json = serde_json::to_string(&IotOnlineStatus::HeartbeatLost)?;
        assert_eq!(json, "\"heartbeat_lost\"");
        assert_eq!(
            serde_json::from_str::<IotOnlineStatus>(&json)?,
            IotOnlineStatus::HeartbeatLost
        );
        Ok(())
    }
}
