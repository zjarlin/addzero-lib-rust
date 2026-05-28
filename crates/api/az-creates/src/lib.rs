#![forbid(unsafe_code)]

//! 聚合式第三方 API 客户端集合，统一封装多种外部服务调用。
//!
//! `az-creates` 是一个"门面"（facade）crate，将多个独立的 API 客户端整合到统一入口，
//! 涵盖以下能力：
//!
//! - **Maven Central**（[`maven`]）—— 搜索 Maven 仓库中的构件（`MavenArtifact`），支持按
//!   groupId、artifactId 或坐标组合查询。
//! - **天眼查**（[`tianyancha`]）—— 企业信息查询，提供标准版（`TianyanchaApi`）与华为云
//!   代理版（`TianyanchaHuaweiApi`）两种接入方式，支持公司搜索与详情查询。
//! - **临时邮箱**（re-export 自 `az-temp-mail`）—— 创建临时邮箱、收发邮件、管理地址等，
//!   支持 Cloudflare、mail.tm、Emailnator 三种后端和 provider factory 注入。
//! - **短信接码**（re-export 自 `az-sms`）—— 统一 5sim、Grizzly SMS 等 provider 的
//!   trait-object 工厂边界，供注册/验证流程依赖注入。
//! - **邮件发送**（re-export 自 `az-email`）—— 统一 SMTP sender 与 sender factory，
//!   供通知、验证码和工作流消息发送依赖注入。
//! - **音乐搜索/生成**（re-export 自 `az-music`）—— 网易云音乐搜索、歌词获取、
//!   Suno AI 音乐生成等。
//!
//! 所有 API 客户端通过 [`Creates`] 结构体的便捷方法创建，也可通过各子模块直接构造。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_creates::Creates;
//!
//! // 搜索 Maven Central
//! let maven = Creates::maven_central()?;
//! let artifacts = maven.search_by_group_id("com.google.guava", 10)?;
//!
//! // 创建临时邮箱
//! let temp_mail = Creates::temp_mail("https://api.example.com")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod config;
mod error;
mod http;
pub mod maven;
pub mod tianyancha;
mod util;

#[cfg(test)]
mod tests;

use az_derive_aliases::{apply, plain_default_copy_eq};

pub use az_email::{
    BoxEmailSender, BuiltinEmailSenderFactory, EmailConfig, EmailConfigBuilder, EmailError,
    EmailMessage, EmailMessageBuilder, EmailSender, EmailSenderConfig, EmailSenderFactory,
    EmailSenderKind, SmtpEmailSender, build_email_sender,
};
pub use az_music::{
    BatchFetchRequest, ConcatSongsRequest, GenerateLyricsRequest, LyricContent, LyricResponse,
    Music, MusicAlbum, MusicArtist, MusicCreator, MusicPlaylist, MusicPrivilege, MusicSearchApi,
    MusicSearchRequest, MusicSearchResponse, MusicSearchResult, MusicSearchType, MusicSong,
    NeteaseMusicApi, SongDetailResponse, SongWithLyric, SunoApi, SunoMusicRequest, SunoTask,
    create_netease_api as create_music_search_api, create_suno_api,
};
pub use az_sms::{
    error::{SmsError, SmsResult},
    fivesim::{FivesimClient, FivesimConfig, FivesimConfigBuilder},
    fivesim_factory::{build_fivesim_provider, build_fivesim_provider_with},
    grizzlysms::{GrizzlySmsClient, GrizzlySmsConfig, GrizzlySmsConfigBuilder},
    model::{
        SmsActivationRequest, SmsHostingRequest, SmsInbox, SmsMessage, SmsOrder, SmsOrderStatus,
        SmsProfile, WaitForSmsOptions,
    },
    provider::{
        BoxSmsProvider, BuiltinSmsProviderFactory, SmsProvider, SmsProviderConfig,
        SmsProviderFactory, SmsProviderKind, build_sms_provider,
    },
};
pub use az_temp_mail::{
    AddressCredential as TempMailAddressCredential,
    AddressLoginRequest as TempMailAddressLoginRequest, AddressSettings as TempMailAddressSettings,
    ApiConfig as TempMailApiConfig, ApiConfigBuilder as TempMailApiConfigBuilder,
    BoxTempMailProvider, BuiltinTempMailProviderFactory, CloudflareTempMailApi,
    CreateMailboxRequest as TempMailCreateMailboxRequest, EmailnatorEmailMode,
    EmailnatorEmailRequest, EmailnatorTempMailApi, ListResponse as TempMailListResponse, MailRow,
    MailTmDomain, MailTmTempMailApi, NewAddressRequest as TempMailNewAddressRequest,
    PageRequest as TempMailPageRequest, ParsedMailAttachment as TempMailParsedMailAttachment,
    ParsedMailRow, SendMailRequest, SuccessResponse as TempMailSuccessResponse, TempMail,
    TempMailApi, TempMailError, TempMailMailbox, TempMailMessageDetail, TempMailMessageSummary,
    TempMailProvider, TempMailProviderConfig, TempMailProviderFactory, TempMailProviderKind,
    TempMailRecipient, TempMailResult, TempMailSettings, build_temp_mail_provider,
    create_emailnator_api, create_mail_tm_api, create_temp_mail_api, extract_first_http_link,
};
pub use config::{ApiConfig, ApiConfigBuilder};
pub use error::{CreatesError, CreatesResult};
pub use maven::{MavenArtifact, MavenCentralApi, create_maven_central_api};
pub use tianyancha::{
    TianyanchaApi, TianyanchaCompany, TianyanchaCompanyDetail, TianyanchaCompanySearchData,
    TianyanchaHuaweiApi, TianyanchaHuaweiCompany, TianyanchaHuaweiCompanySearchData,
    TianyanchaHuaweiPageInfo, create_tianyancha_api, create_tianyancha_huawei_api,
};

