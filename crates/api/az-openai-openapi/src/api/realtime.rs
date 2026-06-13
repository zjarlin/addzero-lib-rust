// Generated from OpenAPI spec. Do not edit by hand.
//! Realtime REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::{
    OpenAiTextBody,
};

use crate::models::{
    RealtimeCallCreateRequest,
    RealtimeCallReferRequest,
    RealtimeCallRejectRequest,
    RealtimeCreateClientSecretRequest,
    RealtimeCreateClientSecretResponse,
    RealtimeSessionCreateRequest,
    RealtimeSessionCreateRequestGA,
    RealtimeSessionCreateResponse,
    RealtimeTranscriptionSessionCreateRequest,
    RealtimeTranscriptionSessionCreateResponse,
    RealtimeTranslationClientSecretCreateRequest,
    RealtimeTranslationClientSecretCreateResponse,
};

/// Realtime REST endpoints.
#[async_trait]
pub trait OpenAiRealtimeApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new Realtime API call over WebRTC and receive the SDP answer needed to complete the peer
    /// connection.
    ///
    /// REST: `POST /realtime/calls`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_CALLS`](crate::paths::OpenAiApiPath::REALTIME_BY_CALLS).
    async fn create_realtime_call(
        &self,
        body: RealtimeCallCreateRequest,
    ) -> Result<OpenAiTextBody, Self::Error>;

    /// Accept an incoming SIP call and configure the realtime session that will handle it.
    ///
    /// REST: `POST /realtime/calls/{call_id}/accept`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_ACCEPT`](crate::paths::OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_ACCEPT).
    async fn accept_realtime_call(
        &self,
        call_id: String,
        body: RealtimeSessionCreateRequestGA,
    ) -> Result<(), Self::Error>;

    /// End an active Realtime API call, whether it was initiated over SIP or WebRTC.
    ///
    /// REST: `POST /realtime/calls/{call_id}/hangup`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_HANGUP`](crate::paths::OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_HANGUP).
    async fn hangup_realtime_call(&self, call_id: String) -> Result<(), Self::Error>;

    /// Transfer an active SIP call to a new destination using the SIP REFER verb.
    ///
    /// REST: `POST /realtime/calls/{call_id}/refer`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_REFER`](crate::paths::OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_REFER).
    async fn refer_realtime_call(
        &self,
        call_id: String,
        body: RealtimeCallReferRequest,
    ) -> Result<(), Self::Error>;

    /// Decline an incoming SIP call by returning a SIP status code to the caller.
    ///
    /// REST: `POST /realtime/calls/{call_id}/reject`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_REJECT`](crate::paths::OpenAiApiPath::REALTIME_BY_CALLS_BY_CALL_ID_BY_REJECT).
    async fn reject_realtime_call(
        &self,
        call_id: String,
        body: Option<RealtimeCallRejectRequest>,
    ) -> Result<(), Self::Error>;

    /// Create a Realtime client secret with an associated session configuration. Client secrets are short-
    /// lived tokens that can be passed to a client app, such as a web frontend or mobile client, which
    /// grants access to the Realtime API without leaking your main API key. You can configure a custom TTL
    /// for each client secret. You can also attach session configuration options to the client secret,
    /// which will be applied to any sessions created using that client secret, but these can also be
    /// overridden by the client connection. [Learn more about authentication with client secrets over
    /// WebRTC](/docs/guides/realtime-webrtc). Returns the created client secret and the effective session
    /// object. The client secret is a string that looks like `ek_1234`.
    ///
    /// REST: `POST /realtime/client_secrets`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_CLIENT_SECRETS`](crate::paths::OpenAiApiPath::REALTIME_BY_CLIENT_SECRETS).
    async fn create_realtime_client_secret(
        &self,
        body: RealtimeCreateClientSecretRequest,
    ) -> Result<RealtimeCreateClientSecretResponse, Self::Error>;

    /// Create an ephemeral API token for use in client-side applications with the Realtime API. Can be
    /// configured with the same session parameters as the `session.update` client event. It responds with a
    /// session object, plus a `client_secret` key which contains a usable ephemeral API token that can be
    /// used to authenticate browser clients for the Realtime API. Returns the created Realtime session
    /// object, plus an ephemeral key.
    ///
    /// REST: `POST /realtime/sessions`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_SESSIONS`](crate::paths::OpenAiApiPath::REALTIME_BY_SESSIONS).
    async fn create_realtime_session(
        &self,
        body: RealtimeSessionCreateRequest,
    ) -> Result<RealtimeSessionCreateResponse, Self::Error>;

    /// Create an ephemeral API token for use in client-side applications with the Realtime API specifically
    /// for realtime transcriptions. Can be configured with the same session parameters as the
    /// `transcription_session.update` client event. It responds with a session object, plus a
    /// `client_secret` key which contains a usable ephemeral API token that can be used to authenticate
    /// browser clients for the Realtime API. Returns the created Realtime transcription session object,
    /// plus an ephemeral key.
    ///
    /// REST: `POST /realtime/transcription_sessions`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_TRANSCRIPTION_SESSIONS`](crate::paths::OpenAiApiPath::REALTIME_BY_TRANSCRIPTION_SESSIONS).
    async fn create_realtime_transcription_session(
        &self,
        body: RealtimeTranscriptionSessionCreateRequest,
    ) -> Result<RealtimeTranscriptionSessionCreateResponse, Self::Error>;

    /// Create a Realtime translation client secret with an associated translation session configuration.
    /// Client secrets are short-lived tokens that can be passed to a client app, such as a web frontend or
    /// mobile client, which grants access to the Realtime Translation API without leaking your main API
    /// key. You can configure a custom TTL for each client secret. Returns the created client secret and
    /// the effective translation session object. The client secret is a string that looks like `ek_1234`.
    ///
    /// REST: `POST /realtime/translations/client_secrets`.
    /// Path constant: [`OpenAiApiPath::REALTIME_BY_TRANSLATIONS_BY_CLIENT_SECRETS`](crate::paths::OpenAiApiPath::REALTIME_BY_TRANSLATIONS_BY_CLIENT_SECRETS).
    async fn create_realtime_translation_client_secret(
        &self,
        body: RealtimeTranslationClientSecretCreateRequest,
    ) -> Result<RealtimeTranslationClientSecretCreateResponse, Self::Error>;
}
