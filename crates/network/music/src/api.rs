//! 音乐平台 HTTP API 客户端，封装网易云音乐搜索与 Suno AI 音乐生成接口。
//!
//! # 概述
//!
//! 本 crate 提供两大类音乐服务的同步 HTTP 客户端：
//!
//! - **网易云音乐**（[`MusicSearchApi`] / [`NeteaseMusicApi`]）——支持歌曲、专辑、歌手、
//!   歌单搜索，歌词查询（含原文、翻译、罗马音），歌曲详情获取，以及按歌名+歌手或歌词片段
//!   模糊检索。
//! - **Suno AI 音乐生成**（[`SunoApi`]）——支持 AI 音乐生成、歌词生成、歌曲拼接，
//!   以及单任务 / 批量任务的轮询等待（可配置超时与轮询间隔）。
//!
//! 统一入口为 [`Music`] 结构体，通过 `Music::netease()` 或 `Music::suno(token)` 快速创建
//! 对应的 API 客户端实例。
//!
//! # 核心类型
//!
//! | 类型 | 说明 |
//! |------|------|
//! | [`Music`] | 客户端工厂，提供 `netease()` / `suno()` 等便捷构造方法 |
//! | [`ApiConfig`] / [`ApiConfigBuilder`] | HTTP 客户端配置（基础 URL、超时、默认请求头等） |
//! | [`MusicSearchApi`] | 网易云音乐搜索与歌曲信息 API 客户端 |
//! | [`SunoApi`] | Suno AI 音乐/歌词生成 API 客户端 |
//! | [`MusicSearchType`] | 搜索维度枚举（歌曲、专辑、歌手、歌单、用户、MV、歌词、电台、视频） |
//! | [`MusicSong`] / [`MusicArtist`] / [`MusicAlbum`] / [`MusicPlaylist`] | 网易云音乐核心数据模型 |
//! | [`LyricResponse`] / [`LyricContent`] | 歌词响应（含原文、翻译、罗马音） |
//! | [`SunoTask`] / [`SunoMusicRequest`] | Suno 任务与音乐生成请求模型 |
//!
//! # 关键特性
//!
//! - 网易云音乐：歌曲/专辑/歌手/歌单搜索、歌词查询、歌曲详情、按歌名+歌手或歌词片段检索
//! - Suno AI：音乐生成、歌词生成、歌曲拼接、单任务轮询等待与批量任务轮询等待
//! - 统一的 [`ApiConfig`] 配置体系，支持自定义基础 URL、连接/请求超时、User-Agent 及默认请求头
//! - 全部为同步阻塞调用（基于 `reqwest::blocking`），使用 `#![forbid(unsafe_code)]` 保证无 unsafe 代码

use anyhow::{Context, Result, bail};
use az_str::api::trim_non_blank;
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;
use std::thread;
use std::time::{Duration, Instant};

/// 第三方音乐 HTTP API 的通用客户端配置。
///
/// 配置只描述 HTTP 层行为；网易云 Referer、Suno token 等服务特定约束由工厂函数或对应客户端处理。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiConfig {
    /// API 基础 URL。
    pub base_url: String,
    /// TCP/HTTPS 连接超时。
    pub connect_timeout: Duration,
    /// 单次请求总超时。
    pub request_timeout: Duration,
    /// 可选 User-Agent；为 `None` 时不主动设置。
    pub user_agent: Option<String>,
    /// 默认请求头。
    pub default_headers: BTreeMap<String, String>,
}