/// 第三方 API 客户端创建门面。
///
/// `Creates` 不保存状态，只提供统一的默认构造入口；需要测试替身、租户隔离或运行时注入时，
/// 优先使用 `*_with_config` 或 `*_with_factory` 方法显式传入配置和工厂。
#[apply(plain_default_copy_eq)]
pub struct Creates;

impl Creates {
    /// 使用默认配置创建 Maven Central 客户端。
    pub fn maven_central() -> CreatesResult<MavenCentralApi> {
        create_maven_central_api()
    }

    /// 使用显式配置创建 Maven Central 客户端。
    pub fn maven_central_with_config(config: ApiConfig) -> CreatesResult<MavenCentralApi> {
        MavenCentralApi::new(config)
    }

    /// 创建 Cloudflare Worker 兼容临时邮箱客户端。
    pub fn temp_mail(base_url: impl Into<String>) -> TempMailResult<TempMailApi> {
        create_temp_mail_api(base_url)
    }

    /// 使用显式配置创建 Cloudflare Worker 兼容临时邮箱客户端。
    pub fn temp_mail_with_config(config: TempMailApiConfig) -> TempMailResult<TempMailApi> {
        TempMailApi::new(config)
    }

    /// 创建 Cloudflare Worker 兼容临时邮箱客户端。
    pub fn temp_mail_cloudflare(
        base_url: impl Into<String>,
    ) -> TempMailResult<CloudflareTempMailApi> {
        create_temp_mail_api(base_url)
    }

    /// 使用显式配置创建 Cloudflare Worker 兼容临时邮箱客户端。
    pub fn temp_mail_cloudflare_with_config(
        config: TempMailApiConfig,
    ) -> TempMailResult<CloudflareTempMailApi> {
        CloudflareTempMailApi::new(config)
    }

    /// 创建 `mail.tm` 临时邮箱客户端。
    pub fn temp_mail_mail_tm() -> TempMailResult<MailTmTempMailApi> {
        create_mail_tm_api()
    }

    /// 使用显式配置创建 `mail.tm` 临时邮箱客户端。
    pub fn temp_mail_mail_tm_with_config(
        config: TempMailApiConfig,
    ) -> TempMailResult<MailTmTempMailApi> {
        MailTmTempMailApi::new(config)
    }

    /// 创建 Emailnator 临时邮箱客户端。
    pub fn temp_mail_emailnator() -> TempMailResult<EmailnatorTempMailApi> {
        create_emailnator_api()
    }

    /// 使用显式配置创建 Emailnator 临时邮箱客户端。
    pub fn temp_mail_emailnator_with_config(
        config: TempMailApiConfig,
    ) -> TempMailResult<EmailnatorTempMailApi> {
        EmailnatorTempMailApi::new(config)
    }

    /// 按 provider 配置构建临时邮箱 trait object。
    pub fn temp_mail_provider(
        config: TempMailProviderConfig,
    ) -> TempMailResult<BoxTempMailProvider> {
        build_temp_mail_provider(config)
    }

