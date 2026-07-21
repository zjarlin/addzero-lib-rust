use jiff::Timestamp;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, toasty::Model)]
#[table = "ai_prompt_buttons"]
pub struct AiPromptButtonRecord {
    #[key]
    pub id: Uuid,
    pub label: String,
    #[index]
    pub target_kind: String,
    pub prompt_template: String,
    pub provider: String,
    pub model: String,
    pub enabled: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