impl ApiConfig {
    /// 创建 HTTP API 配置构建器。
    ///
    /// 默认连接超时 10 秒、请求超时 20 秒，并使用 `az-music/<version>` 作为 User-Agent。
    pub fn builder(base_url: impl Into<String>) -> ApiConfigBuilder {
        ApiConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    /// 校验 HTTP 配置的本地约束。
    ///
    /// 该方法不发起网络请求，只检查基础 URL 为空和超时为零这类明显错误。
    pub fn validate(&self) -> Result<()> {
        if self.base_url.trim().is_empty() {
            bail!("invalid config: base_url cannot be blank");
        }
        if self.connect_timeout.is_zero() {
            bail!("invalid config: connect_timeout cannot be zero");
        }
        if self.request_timeout.is_zero() {
            bail!("invalid config: request_timeout cannot be zero");
        }
        Ok(())
    }
}

/// `ApiConfig` 的链式构建器。
#[derive(Clone, Debug)]
pub struct ApiConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl ApiConfigBuilder {
    /// 设置连接超时。
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// 设置请求总超时。
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// 设置 User-Agent。
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// 清除默认 User-Agent。
    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    /// 追加默认请求头。
    ///
    /// header 名称和值会在构建 HTTP 客户端时校验。
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// 完成构建并执行本地配置校验。
    pub fn build(self) -> Result<ApiConfig> {
        let config = ApiConfig {
            base_url: self.base_url,
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            user_agent: self.user_agent,
            default_headers: self.default_headers,
        };
        config.validate()?;
        Ok(config)
    }
}

/// 音乐客户端工厂门面。
///
/// 该类型不持有状态，只提供默认配置下的网易云音乐和 Suno 客户端创建入口。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Music;

impl Music {
    /// 使用默认网易云音乐配置创建搜索 API 客户端。
    pub fn netease() -> Result<MusicSearchApi> {
        create_netease_api()
    }

    /// 使用调用方提供的 HTTP 配置创建网易云音乐客户端。
    pub fn netease_with_config(config: ApiConfig) -> Result<MusicSearchApi> {
        MusicSearchApi::new(config)
    }

    /// 使用默认 Suno API 配置和 bearer token 创建客户端。
    pub fn suno(api_token: impl Into<String>) -> Result<SunoApi> {
        create_suno_api(api_token)
    }

    /// 使用调用方提供的 HTTP 配置创建 Suno 客户端。
    pub fn suno_with_config(api_token: impl Into<String>, config: ApiConfig) -> Result<SunoApi> {
        SunoApi::new(api_token, config)
    }
}

/// 创建默认网易云音乐搜索客户端。
///
/// 默认配置会设置网易云需要的 `Referer` 和浏览器风格 User-Agent。
pub fn create_netease_api() -> Result<MusicSearchApi> {
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

/// 创建默认 Suno 客户端。
///
/// `api_token` 会作为 bearer token 使用，并在 `Debug` 输出中脱敏。
pub fn create_suno_api(api_token: impl Into<String>) -> Result<SunoApi> {
    let config = ApiConfig::builder("https://api.vectorengine.ai")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    SunoApi::new(api_token, config)
}

/// 网易云音乐搜索类型。
///
/// `code()` / `from_code()` 使用 snake_case 机器码；`value()` 返回网易云 `type` 查询参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum MusicSearchType {
    /// 歌曲搜索，网易云 type=1。
    #[default]
    Song,
    /// 专辑搜索，网易云 type=10。
    Album,
    /// 歌手搜索，网易云 type=100。
    Artist,
    /// 歌单搜索，网易云 type=1000。
    Playlist,
    /// 用户搜索，网易云 type=1002。
    User,
    /// MV 搜索，网易云 type=1004。
    Mv,
    /// 歌词搜索，网易云 type=1006。
    Lyric,
    /// 电台搜索，网易云 type=1009。
    Radio,
    /// 视频搜索，网易云 type=1014。
    Video,
}

impl MusicSearchType {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

impl MusicSearchType {
    /// 返回网易云搜索接口的 `type` 参数值。
    pub const fn value(self) -> u16 {
        match self {
            Self::Song => 1,
            Self::Album => 10,
            Self::Artist => 100,
            Self::Playlist => 1000,
            Self::User => 1002,
            Self::Mv => 1004,
            Self::Lyric => 1006,
            Self::Radio => 1009,
            Self::Video => 1014,
        }
    }
}

/// 网易云音乐搜索请求。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MusicSearchRequest {
    /// 搜索关键词。
    pub keywords: String,
    /// 搜索类型。
    pub search_type: MusicSearchType,
    /// 返回数量；实际请求时会至少为 1。
    pub limit: usize,
    /// 零基分页偏移。
    pub offset: usize,
}

impl MusicSearchRequest {
    /// 创建歌曲搜索请求，默认 `limit=30`、`offset=0`。
    pub fn new(keywords: impl Into<String>) -> Self {
        Self {
            keywords: keywords.into(),
            search_type: MusicSearchType::Song,
            limit: 30,
            offset: 0,
        }
    }

    /// 设置搜索类型。
    pub fn search_type(mut self, value: MusicSearchType) -> Self {
        self.search_type = value;
        self
    }

    /// 设置返回数量。
    pub fn limit(mut self, value: usize) -> Self {
        self.limit = value;
        self
    }

