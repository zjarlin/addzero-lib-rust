use addzero_music::{MusicSearchApi as InnerMusicSearchApi, SunoApi as InnerSunoApi};
use crate::{ApiConfig, CreatesResult};
use std::time::Duration;

pub use addzero_music::{
    BatchFetchRequest, ConcatSongsRequest, GenerateLyricsRequest, LyricContent, LyricResponse,
    MusicAlbum, MusicArtist, MusicCreator, MusicPlaylist, MusicPrivilege, MusicSearchRequest,
    MusicSearchResponse, MusicSearchResult, MusicSearchType, MusicSong, SongDetailResponse,
    SongWithLyric, SunoMusicRequest, SunoTask,
};

#[derive(Debug, Clone)]
pub struct MusicSearchApi {
    inner: InnerMusicSearchApi,
}

pub type NeteaseMusicApi = MusicSearchApi;

impl MusicSearchApi {
    pub fn new(config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            inner: InnerMusicSearchApi::new(config.into_music_config())?,
        })
    }

    pub fn search(&self, request: MusicSearchRequest) -> CreatesResult<MusicSearchResult> {
        self.inner.search(request).map_err(Into::into)
    }

    pub fn search_songs(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicSong>> {
        self.inner
            .search_songs(keywords, limit, offset)
            .map_err(Into::into)
    }

    pub fn search_artists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicArtist>> {
        self.inner
            .search_artists(keywords, limit, offset)
            .map_err(Into::into)
    }

    pub fn search_albums(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicAlbum>> {
        self.inner
            .search_albums(keywords, limit, offset)
            .map_err(Into::into)
    }

    pub fn search_playlists(
        &self,
        keywords: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> CreatesResult<Vec<MusicPlaylist>> {
        self.inner
            .search_playlists(keywords, limit, offset)
            .map_err(Into::into)
    }

    pub fn get_lyric(&self, song_id: i64) -> CreatesResult<LyricResponse> {
        self.inner.get_lyric(song_id).map_err(Into::into)
    }

    pub fn get_song_detail(&self, song_ids: &[i64]) -> CreatesResult<Vec<MusicSong>> {
        self.inner.get_song_detail(song_ids).map_err(Into::into)
    }

    pub fn search_by_song_and_artist(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> CreatesResult<Vec<MusicSong>> {
        self.inner
            .search_by_song_and_artist(song_name, artist_name)
            .map_err(Into::into)
    }

    pub fn search_by_lyric(
        &self,
        lyric_fragment: impl Into<String>,
    ) -> CreatesResult<Vec<MusicSong>> {
        self.inner.search_by_lyric(lyric_fragment).map_err(Into::into)
    }

    pub fn get_lyric_by_song_name(
        &self,
        song_name: impl AsRef<str>,
        artist_name: Option<&str>,
    ) -> CreatesResult<Option<LyricResponse>> {
        self.inner
            .get_lyric_by_song_name(song_name, artist_name)
            .map_err(Into::into)
    }

    pub fn get_lyrics_by_fragment(
        &self,
        lyric_fragment: impl Into<String>,
        limit: usize,
        filter_empty: bool,
    ) -> CreatesResult<Vec<SongWithLyric>> {
        self.inner
            .get_lyrics_by_fragment(lyric_fragment, limit, filter_empty)
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct SunoApi {
    inner: InnerSunoApi,
}

impl SunoApi {
    pub fn new(api_token: impl Into<String>, config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            inner: InnerSunoApi::new(api_token, config.into_music_config())?,
        })
    }

    pub fn generate_music(&self, request: &SunoMusicRequest) -> CreatesResult<String> {
        self.inner.generate_music(request).map_err(Into::into)
    }

    pub fn generate_lyrics(&self, prompt: impl AsRef<str>) -> CreatesResult<String> {
        self.inner.generate_lyrics(prompt).map_err(Into::into)
    }

    pub fn concat_songs(&self, clip_id: impl AsRef<str>) -> CreatesResult<String> {
        self.inner.concat_songs(clip_id).map_err(Into::into)
    }

    pub fn fetch_task(&self, task_id: impl AsRef<str>) -> CreatesResult<Option<SunoTask>> {
        self.inner.fetch_task(task_id).map_err(Into::into)
    }

    pub fn batch_fetch_tasks<I, S>(&self, task_ids: I) -> CreatesResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner.batch_fetch_tasks(task_ids).map_err(Into::into)
    }

    pub fn wait_for_completion(&self, task_id: impl AsRef<str>) -> CreatesResult<SunoTask> {
        self.inner.wait_for_completion(task_id).map_err(Into::into)
    }

    pub fn wait_for_completion_with<F>(
        &self,
        task_id: impl AsRef<str>,
        max_wait: Duration,
        poll_interval: Duration,
        on_status_update: F,
    ) -> CreatesResult<SunoTask>
    where
        F: FnMut(Option<&str>),
    {
        self.inner
            .wait_for_completion_with(task_id, max_wait, poll_interval, on_status_update)
            .map_err(Into::into)
    }

    pub fn wait_for_batch_completion<I, S>(&self, task_ids: I) -> CreatesResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner
            .wait_for_batch_completion(task_ids)
            .map_err(Into::into)
    }

    pub fn wait_for_batch_completion_with<I, S>(
        &self,
        task_ids: I,
        max_wait: Duration,
        poll_interval: Duration,
    ) -> CreatesResult<Vec<SunoTask>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner
            .wait_for_batch_completion_with(task_ids, max_wait, poll_interval)
            .map_err(Into::into)
    }
}
