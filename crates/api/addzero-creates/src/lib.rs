#![forbid(unsafe_code)]

mod config;
mod error;
mod http;
pub mod maven;
pub mod tianyancha;
mod util;

#[cfg(test)]
mod tests;

pub use addzero_music::{
    BatchFetchRequest, ConcatSongsRequest, GenerateLyricsRequest, LyricContent, LyricResponse,
    Music, MusicAlbum, MusicArtist, MusicCreator, MusicPlaylist, MusicPrivilege, MusicSearchApi,
    MusicSearchRequest, MusicSearchResponse, MusicSearchResult, MusicSearchType, MusicSong,
    NeteaseMusicApi, SongDetailResponse, SongWithLyric, SunoApi, SunoMusicRequest, SunoTask,
    create_netease_api as create_music_search_api, create_suno_api,
};
pub use addzero_temp_mail::{
    ApiConfig as TempMailApiConfig, ApiConfigBuilder as TempMailApiConfigBuilder, TempMailApi,
    TempMailDomain, TempMailError, TempMailMailbox, TempMailMessageDetail, TempMailMessageSummary,
    TempMailRecipient, TempMailResult, create_temp_mail_api,
};
pub use config::{ApiConfig, ApiConfigBuilder};
pub use error::{CreatesError, CreatesResult};
pub use maven::{MavenArtifact, MavenCentralApi, create_maven_central_api};
pub use tianyancha::{
    TianyanchaApi, TianyanchaCompany, TianyanchaCompanyDetail, TianyanchaCompanySearchData,
    TianyanchaHuaweiApi, TianyanchaHuaweiCompany, TianyanchaHuaweiCompanySearchData,
    TianyanchaHuaweiPageInfo, create_tianyancha_api, create_tianyancha_huawei_api,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Creates;

impl Creates {
    pub fn maven_central() -> CreatesResult<MavenCentralApi> {
        create_maven_central_api()
    }

    pub fn maven_central_with_config(config: ApiConfig) -> CreatesResult<MavenCentralApi> {
        MavenCentralApi::new(config)
    }

    pub fn temp_mail() -> TempMailResult<TempMailApi> {
        create_temp_mail_api()
    }

    pub fn temp_mail_with_config(config: TempMailApiConfig) -> TempMailResult<TempMailApi> {
        TempMailApi::new(config)
    }

    pub fn music_search() -> CreatesResult<MusicSearchApi> {
        Ok(create_music_search_api()?)
    }

    pub fn music_search_with_config(config: ApiConfig) -> CreatesResult<MusicSearchApi> {
        Ok(MusicSearchApi::new(config)?)
    }

    pub fn suno(api_token: impl Into<String>) -> CreatesResult<SunoApi> {
        Ok(create_suno_api(api_token)?)
    }

    pub fn suno_with_config(
        api_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<SunoApi> {
        Ok(SunoApi::new(api_token, config)?)
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