    /// 设置零基分页偏移。
    pub fn offset(mut self, value: usize) -> Self {
        self.offset = value;
        self
    }
}

/// 网易云音乐搜索和歌曲信息客户端。
#[derive(Clone, Debug)]
pub struct MusicSearchApi {
    http: HttpApiClient,
}

/// 网易云音乐客户端的兼容类型别名。
pub type NeteaseMusicApi = MusicSearchApi;

impl MusicSearchApi {
    /// 使用指定 HTTP 配置创建网易云音乐客户端。
    pub fn new(config: ApiConfig) -> Result<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    /// 执行通用网易云搜索。
    ///
    /// 空关键词或业务响应码非 200 时返回错误。
    pub fn search(&self, request: MusicSearchRequest) -> Result<MusicSearchResult> {
        let keywords = trim_non_blank(Some(request.keywords.as_str()))
            .context("invalid config: music keywords cannot be blank")?;
        let response = self
            .http
            .get("search/get/web")?
            .query(&[
                ("s", keywords.to_owned()),
                ("type", request.search_type.value().to_string()),
                ("limit", request.limit.max(1).to_string()),
                ("offset", request.offset.to_string()),
            ])
            .send()?;
        let response: MusicSearchResponse = HttpApiClient::read_json(response)?;
        ensure_code_200(response.code, response.msg.as_deref(), "music search")?;
        Ok(response.result.unwrap_or_default())
    }

    /// 搜索歌曲。
    pub fn search_songs(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicSong>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Song)
                    .limit(limit)
                    .offset(offset),
            )?
            .songs)
    }

    /// 搜索歌手。
    pub fn search_artists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicArtist>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Artist)
                    .limit(limit)
                    .offset(offset),
            )?
            .artists)
    }

    /// 搜索专辑。
    pub fn search_albums(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicAlbum>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Album)
                    .limit(limit)
                    .offset(offset),
            )?
            .albums)
    }

    /// 搜索歌单。
    pub fn search_playlists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MusicPlaylist>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Playlist)
                    .limit(limit)
                    .offset(offset),
            )?
            .playlists)
    }

    /// 获取指定歌曲的歌词响应。
    ///
    /// 请求会同时启用原文歌词和翻译歌词参数。
    pub fn get_lyric(&self, song_id: i64) -> Result<LyricResponse> {
        let response = self
            .http
            .get("song/lyric")?
            .query(&[
                ("id", song_id.to_string()),
                ("lv", "1".to_owned()),
                ("tv", "1".to_owned()),
            ])
            .send()?;
        let response: LyricResponse = HttpApiClient::read_json(response)?;
        ensure_code_200(response.code, None, "get lyric")?;
        Ok(response)
    }

    /// 批量获取歌曲详情。
    ///
    /// 空 id 列表直接返回空列表，不发起 HTTP 请求。
    pub fn get_song_detail(&self, song_ids: &[i64]) -> Result<Vec<MusicSong>> {
        if song_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids = song_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let response = self
            .http
            .get("song/detail")?
            .query(&[("ids", format!("[{ids}]"))])
            .send()?;
        let response: SongDetailResponse = HttpApiClient::read_json(response)?;
        ensure_code_200(response.code, None, "get song detail")?;
        Ok(response.songs)
    }

    /// 按歌名和可选歌手名搜索歌曲，并在本地按歌手名做二次过滤。
    pub fn search_by_song_and_artist(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> Result<Vec<MusicSong>> {
        let keywords = match trim_non_blank(artist_name) {
            Some(artist) => format!("{} {artist}", song_name.as_ref().trim()),
            None => song_name.as_ref().trim().to_owned(),
        };
        let songs = self.search_songs(keywords, 10, 0)?;
        let Some(artist_name) = trim_non_blank(artist_name) else {
            return Ok(songs);
        };
        let needle = artist_name.to_lowercase();
        Ok(songs
            .into_iter()
            .filter(|song| {
                song.artists
                    .iter()
                    .any(|artist| artist.name.to_lowercase().contains(&needle))
            })
            .collect())
    }

    /// 按歌词片段搜索歌曲。
    pub fn search_by_lyric(&self, lyric_fragment: impl Into<String>) -> Result<Vec<MusicSong>> {
        Ok(self
            .search(
                MusicSearchRequest::new(lyric_fragment)
                    .search_type(MusicSearchType::Lyric)
                    .limit(20),
            )?
            .songs)
    }

    /// 先按歌名/歌手检索首个候选歌曲，再获取歌词。
    pub fn get_lyric_by_song_name(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> Result<Option<LyricResponse>> {
        let songs = self.search_by_song_and_artist(song_name, artist_name)?;
        if let Some(song) = songs.first() {
            return self.get_lyric(song.id).map(Some);
        }
        Ok(None)
    }

    /// 按歌词片段搜索歌曲，并为候选歌曲逐个拉取歌词。
    ///
    /// 单首歌词获取失败会被跳过；`filter_empty` 为 true 时会过滤空原文歌词。
    pub fn get_lyrics_by_fragment(
        &self,
        lyric_fragment: impl Into<String>,
        limit: usize,
        filter_empty: bool,
    ) -> Result<Vec<SongWithLyric>> {
        let songs = self.search_by_lyric(lyric_fragment)?;
        let mut items = Vec::new();
        for song in songs.into_iter().take(limit.max(1)) {
            let lyric = match self.get_lyric(song.id) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if filter_empty
                && lyric
                    .lrc
                    .as_ref()
                    .and_then(|item| trim_non_blank(item.lyric.as_deref()))
                    .is_none()
            {
                continue;
            }
            items.push(SongWithLyric { song, lyric });
        }
        Ok(items)
    }
}

/// 网易云音乐搜索接口原始响应。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicSearchResponse {
    /// 网易云业务响应码，通常 200 表示成功。
    #[serde(default)]
    pub code: i32,
    /// 可选业务消息。
    #[serde(default)]
    pub msg: Option<String>,
    /// 搜索结果主体。
    #[serde(default)]
    pub result: Option<MusicSearchResult>,
}