    /// 通过调用方提供的工厂构建临时邮箱 provider。
    ///
    /// 这是临时邮箱能力的依赖注入入口，适合测试替身、自定义 provider 或上层插件化装配。
    pub fn temp_mail_provider_with_factory(
        factory: &dyn TempMailProviderFactory,
        config: TempMailProviderConfig,
    ) -> TempMailResult<BoxTempMailProvider> {
        factory.build_provider(config)
    }

    /// 按 provider 配置构建短信接码 trait object。
    pub fn sms_provider(config: SmsProviderConfig) -> SmsResult<BoxSmsProvider> {
        build_sms_provider(config)
    }

    /// 通过调用方提供的工厂构建短信接码 provider。
    ///
    /// 这是短信能力的依赖注入入口，避免门面层硬编码所有运行时 provider。
    pub fn sms_provider_with_factory(
        factory: &dyn SmsProviderFactory,
        config: SmsProviderConfig,
    ) -> SmsResult<BoxSmsProvider> {
        factory.build_provider(config)
    }

    /// 使用 token 快速创建 5sim 短信接码 provider。
    pub fn fivesim_sms(token: &str) -> SmsResult<BoxSmsProvider> {
        build_fivesim_provider(token)
    }

    /// 通过工厂创建 5sim 短信接码 provider。
    pub fn fivesim_sms_with_factory(
        factory: &dyn SmsProviderFactory,
        token: &str,
    ) -> SmsResult<BoxSmsProvider> {
        build_fivesim_provider_with(factory, token)
    }

    /// 按 sender 配置构建邮件发送 trait object。
    pub fn email_sender(config: EmailSenderConfig) -> Result<BoxEmailSender, EmailError> {
        build_email_sender(config)
    }

    /// 通过调用方提供的工厂构建邮件发送器。
    ///
    /// 这是邮件能力的依赖注入入口，适合替换 SMTP 实现或接入自定义 sender。
    pub fn email_sender_with_factory(
        factory: &dyn EmailSenderFactory,
        config: EmailSenderConfig,
    ) -> Result<BoxEmailSender, EmailError> {
        factory.build_sender(config)
    }

    /// 使用 SMTP 配置快速创建邮件发送器。
    pub fn smtp_email(config: EmailConfig) -> Result<BoxEmailSender, EmailError> {
        build_email_sender(config.into())
    }

    /// 通过工厂使用 SMTP 配置创建邮件发送器。
    pub fn smtp_email_with_factory(
        factory: &dyn EmailSenderFactory,
        config: EmailConfig,
    ) -> Result<BoxEmailSender, EmailError> {
        factory.build_sender(config.into())
    }

    /// 使用默认配置创建网易云音乐搜索客户端。
    pub fn music_search() -> CreatesResult<MusicSearchApi> {
        Ok(create_music_search_api()?)
    }

    /// 使用显式配置创建网易云音乐搜索客户端。
    pub fn music_search_with_config(config: ApiConfig) -> CreatesResult<MusicSearchApi> {
        Ok(MusicSearchApi::new(config)?)
    }

    /// 使用默认配置创建 Suno 音乐生成客户端。
    pub fn suno(api_token: impl Into<String>) -> CreatesResult<SunoApi> {
        Ok(create_suno_api(api_token)?)
    }

    /// 使用显式配置创建 Suno 音乐生成客户端。
    pub fn suno_with_config(
        api_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<SunoApi> {
        Ok(SunoApi::new(api_token, config)?)
    }

    /// 使用默认配置创建天眼查普通接口客户端。
    pub fn tianyancha(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> CreatesResult<TianyanchaApi> {
        create_tianyancha_api(authorization, auth_token)
    }

    /// 使用显式配置创建天眼查普通接口客户端。
    pub fn tianyancha_with_config(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<TianyanchaApi> {
        TianyanchaApi::new(authorization, auth_token, config)
    }

    /// 使用默认配置创建天眼查华为云签名版客户端。
    pub fn tianyancha_huawei(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> CreatesResult<TianyanchaHuaweiApi> {
        create_tianyancha_huawei_api(access_key, secret_key)
    }

    /// 使用显式配置创建天眼查华为云签名版客户端。
    pub fn tianyancha_huawei_with_config(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<TianyanchaHuaweiApi> {
        TianyanchaHuaweiApi::new(access_key, secret_key, config)
    }
}
