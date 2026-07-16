use az_aio_platform::core::db::ToastyModelContribution;
use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_config_center_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_config_center_config_entries"]
pub struct ConfigEntry {
    #[key]
    pub id: String,
    #[index]
    pub namespace: String,
    #[index]
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[rudi::Singleton(name = "config-center-toasty-models")]
pub fn config_center_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(toasty::models!(ConfigEntry))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigEntrySummary {
    pub id: String,
    pub namespace: String,
    pub key: String,
    pub value: String,
}

impl From<ConfigEntry> for ConfigEntrySummary {
    fn from(entry: ConfigEntry) -> Self {
        Self {
            id: entry.id,
            namespace: entry.namespace,
            key: entry.key,
            value: entry.value,
        }
    }
}
