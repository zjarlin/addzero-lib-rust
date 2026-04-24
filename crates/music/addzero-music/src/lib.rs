#![forbid(unsafe_code)]

use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{
    ACCEPT, HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::thread;
use std::time::{Duration, SystemTime};
use thiserror::Error;

pub type MusicResult<T> = Result<T, MusicError>;

#[derive(Debug, Error)]
pub enum MusicError {
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("invalid base url `{0}`")]
    InvalidBaseUrl(String),
    #[error("invalid request path `{0}`")]
    InvalidPath(String),
    #[error("invalid header name `{name}`: {source}")]
    InvalidHeaderName {
        name: String,
        #[source]
        source: InvalidHeaderName,
    },
    #[error("invalid header value for `{name}`: {source}")]
    InvalidHeaderValue {
        name: String,
        #[source]
        source: InvalidHeaderValue,
    },
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("failed to parse json payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("request to `{url}` returned HTTP {status}: {body}")]
    HttpStatus {
        url: String,
        status: u16,
        body: String,
    },
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    pub base_url: String,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub user_agent: Option<String>,
    pub default_headers: BTreeMap<String, String>,
}

impl ApiConfig {
    pub fn builder(base_url: impl Into<String>) -> ApiConfigBuilder {
        ApiConfigBuilder {
            base_url: base_url.into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(20),
            user_agent: Some(default_user_agent()),
            default_headers: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> MusicResult<()> {
        if self.base_url.trim().is_empty() {
            return Err(MusicError::InvalidConfig(
                "base_url cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(MusicError::InvalidConfig(
                "connect_timeout cannot be zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(MusicError::InvalidConfig(
                "request_timeout cannot be zero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ApiConfigBuilder {
    base_url: String,
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
    default_headers: BTreeMap<String, String>,
}

impl ApiConfigBuilder {
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn clear_user_agent(mut self) -> Self {
        self.user_agent = None;
        self
    }

    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    pub fn build(self) -> MusicResult<ApiConfig> {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Music;

impl Music {
    pub fn netease() -> MusicResult<MusicSearchApi> {
        create_netease_api()
    }

    pub fn netease_with_config(config: ApiConfig) -> MusicResult<MusicSearchApi> {
        MusicSearchApi::new(config)
    }

    pub fn suno(api_token: impl Into<String>) -> MusicResult<SunoApi> {
        create_suno_api(api_token)
    }

    pub fn suno_with_config(
        api_token: impl Into<String>,
        config: ApiConfig,
    ) -> MusicResult<SunoApi> {
        SunoApi::new(api_token, config)
    }
}

pub fn create_netease_api() -> MusicResult<MusicSearchApi> {
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

pub fn create_suno_api(api_token: impl Into<String>) -> MusicResult<SunoApi> {
    let config = ApiConfig::builder("https://api.vectorengine.ai")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    SunoApi::new(api_token, config)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicSearchType {
    #[default]
    Song,
    Album,
    Artist,
    Playlist,
    User,
    Mv,
    Lyric,
    Radio,
    Video,
}

impl MusicSearchType {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicSearchRequest {
    pub keywords: String,
    pub search_type: MusicSearchType,
    pub limit: usize,
    pub offset: usize,
}

impl MusicSearchRequest {
    pub fn new(keywords: impl Into<String>) -> Self {
        Self {
            keywords: keywords.into(),
            search_type: MusicSearchType::Song,
            limit: 30,
            offset: 0,
        }
    }

    pub fn search_type(mut self, value: MusicSearchType) -> Self {
        self.search_type = value;
        self
    }

    pub fn limit(mut self, value: usize) -> Self {
        self.limit = value;
        self
    }

    pub fn offset(mut self, value: usize) -> Self {
        self.offset = value;
        self
    }
}

#[derive(Debug, Clone)]
pub struct MusicSearchApi {
    http: HttpApiClient,
}

pub type NeteaseMusicApi = MusicSearchApi;

impl MusicSearchApi {
    pub fn new(config: ApiConfig) -> MusicResult<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search(&self, request: MusicSearchRequest) -> MusicResult<MusicSearchResult> {
        let keywords = trim_non_blank(Some(request.keywords.as_str())).ok_or_else(|| {
            MusicError::InvalidConfig("music keywords cannot be blank".to_owned())
        })?;
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

    pub fn search_songs(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> MusicResult<Vec<MusicSong>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Song)
                    .limit(limit)
                    .offset(offset),
            )?
            .songs)
    }

    pub fn search_artists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> MusicResult<Vec<MusicArtist>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Artist)
                    .limit(limit)
                    .offset(offset),
            )?
            .artists)
    }

    pub fn search_albums(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> MusicResult<Vec<MusicAlbum>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Album)
                    .limit(limit)
                    .offset(offset),
            )?
            .albums)
    }

    pub fn search_playlists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> MusicResult<Vec<MusicPlaylist>> {
        Ok(self
            .search(
                MusicSearchRequest::new(keywords)
                    .search_type(MusicSearchType::Playlist)
                    .limit(limit)
                    .offset(offset),
            )?
            .playlists)
    }

    pub fn get_lyric(&self, song_id: i64) -> MusicResult<LyricResponse> {
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

    pub fn get_song_detail(&self, song_ids: &[i64]) -> MusicResult<Vec<MusicSong>> {
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

    pub fn search_by_song_and_artist(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> MusicResult<Vec<MusicSong>> {
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

    pub fn search_by_lyric(
        &self,
        lyric_fragment: impl Into<String>,
    ) -> MusicResult<Vec<MusicSong>> {
        Ok(self
            .search(
                MusicSearchRequest::new(lyric_fragment)
                    .search_type(MusicSearchType::Lyric)
                    .limit(20),
            )?
            .songs)
    }

    pub fn get_lyric_by_song_name(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> MusicResult<Option<LyricResponse>> {
        let songs = self.search_by_song_and_artist(song_name, artist_name)?;
        if let Some(song) = songs.first() {
            return self.get_lyric(song.id).map(Some);
        }
        Ok(None)
    }

    pub fn get_lyrics_by_fragment(
        &self,
        lyric_fragment: impl Into<String>,
        limit: usize,
        filter_empty: bool,
    ) -> MusicResult<Vec<SongWithLyric>> {
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicSearchResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub result: Option<MusicSearchResult>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicSearchResult {
    #[serde(default)]
    pub songs: Vec<MusicSong>,
    #[serde(rename = "songCount", default)]
    pub song_count: Option<i64>,
    #[serde(default)]
    pub albums: Vec<MusicAlbum>,
    #[serde(rename = "albumCount", default)]
    pub album_count: Option<i64>,
    #[serde(default)]
    pub artists: Vec<MusicArtist>,
    #[serde(rename = "artistCount", default)]
    pub artist_count: Option<i64>,
    #[serde(default)]
    pub playlists: Vec<MusicPlaylist>,
    #[serde(rename = "playlistCount", default)]
    pub playlist_count: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicSong {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "ar")]
    pub artists: Vec<MusicArtist>,
    #[serde(default, alias = "al")]
    pub album: Option<MusicAlbum>,
    #[serde(default, alias = "dt")]
    pub duration: Option<i64>,
    #[serde(rename = "mvid", default)]
    pub mv_id: Option<i64>,
    #[serde(default)]
    pub fee: Option<i32>,
    #[serde(default)]
    pub privilege: Option<MusicPrivilege>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicArtist {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    #[serde(default)]
    pub alias: Vec<String>,
    #[serde(rename = "albumSize", default)]
    pub album_size: Option<i32>,
    #[serde(rename = "musicSize", default)]
    pub music_size: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicAlbum {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "picUrl", default)]
    pub pic_url: Option<String>,
    #[serde(default)]
    pub artist: Option<MusicArtist>,
    #[serde(rename = "publishTime", default)]
    pub publish_time: Option<i64>,
    #[serde(default)]
    pub size: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicPlaylist {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "coverImgUrl", default)]
    pub cover_img_url: Option<String>,
    #[serde(default)]
    pub creator: Option<MusicCreator>,
    #[serde(rename = "trackCount", default)]
    pub track_count: Option<i32>,
    #[serde(rename = "playCount", default)]
    pub play_count: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicCreator {
    #[serde(rename = "userId", default)]
    pub user_id: i64,
    #[serde(default)]
    pub nickname: String,
    #[serde(rename = "avatarUrl", default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicPrivilege {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub fee: Option<i32>,
    #[serde(default)]
    pub st: Option<i32>,
    #[serde(default)]
    pub pl: Option<i32>,
    #[serde(default)]
    pub dl: Option<i32>,
    #[serde(default)]
    pub maxbr: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub lrc: Option<LyricContent>,
    #[serde(default)]
    pub tlyric: Option<LyricContent>,
    #[serde(default)]
    pub romalrc: Option<LyricContent>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LyricContent {
    #[serde(default)]
    pub version: Option<i32>,
    #[serde(default)]
    pub lyric: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SongDetailResponse {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub songs: Vec<MusicSong>,
    #[serde(default)]
    pub privileges: Vec<MusicPrivilege>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SongWithLyric {
    pub song: MusicSong,
    pub lyric: LyricResponse,
}

#[derive(Debug, Clone)]
pub struct SunoApi {
    api_token: String,
    http: HttpApiClient,
}

impl SunoApi {
    pub fn new(api_token: impl Into<String>, config: ApiConfig) -> MusicResult<Self> {
        let api_token = api_token.into();
        if trim_non_blank(Some(api_token.as_str())).is_none() {
            return Err(MusicError::InvalidConfig(
                "suno api_token cannot be blank".to_owned(),
            ));
        }
        Ok(Self {
            api_token,
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn generate_music(&self, request: &SunoMusicRequest) -> MusicResult<String> {
        let response = HttpApiClient::with_bearer_auth(
            self.http.post("suno/submit/music")?,
            Some(self.api_token.as_str()),
        )
        .json(request)
        .send()?;
        let response: ApiEnvelope<String> = HttpApiClient::read_json(response)?;
        response.into_data("generate suno music")
    }

    pub fn generate_lyrics(&self, prompt: impl AsRef<str>) -> MusicResult<String> {
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

    pub fn concat_songs(&self, clip_id: impl AsRef<str>) -> MusicResult<String> {
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

    pub fn fetch_task(&self, task_id: impl AsRef<str>) -> MusicResult<Option<SunoTask>> {
        let path = format!("suno/fetch/{}", task_id.as_ref().trim());
        let response =
            HttpApiClient::with_bearer_auth(self.http.get(&path)?, Some(self.api_token.as_str()))
                .send()?;
        let response: ApiEnvelope<SunoTask> = HttpApiClient::read_json(response)?;
        response.into_optional_data("fetch suno task")
    }

    pub fn batch_fetch_tasks<I, S>(&self, task_ids: I) -> MusicResult<Vec<SunoTask>>
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

    pub fn wait_for_completion(&self, task_id: impl AsRef<str>) -> MusicResult<SunoTask> {
        self.wait_for_completion_with(
            task_id,
            Duration::from_secs(600),
            Duration::from_secs(10),
            |_| {},
        )
    }

    pub fn wait_for_completion_with<F>(
        &self,
        task_id: impl AsRef<str>,
        max_wait: Duration,
        poll_interval: Duration,
        mut on_status_update: F,
    ) -> MusicResult<SunoTask>
    where
        F: FnMut(Option<&str>),
    {
        let task_id = task_id.as_ref().trim().to_owned();
        let started = SystemTime::now();

        loop {
            let task = self.fetch_task(task_id.as_str())?;
            on_status_update(task.as_ref().and_then(|item| item.status.as_deref()));

            match task {
                Some(task) if matches!(task.status.as_deref(), Some("complete" | "streaming")) => {
                    return Ok(task);
                }
                Some(task) if matches!(task.status.as_deref(), Some("error")) => {
                    return Err(MusicError::InvalidResponse(format!(
                        "suno task failed: {}",
                        task.error
                            .or(task.error_message)
                            .unwrap_or_else(|| "unknown error".to_owned())
                    )));
                }
                _ => {
                    if started.elapsed().unwrap_or_default() >= max_wait {
                        return Err(MusicError::InvalidResponse(format!(
                            "suno task `{task_id}` timed out after {:?}",
                            max_wait
                        )));
                    }
                    thread::sleep(poll_interval);
                }
            }
        }
    }

    pub fn wait_for_batch_completion<I, S>(&self, task_ids: I) -> MusicResult<Vec<SunoTask>>
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

    pub fn wait_for_batch_completion_with<I, S>(
        &self,
        task_ids: I,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> MusicResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let task_ids = task_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        let started = SystemTime::now();

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
                return Err(MusicError::InvalidResponse(format!(
                    "suno task failed: {}",
                    task.error
                        .clone()
                        .or(task.error_message.clone())
                        .unwrap_or_else(|| "unknown error".to_owned())
                )));
            }
            if started.elapsed().unwrap_or_default() >= max_wait {
                return Err(MusicError::InvalidResponse(format!(
                    "suno tasks timed out after {:?}",
                    max_wait
                )));
            }
            thread::sleep(poll_interval);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SunoMusicRequest {
    #[serde(default = "default_suno_mv")]
    pub mv: String,
    #[serde(rename = "gpt_description_prompt", default)]
    pub gpt_description_prompt: Option<String>,
    #[serde(rename = "notify_hook", default)]
    pub notify_hook: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    pub prompt: String,
    #[serde(rename = "make_instrumental", default)]
    pub make_instrumental: Option<bool>,
    #[serde(rename = "task_id", default)]
    pub task_id: Option<String>,
    #[serde(rename = "continue_clip_id", default)]
    pub continue_clip_id: Option<String>,
    #[serde(rename = "continue_at", default)]
    pub continue_at: Option<i32>,
    #[serde(rename = "persona_id", default)]
    pub persona_id: Option<String>,
    #[serde(rename = "artist_clip_id", default)]
    pub artist_clip_id: Option<String>,
    #[serde(rename = "vocal_gender", default)]
    pub vocal_gender: Option<String>,
    #[serde(rename = "generation_type", default)]
    pub generation_type: Option<String>,
    #[serde(rename = "negative_tags", default)]
    pub negative_tags: Option<String>,
    #[serde(rename = "clip_id", default)]
    pub clip_id: Option<String>,
    #[serde(rename = "is_infill", default)]
    pub is_infill: Option<bool>,
    #[serde(default = "default_suno_task")]
    pub task: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateLyricsRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchFetchRequest {
    #[serde(default)]
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcatSongsRequest {
    #[serde(rename = "clip_id")]
    pub clip_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SunoTask {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(rename = "gpt_description_prompt", default)]
    pub gpt_description_prompt: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub mv: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(rename = "audio_url", default)]
    pub audio_url: Option<String>,
    #[serde(rename = "video_url", default)]
    pub video_url: Option<String>,
    #[serde(rename = "created_at", default)]
    pub created_at: Option<String>,
    #[serde(rename = "error_message", default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(rename = "clip_id", default)]
    pub clip_id: Option<String>,
    #[serde(rename = "instrumental", default)]
    pub instrumental: Option<bool>,
}

#[derive(Debug, Deserialize)]
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
    fn into_data(self, action: &str) -> MusicResult<T> {
        if self.is_success() {
            return self.data.ok_or_else(|| {
                MusicError::InvalidResponse(format!("{action} returned success without data"))
            });
        }
        Err(MusicError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message_or_default()
        )))
    }

    fn into_optional_data(self, action: &str) -> MusicResult<Option<T>> {
        if self.is_success() {
            return Ok(self.data);
        }
        Err(MusicError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message_or_default()
        )))
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

#[derive(Debug, Clone)]
struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    fn new(config: ApiConfig) -> MusicResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| MusicError::InvalidBaseUrl(config.base_url))?;
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
            client: builder.build()?,
        })
    }

    fn get(&self, path: &str) -> MusicResult<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    fn post(&self, path: &str) -> MusicResult<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
    }

    fn with_bearer_auth(builder: RequestBuilder, bearer_token: Option<&str>) -> RequestBuilder {
        match trim_non_blank(bearer_token) {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    fn read_json<T: DeserializeOwned>(response: Response) -> MusicResult<T> {
        let response = Self::ensure_success(response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    fn ensure_success(response: Response) -> MusicResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(MusicError::Transport(error)),
        };

        Err(MusicError::HttpStatus {
            url,
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> MusicResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| MusicError::InvalidPath(path.to_owned()))
    }
}

fn build_header_map(headers: &BTreeMap<String, String>) -> MusicResult<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|source| {
            MusicError::InvalidHeaderName {
                name: name.clone(),
                source,
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|source| MusicError::InvalidHeaderValue {
                name: name.clone(),
                source,
            })?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}

fn trim_non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn ensure_code_200(code: i32, message: Option<&str>, action: &str) -> MusicResult<()> {
    if code == 200 {
        return Ok(());
    }
    Err(MusicError::InvalidResponse(format!(
        "{action} failed: code={code}, message={}",
        message.unwrap_or_default()
    )))
}

fn default_user_agent() -> String {
    format!("addzero-music/{}", env!("CARGO_PKG_VERSION"))
}

fn default_suno_mv() -> String {
    "chirp-v5".to_owned()
}

fn default_suno_task() -> Option<String> {
    Some("extend".to_owned())
}
