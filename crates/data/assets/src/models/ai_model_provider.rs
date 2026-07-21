use jiff::Timestamp;

#[derive(Clone, Debug, Eq, PartialEq, toasty::Model)]
#[table = "ai_model_providers"]
pub struct AiModelProviderRecord {
    #[key]
    pub provider: String,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
    pub key_id: String,
    pub encrypted_api_key: Option<String>,
    pub api_key_configured: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