/// 网易云音乐搜索结果集合。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicSearchResult {
    /// 歌曲结果。
    #[serde(default)]
    pub songs: Vec<MusicSong>,
    /// 歌曲总数。
    #[serde(rename = "songCount", default)]
    pub song_count: Option<i64>,
    /// 专辑结果。
    #[serde(default)]
    pub albums: Vec<MusicAlbum>,
    /// 专辑总数。
    #[serde(rename = "albumCount", default)]
    pub album_count: Option<i64>,
    /// 歌手结果。
    #[serde(default)]
    pub artists: Vec<MusicArtist>,
    /// 歌手总数。
    #[serde(rename = "artistCount", default)]
    pub artist_count: Option<i64>,
    /// 歌单结果。
    #[serde(default)]
    pub playlists: Vec<MusicPlaylist>,
    /// 歌单总数。
    #[serde(rename = "playlistCount", default)]
    pub playlist_count: Option<i64>,
}

/// 网易云歌曲模型。
///
/// 字段兼容搜索接口和歌曲详情接口，支持 `ar` / `al` / `dt` 等网易云短字段别名。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicSong {
    /// 歌曲 id。
    #[serde(default)]
    pub id: i64,
    /// 歌曲名称。
    #[serde(default)]
    pub name: String,
    /// 歌手列表。
    #[serde(default, alias = "ar")]
    pub artists: Vec<MusicArtist>,
    /// 所属专辑。
    #[serde(default, alias = "al")]
    pub album: Option<MusicAlbum>,
    /// 时长，单位通常为毫秒。
    #[serde(default, alias = "dt")]
    pub duration: Option<i64>,
    /// 关联 MV id。
    #[serde(rename = "mvid", default)]
    pub mv_id: Option<i64>,
    /// 版权/付费类型。
    #[serde(default)]
    pub fee: Option<i32>,
    /// 播放权限信息。
    #[serde(default)]
    pub privilege: Option<MusicPrivilege>,
}

/// 网易云歌手模型。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicArtist {
    /// 歌手 id。
    #[serde(default)]
    pub id: i64,
    /// 歌手名称。
    #[serde(default)]
    pub name: String,
    /// 头像或封面图 URL。
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    /// 别名列表。
    #[serde(default)]
    pub alias: Vec<String>,
    /// 专辑数量。
    #[serde(rename = "albumSize", default)]
    pub album_size: Option<i32>,
    /// 歌曲数量。
    #[serde(rename = "musicSize", default)]
    pub music_size: Option<i32>,
}

