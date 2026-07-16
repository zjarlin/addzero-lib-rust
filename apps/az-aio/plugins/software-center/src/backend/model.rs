use az_aio_platform::core::db::ToastyModelContribution;
use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_software_center_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_software_center_software_package_records"]
pub struct SoftwarePackageRecord {
    #[key]
    pub id: String,
    #[index]
    pub name: String,
    pub source_path: String,
    pub platform: String,
    pub arch: String,
    pub status: String,
    pub updated_at: String,
}

#[rudi::Singleton(name = "software-center-toasty-models")]
pub fn software_center_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(toasty::models!(SoftwarePackageRecord))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SoftwarePackageSummary {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub platform: String,
    pub arch: String,
    pub status: String,
}

impl From<SoftwarePackageRecord> for SoftwarePackageSummary {
    fn from(record: SoftwarePackageRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            source_path: record.source_path,
            platform: record.platform,
            arch: record.arch,
            status: record.status,
        }
    }
}
