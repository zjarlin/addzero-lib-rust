//! Audio REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Audio REST endpoints.
#[async_trait]
pub trait OpenAiAudioApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Generates audio from the input text. Returns the audio file content, or a stream of audio events.
    ///
    /// REST: `POST /audio/speech`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_SPEECH`](crate::paths::OpenAiApiPath::AUDIO_BY_SPEECH).
    async fn create_speech(&self, body: OpenAiRequestBody)
    -> Result<OpenAiBinaryBody, Self::Error>;

    /// Transcribes audio into the input language. Returns a transcription object in `json`, `diarized_json`, or `verbose_json` format, or a stream of transcript events.
    ///
    /// REST: `POST /audio/transcriptions`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_TRANSCRIPTIONS`](crate::paths::OpenAiApiPath::AUDIO_BY_TRANSCRIPTIONS).
    async fn create_transcription(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Translates audio into English.
    ///
    /// REST: `POST /audio/translations`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_TRANSLATIONS`](crate::paths::OpenAiApiPath::AUDIO_BY_TRANSLATIONS).
    async fn create_translation(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Returns a list of voice consent recordings.
    ///
    /// REST: `GET /audio/voice_consents`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS`](crate::paths::OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS).
    async fn list_voice_consents(
        &self,
        after: Option<String>,
        limit: Option<i64>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Upload a voice consent recording.
    ///
    /// REST: `POST /audio/voice_consents`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS`](crate::paths::OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS).
    async fn create_voice_consent(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Deletes a voice consent recording.
    ///
    /// REST: `DELETE /audio/voice_consents/{consent_id}`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID`](crate::paths::OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID).
    async fn delete_voice_consent(
        &self,
        consent_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Retrieves a voice consent recording.
    ///
    /// REST: `GET /audio/voice_consents/{consent_id}`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID`](crate::paths::OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID).
    async fn get_voice_consent(
        &self,
        consent_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Updates a voice consent recording (metadata only).
    ///
    /// REST: `POST /audio/voice_consents/{consent_id}`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID`](crate::paths::OpenAiApiPath::AUDIO_BY_VOICE_CONSENTS_BY_CONSENT_ID).
    async fn update_voice_consent(
        &self,
        consent_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates a custom voice.
    ///
    /// REST: `POST /audio/voices`.
    /// Path constant: [`OpenAiApiPath::AUDIO_BY_VOICES`](crate::paths::OpenAiApiPath::AUDIO_BY_VOICES).
    async fn create_voice(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