/// 网易云专辑模型。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicAlbum {
    /// 专辑 id。
    #[serde(default)]
    pub id: i64,
    /// 专辑名称。
    #[serde(default)]
    pub name: String,
    /// 专辑封面 URL。
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    /// 专辑歌手。
    #[serde(default)]
    pub artist: Option<MusicArtist>,
    /// 发布时间戳，沿用上游毫秒时间戳语义。
    #[serde(rename = "publishTime", default)]
    pub publish_time: Option<i64>,
    /// 专辑歌曲数量。
    #[serde(default)]
    pub size: Option<i32>,
}

/// 网易云歌单模型。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicPlaylist {
    /// 歌单 id。
    #[serde(default)]
    pub id: i64,
    /// 歌单名称。
    #[serde(default)]
    pub name: String,
    /// 封面图 URL。
    #[serde(rename = "coverImgUrl", default)]
    pub cover_img_url: Option<String>,
    /// 创建者信息。
    #[serde(default)]
    pub creator: Option<MusicCreator>,
    /// 歌曲数量。
    #[serde(rename = "trackCount", default)]
    pub track_count: Option<i32>,
    /// 播放次数。
    #[serde(rename = "playCount", default)]
    pub play_count: Option<i64>,
    /// 歌单描述。
    #[serde(default)]
    pub description: Option<String>,
}

/// 网易云用户/歌单创建者模型。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicCreator {
    /// 用户 id。
    #[serde(rename = "userId", default)]
    pub user_id: i64,
    /// 用户昵称。
    #[serde(default)]
    pub nickname: String,
    /// 用户头像 URL。
    #[serde(rename = "avatarUrl", default)]
    pub avatar_url: Option<String>,
}

/// 网易云播放权限模型。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicPrivilege {
    /// 权限所属歌曲 id。
    #[serde(default)]
    pub id: Option<i64>,
    /// 费用/版权类型。
    #[serde(default)]
    pub fee: Option<i32>,
    /// 上游状态字段。
    #[serde(default)]
    pub st: Option<i32>,
    /// 可播放码率。
    #[serde(default)]
    pub pl: Option<i32>,
    /// 可下载码率。
    #[serde(default)]
    pub dl: Option<i32>,
    /// 最大码率。
    #[serde(default)]
    pub maxbr: Option<i32>,
}

/// 网易云歌词响应。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LyricResponse {
    /// 业务响应码。
    #[serde(default)]
    pub code: i32,
    /// 原文歌词。
    #[serde(default)]
    pub lrc: Option<LyricContent>,
    /// 翻译歌词。
    #[serde(default)]
    pub tlyric: Option<LyricContent>,
    /// 罗马音歌词。
    #[serde(default)]
    pub romalrc: Option<LyricContent>,
}

/// 单段歌词内容。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LyricContent {
    /// 歌词版本。
    #[serde(default)]
    pub version: Option<i32>,
    /// 歌词文本，通常包含时间戳行。
    #[serde(default)]
    pub lyric: Option<String>,
}

/// 网易云歌曲详情响应。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SongDetailResponse {
    /// 业务响应码。
    #[serde(default)]
    pub code: i32,
    /// 歌曲列表。
    #[serde(default)]
    pub songs: Vec<MusicSong>,
    /// 权限列表。
    #[serde(default)]
    pub privileges: Vec<MusicPrivilege>,
}

/// 歌曲和歌词的组合结果。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SongWithLyric {
    /// 歌曲信息。
    pub song: MusicSong,
    /// 歌词响应。
    pub lyric: LyricResponse,
}

/// Suno AI 音乐生成客户端。
///
/// `api_token` 仅用于 bearer auth，并在 `Debug` 输出中脱敏。
#[derive(Clone, derive_more::Debug)]
pub struct SunoApi {
    #[debug(skip)]
    api_token: String,
    http: HttpApiClient,
}

impl SunoApi {
    /// 使用 API token 和 HTTP 配置创建 Suno 客户端。
    pub fn new(api_token: impl Into<String>, config: ApiConfig) -> Result<Self> {
        let api_token = api_token.into();
        if trim_non_blank(Some(api_token.as_str())).is_none() {
            bail!("invalid config: suno api_token cannot be blank");
        }
        Ok(Self {
            api_token,
            http: HttpApiClient::new(config)?,
        })
    }

