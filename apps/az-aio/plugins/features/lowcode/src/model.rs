use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct LowcodeApp {
    #[key]
    pub id: String,
    #[unique]
    pub slug: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct LowcodePage {
    #[key]
    pub id: String,
    #[index]
    pub app_id: String,
    pub route: String,
    pub title: String,
    pub schema_json: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeAppSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodePageSummary {
    pub id: String,
    pub app_id: String,
    pub route: String,
    pub title: String,
    pub enabled: bool,
}

impl From<LowcodeApp> for LowcodeAppSummary {
    fn from(app: LowcodeApp) -> Self {
        Self {
            id: app.id,
            slug: app.slug,
            name: app.name,
            description: app.description,
            enabled: app.enabled,
        }
    }
}

impl From<LowcodePage> for LowcodePageSummary {
    fn from(page: LowcodePage) -> Self {
        Self {
            id: page.id,
            app_id: page.app_id,
            route: page.route,
            title: page.title,
            enabled: page.enabled,
        }
    }
}
