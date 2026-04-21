#![forbid(unsafe_code)]

mod config;
mod domains;
mod error;
mod http;
mod util;

#[cfg(test)]
mod tests;

pub use config::{ApiConfig, ApiConfigBuilder};
pub use domains::maven::{MavenArtifact, MavenCentralApi};
pub use domains::music::*;
pub use domains::temp_mail::{
    TempMailApi, TempMailDomain, TempMailMailbox, TempMailMessageDetail, TempMailMessageSummary,
    TempMailRecipient,
};
pub use domains::tianyancha::{
    TianyanchaApi, TianyanchaCompany, TianyanchaCompanyDetail, TianyanchaCompanySearchData,
    TianyanchaHuaweiApi, TianyanchaHuaweiCompany, TianyanchaHuaweiCompanySearchData,
    TianyanchaHuaweiPageInfo,
};
pub use error::{CreatesError, CreatesResult};

use reqwest::header::{ACCEPT, CONTENT_TYPE, HOST};

#[derive(Debug, Clone, Copy, Default)]
pub struct Creates;

impl Creates {
    pub fn maven_central() -> CreatesResult<MavenCentralApi> {
        create_maven_central_api()
    }

    pub fn maven_central_with_config(config: ApiConfig) -> CreatesResult<MavenCentralApi> {
        MavenCentralApi::new(config)
    }

    pub fn temp_mail() -> CreatesResult<TempMailApi> {
        create_temp_mail_api()
    }

    pub fn temp_mail_with_config(config: ApiConfig) -> CreatesResult<TempMailApi> {
        TempMailApi::new(config)
    }

    pub fn music_search() -> CreatesResult<MusicSearchApi> {
        create_music_search_api()
    }

    pub fn music_search_with_config(config: ApiConfig) -> CreatesResult<MusicSearchApi> {
        MusicSearchApi::new(config)
    }

    pub fn suno(api_token: impl Into<String>) -> CreatesResult<SunoApi> {
        create_suno_api(api_token)
    }

    pub fn suno_with_config(
        api_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<SunoApi> {
        SunoApi::new(api_token, config)
    }

    pub fn tianyancha(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> CreatesResult<TianyanchaApi> {
        create_tianyancha_api(authorization, auth_token)
    }

    pub fn tianyancha_with_config(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<TianyanchaApi> {
        TianyanchaApi::new(authorization, auth_token, config)
    }

    pub fn tianyancha_huawei(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> CreatesResult<TianyanchaHuaweiApi> {
        create_tianyancha_huawei_api(access_key, secret_key)
    }

    pub fn tianyancha_huawei_with_config(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<TianyanchaHuaweiApi> {
        TianyanchaHuaweiApi::new(access_key, secret_key, config)
    }
}

pub fn create_maven_central_api() -> CreatesResult<MavenCentralApi> {
    let config = ApiConfig::builder("https://search.maven.org").build()?;
    MavenCentralApi::new(config)
}

pub fn create_temp_mail_api() -> CreatesResult<TempMailApi> {
    let config = ApiConfig::builder("https://api.mail.tm")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    TempMailApi::new(config)
}

pub fn create_music_search_api() -> CreatesResult<MusicSearchApi> {
    let config = ApiConfig::builder("https://music.163.com/api/")
        .default_header(ACCEPT.as_str(), "application/json")
        .default_header("Referer", "https://music.163.com/")
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .build()?;
    MusicSearchApi::new(config)
}

pub fn create_suno_api(api_token: impl Into<String>) -> CreatesResult<SunoApi> {
    let config = ApiConfig::builder("https://api.vectorengine.ai")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    SunoApi::new(api_token, config)
}

pub fn create_tianyancha_api(
    authorization: impl Into<String>,
    auth_token: impl Into<String>,
) -> CreatesResult<TianyanchaApi> {
    let config = ApiConfig::builder("https://api9.tianyancha.com")
        .default_header(CONTENT_TYPE.as_str(), "application/json")
        .default_header(HOST.as_str(), "api9.tianyancha.com")
        .default_header(ACCEPT.as_str(), "*/*")
        .default_header("version", "TYC-XCX-WX")
        .default_header(
            "User-Agent",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 12_1_4 like Mac OS X) \
             AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/16D57 \
             MicroMessenger/7.0.5(0x17000523) NetType/WIFI Language/zh_CN",
        )
        .default_header("Accept-Language", "zh-cn")
        .build()?;
    TianyanchaApi::new(authorization, auth_token, config)
}

pub fn create_tianyancha_huawei_api(
    access_key: impl Into<String>,
    secret_key: impl Into<String>,
) -> CreatesResult<TianyanchaHuaweiApi> {
    let config = ApiConfig::builder("http://kzenterprisewmh.apistore.huaweicloud.com")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    TianyanchaHuaweiApi::new(access_key, secret_key, config)
}