    /// 提交 Suno 音乐生成任务并返回任务 id。
    pub fn generate_music(&self, request: &SunoMusicRequest) -> Result<String> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("suno/submit/music")?,
            Some(self.api_token.as_str()),
        )
        .json(request)
        .send()?;
        let response: ApiEnvelope<String> = HttpApiClient::read_json(response)?;
        response.into_data("generate suno music")
    }

    /// 提交歌词生成请求并返回生成文本。
    pub fn generate_lyrics(&self, prompt: impl AsRef<str>) -> Result<String> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("suno/lyrics")?,
            Some(self.api_token.as_str()),
        )
        .json(&GenerateLyricsRequest {
            prompt: prompt.as_ref().trim().to_owned(),
        })
        .send()?;
        let response: ApiEnvelope<String> = HttpApiClient::read_json(response)?;
        response.into_data("generate suno lyrics")
    }

    /// 提交歌曲拼接任务并返回任务 id。
    pub fn concat_songs(&self, clip_id: impl AsRef<str>) -> Result<String> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("suno/concat")?,
            Some(self.api_token.as_str()),
        )
        .json(&ConcatSongsRequest {
            clip_id: clip_id.as_ref().trim().to_owned(),
        })
        .send()?;
        let response: ApiEnvelope<String> = HttpApiClient::read_json(response)?;
        response.into_data("concat suno songs")
    }

    /// 查询单个 Suno 任务。
    ///
    /// 上游成功但没有 data 时返回 `Ok(None)`。
    pub fn fetch_task(&self, task_id: impl AsRef<str>) -> Result<Option<SunoTask>> {
        let path = format!("suno/fetch/{}", task_id.as_ref().trim());
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(self.api_token.as_str()))
                .send()?;
        let response: ApiEnvelope<SunoTask> = HttpApiClient::read_json(response)?;
        response.into_optional_data("fetch suno task")
    }

    /// 批量查询 Suno 任务。
    ///
    /// 上游成功但没有 data 时返回空列表。
    pub fn batch_fetch_tasks<I, S>(&self, task_ids: I) -> Result<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let ids = task_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("suno/fetch")?,
            Some(self.api_token.as_str()),
        )
        .json(&BatchFetchRequest { ids })
        .send()?;
        let response: ApiEnvelope<Vec<SunoTask>> = HttpApiClient::read_json(response)?;
        Ok(response
            .into_optional_data("batch fetch suno task")?
            .unwrap_or_default())
    }

    /// 使用默认 10 分钟超时和 10 秒轮询间隔等待单个任务完成。
    pub fn wait_for_completion(&self, task_id: impl AsRef<str>) -> Result<SunoTask> {
        self.wait_for_completion_with(
            task_id,
            Duration::from_secs(600),
            Duration::from_secs(10),
            |_| {},
        )
    }

    /// 使用自定义超时、轮询间隔和状态回调等待单个任务完成。
    ///
    /// `complete` 和 `streaming` 视为完成，`error` 会映射为错误。
    pub fn wait_for_completion_with<F>(
        &self,
        task_id: impl AsRef<str>,
        max_wait: Duration,
        poll_interval: Duration,
        mut on_status_update: F,
    ) -> Result<SunoTask>
    where
        F: FnMut(Option<&str>),
    {
        let task_id = task_id.as_ref().trim().to_owned();
        let started = Instant::now();

        loop {
            let task = self.fetch_task(task_id.as_str())?;
            on_status_update(task.as_ref().and_then(|item| item.status.as_deref()));

            match task {
                Some(task) if matches!(task.status.as_deref(), Some("complete" | "streaming")) => {
                    return Ok(task);
                }
                Some(task) if matches!(task.status.as_deref(), Some("error")) => {
                    let detail = task
                        .error
                        .or(task.error_message)
                        .unwrap_or_else(|| "unknown error".to_owned());
                    bail!("invalid response: suno task failed: {detail}");
                }
                _ => {
                    if started.elapsed() >= max_wait {
                        bail!(
                            "invalid response: suno task `{task_id}` timed out after {max_wait:?}"
                        );
                    }
                    thread::sleep(poll_interval);
                }
            }
        }
    }

    /// 使用默认超时和轮询间隔等待一组任务全部完成。
    pub fn wait_for_batch_completion<I, S>(&self, task_ids: I) -> Result<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.wait_for_batch_completion_with(
            task_ids,
            Duration::from_secs(600),
            Duration::from_secs(10),
        )
    }

    /// 使用自定义超时和轮询间隔等待一组任务全部完成。
    ///
    /// 任一任务进入 `error` 状态会立即返回错误。
    pub fn wait_for_batch_completion_with<I, S>(
        &self,
        task_ids: I,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> Result<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let task_ids = task_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        let started = Instant::now();

        loop {
            let tasks = self.batch_fetch_tasks(task_ids.clone())?;
            if tasks
                .iter()
                .all(|item| matches!(item.status.as_deref(), Some("complete" | "streaming")))
            {
                return Ok(tasks);
            }
            if let Some(task) = tasks
                .iter()
                .find(|item| matches!(item.status.as_deref(), Some("error")))
            {
                let detail = task
                    .error
                    .clone()
                    .or(task.error_message.clone())
                    .unwrap_or_else(|| "unknown error".to_owned());
                bail!("invalid response: suno task failed: {detail}");
            }
            if started.elapsed() >= max_wait {
                bail!("invalid response: suno tasks timed out after {max_wait:?}");
            }
            thread::sleep(poll_interval);
        }
    }
}

/// Suno 音乐生成请求。
///
/// 字段名保持 Suno/VectorEngine HTTP wire contract，不在本地重命名为业务别名。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SunoMusicRequest {
    /// 模型版本，默认 `chirp-v5`。
    #[serde(default = "default_suno_mv")]
    pub mv: String,
    /// GPT 描述提示词。
    #[serde(rename = "gpt_description_prompt", default)]
    pub gpt_description_prompt: Option<String>,
    /// 任务完成回调地址。
    #[serde(rename = "notify_hook", default)]
    pub notify_hook: Option<String>,
    /// 歌曲标题。
    #[serde(default)]
    pub title: Option<String>,
    /// 风格标签。
    #[serde(default)]
    pub tags: Option<String>,
    /// 核心生成提示词。
    pub prompt: String,
    /// 是否生成纯伴奏。
    #[serde(rename = "make_instrumental", default)]
    pub make_instrumental: Option<bool>,
    /// 上游任务 id。
    #[serde(rename = "task_id", default)]
    pub task_id: Option<String>,
    /// 续写来源 clip id。
    #[serde(rename = "continue_clip_id", default)]
    pub continue_clip_id: Option<String>,
    /// 续写起始位置。
    #[serde(rename = "continue_at", default)]
    pub continue_at: Option<i32>,
    /// Persona id。
    #[serde(rename = "persona_id", default)]
    pub persona_id: Option<String>,
    /// 艺人风格来源 clip id。
    #[serde(rename = "artist_clip_id", default)]
    pub artist_clip_id: Option<String>,
    /// 人声性别。
    #[serde(rename = "vocal_gender", default)]
    pub vocal_gender: Option<String>,
    /// 生成类型。
    #[serde(rename = "generation_type", default)]
    pub generation_type: Option<String>,
    /// 负向标签。
    #[serde(rename = "negative_tags", default)]
    pub negative_tags: Option<String>,
    /// 相关 clip id。
    #[serde(rename = "clip_id", default)]
    pub clip_id: Option<String>,
    /// 是否执行 infill。
    #[serde(rename = "is_infill", default)]
    pub is_infill: Option<bool>,
    /// 上游任务模式，默认 `extend`。
    #[serde(default = "default_suno_task")]
    pub task: Option<String>,
}

/// Suno 歌词生成请求。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenerateLyricsRequest {
    /// 歌词提示词。
    pub prompt: String,
}

/// Suno 批量任务查询请求。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BatchFetchRequest {
    /// 任务 id 列表。
    #[serde(default)]
    pub ids: Vec<String>,
}

/// Suno 歌曲拼接请求。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConcatSongsRequest {
    /// 要拼接的 clip id。
    #[serde(rename = "clip_id")]
    pub clip_id: String,
}

/// Suno 任务状态。
///
/// 状态字符串直接保留上游值；等待逻辑把 `complete` / `streaming` 视为完成，把 `error` 视为失败。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SunoTask {
    /// 任务 id。
    #[serde(default)]
    pub id: Option<String>,
    /// 上游状态字符串。
    #[serde(default)]
    pub status: Option<String>,
    /// 原始提示词。
    #[serde(default)]
    pub prompt: Option<String>,
    /// GPT 描述提示词。
    #[serde(rename = "gpt_description_prompt", default)]
    pub gpt_description_prompt: Option<String>,
    /// 标题。
    #[serde(default)]
    pub title: Option<String>,
    /// 风格标签。
    #[serde(default)]
    pub tags: Option<String>,
    /// 模型版本。
    #[serde(default)]
    pub mv: Option<String>,
    /// 上游类型字段。
    #[serde(default)]
    pub r#type: Option<String>,
    /// 生成音频时长。
    #[serde(default)]
    pub duration: Option<f64>,
    /// 音频 URL。
    #[serde(rename = "audio_url", default)]
    pub audio_url: Option<String>,
    /// 视频 URL。
    #[serde(rename = "video_url", default)]
    pub video_url: Option<String>,
    /// 创建时间，保持上游字符串格式。
    #[serde(rename = "created_at", default)]
    pub created_at: Option<String>,
    /// 错误消息。
    #[serde(rename = "error_message", default)]
    pub error_message: Option<String>,
    /// 错误详情。
    #[serde(default)]
    pub error: Option<String>,
    /// clip id。
    #[serde(rename = "clip_id", default)]
    pub clip_id: Option<String>,
    /// 是否为纯伴奏。
    #[serde(rename = "instrumental", default)]
    pub instrumental: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiEnvelope<T> {
    #[serde(default)]
    code: Value,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    data: Option<T>,
}

impl<T> ApiEnvelope<T> {
    fn into_data(self, action: &str) -> Result<T> {
        if self.is_success() {
            return self.data.with_context(|| {
                format!("invalid response: {action} returned success without data")
            });
        }
        bail!(
            "invalid response: {action} failed: {}",
            self.message_or_default()
        )
    }

    fn into_optional_data(self, action: &str) -> Result<Option<T>> {
        if self.is_success() {
            return Ok(self.data);
        }
        bail!(
            "invalid response: {action} failed: {}",
            self.message_or_default()
        )
    }

    fn is_success(&self) -> bool {
        self.success.unwrap_or(false)
            || matches!(self.code, Value::Number(ref value) if value.as_i64() == Some(200))
            || matches!(self.code, Value::String(ref value) if value == "200" || value.eq_ignore_ascii_case("success"))
            || self.data.is_some() && self.code.is_null()
    }

    fn message_or_default(&self) -> String {
        self.message
            .clone()
            .or_else(|| self.msg.clone())
            .unwrap_or_else(|| format!("code={}", self.code))
    }
}

#[derive(Clone, Debug)]
struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    fn new(config: ApiConfig) -> Result<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .with_context(|| format!("invalid base url `{}`", config.base_url))?;
        let default_headers = build_header_map(&config.default_headers)?;

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .default_headers(default_headers);

        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            base_url,
            client: builder.build().context("build music HTTP client")?,
        })
    }

    fn get(&self, path: &str) -> Result<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    fn post(&self, path: &str) -> Result<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
    }

    fn with_bearer_auth(builder: RequestBuilder, bearer_token: Option<&str>) -> RequestBuilder {
        match trim_non_blank(bearer_token) {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    fn read_json<T: DeserializeOwned>(response: Response) -> Result<T> {
        let response = Self::ensure_success(response)?;
        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("read music API response body `{url}`"))?;
        serde_json::from_slice(bytes.as_ref())
            .with_context(|| format!("parse music API JSON response `{url}`"))
    }

    fn ensure_success(response: Response) -> Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = response
            .bytes()
            .with_context(|| format!("read failed music API response body `{url}`"))?;
        let body = String::from_utf8_lossy(body.as_ref()).into_owned();

        bail!(
            "request to `{url}` returned HTTP {}: {body}",
            status.as_u16()
        )
    }

    fn join_url(&self, path: &str) -> Result<Url> {
        self.base_url
            .join(path)
            .with_context(|| format!("invalid request path `{path}`"))
    }
}

fn build_header_map(headers: &BTreeMap<String, String>) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .with_context(|| format!("invalid header name `{name}`"))?;
        let header_value = HeaderValue::from_str(value)
            .with_context(|| format!("invalid header value for `{name}`"))?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}

fn ensure_code_200(code: i32, message: Option<&str>, action: &str) -> Result<()> {
    if code == 200 {
        return Ok(());
    }
    bail!(
        "invalid response: {action} failed: code={code}, message={}",
        message.unwrap_or_default()
    )
}

fn default_user_agent() -> String {
    format!("az-music/{}", env!("CARGO_PKG_VERSION"))
}

fn default_suno_mv() -> String {
    "chirp-v5".to_owned()
}

fn default_suno_task() -> Option<String> {
    Some("extend".to_owned())
}
