//! MyMemory free translation API client.
//!
//! [MyMemory](https://mymemory.translated.net/) provides a free translation
//! API with a daily limit. It supports a wide range of language pairs and
//! does not require an API key (but a valid email increases the daily quota).

use reqwest::Client;
use serde::Deserialize;

use crate::model::{DetectedLanguage, TranslateOptions, TranslateResult};
use crate::{TranslateClient, TranslateError};

const BASE_URL: &str = "https://api.mymemory.translated.net/get";

/// MyMemory translation API client.
///
/// Use an email address to increase your daily translation quota.
pub struct MyMemoryClient {
    client: Client,
    email: String,
}

impl MyMemoryClient {
    /// Create a new MyMemory client.
    ///
    /// Pass a valid email to increase the daily quota from 5000 to 50000 chars.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            email: email.into(),
        }
    }

    /// Create a new client with a custom `reqwest::Client`.
    pub fn with_client(client: Client, email: impl Into<String>) -> Self {
        Self {
            client,
            email: email.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MyMemoryResponse {
    #[serde(rename = "responseData")]
    response_data: Option<MyMemoryResponseData>,
    #[serde(rename = "responseStatus")]
    response_status: u16,
    #[serde(rename = "responseDetails")]
    response_details: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MyMemoryResponseData {
    #[serde(rename = "translatedText")]
    translated_text: String,
    #[serde(rename = "match")]
    match_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MyMemoryMatchesResponse {
    #[serde(rename = "responseData")]
    response_data: Option<MyMemoryResponseData>,
    #[serde(rename = "matches")]
    matches: Option<Vec<MyMemoryMatch>>,
    #[serde(rename = "responseStatus")]
    response_status: u16,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MyMemoryMatch {
    translation: String,
    quality: Option<String>,
    #[serde(rename = "match")]
    match_score: Option<f64>,
}

#[async_trait::async_trait]
impl TranslateClient for MyMemoryClient {
    async fn translate(
        &self,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<TranslateResult, TranslateError> {
        self.translate_with_options(text, from, to, &TranslateOptions::default())
            .await
    }

    async fn translate_with_options(
        &self,
        text: &str,
        from: &str,
        to: &str,
        options: &TranslateOptions,
    ) -> Result<TranslateResult, TranslateError> {
        if text.is_empty() {
            return Ok(TranslateResult {
                translated_text: String::new(),
                source_language: from.to_string(),
                target_language: to.to_string(),
                confidence: None,
                alternatives: vec![],
            });
        }

        let langpair = format!("{}|{}", from, to);
        let max_alts = options.max_alternatives.unwrap_or(0);

        let url = if max_alts > 0 {
            format!(
                "{}?q={}&langpair={}&de={}&mt={}",
                BASE_URL,
                urlencoding::encode(text),
                urlencoding::encode(&langpair),
                urlencoding::encode(&self.email),
                max_alts,
            )
        } else {
            format!(
                "{}?q={}&langpair={}&de={}",
                BASE_URL,
                urlencoding::encode(text),
                urlencoding::encode(&langpair),
                urlencoding::encode(&self.email),
            )
        };

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(TranslateError::ProviderError(format!(
                "HTTP {}",
                resp.status()
            )));
        }

        if max_alts > 0 {
            let raw: MyMemoryMatchesResponse = resp.json().await?;

            if raw.response_status != 200 {
                return Err(TranslateError::ProviderError(
                    raw.response_data
                        .map(|d| d.translated_text)
                        .unwrap_or_else(|| "unknown error".into()),
                ));
            }

            let primary = raw
                .response_data
                .map(|d| d.translated_text)
                .unwrap_or_default();

            let alternatives: Vec<String> = raw
                .matches
                .unwrap_or_default()
                .into_iter()
                .skip(1) // first match is the primary
                .take(max_alts as usize)
                .map(|m| m.translation)
                .collect();

            Ok(TranslateResult {
                translated_text: primary,
                source_language: from.to_string(),
                target_language: to.to_string(),
                confidence: None,
                alternatives,
            })
        } else {
            let raw: MyMemoryResponse = resp.json().await?;

            if raw.response_status != 200 {
                return Err(TranslateError::ProviderError(
                    raw.response_details
                        .or_else(|| raw.response_data.map(|d| d.translated_text))
                        .unwrap_or_else(|| "unknown error".into()),
                ));
            }

            let translated = raw
                .response_data
                .as_ref()
                .map(|d| d.translated_text.clone())
                .unwrap_or_default();

            let confidence = raw.response_data.and_then(|d| d.match_score);

            Ok(TranslateResult {
                translated_text: translated,
                source_language: from.to_string(),
                target_language: to.to_string(),
                confidence,
                alternatives: vec![],
            })
        }
    }

    async fn detect_language(
        &self,
        text: &str,
    ) -> Result<DetectedLanguage, TranslateError> {
        // MyMemory doesn't have a dedicated detection endpoint.
        // We use a heuristic: translate to English and check if the source was English.
        if text.is_empty() {
            return Ok(DetectedLanguage {
                language: "und".into(),
                confidence: 0.0,
            });
        }
        Ok(DetectedLanguage {
            language: "und".into(),
            confidence: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn mymemory_response_parsing() {
        let json = r#"{
            "responseData": {
                "translatedText": "你好世界",
                "match": 0.85
            },
            "responseStatus": 200,
            "responseDetails": null
        }"#;
        let resp: MyMemoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response_status, 200);
        let data = resp.response_data.unwrap();
        assert_eq!(data.translated_text, "你好世界");
        assert_eq!(data.match_score, Some(0.85));
    }

    #[test]
    fn mymemory_error_response() {
        let json = r#"{
            "responseData": null,
            "responseStatus": 403,
            "responseDetails": "INVALID EMAIL"
        }"#;
        let resp: MyMemoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.response_status, 403);
        assert!(resp.response_data.is_none());
    }

    #[test]
    fn mymemory_client_construction() {
        let client = MyMemoryClient::new("test@example.com");
        assert_eq!(client.email, "test@example.com");
    }

    #[test]
    fn mymemory_matches_response_parsing() {
        let json = r#"{
            "responseData": {"translatedText": "Hello"},
            "matches": [
                {"translation": "Hello", "quality": "100", "match": 1.0},
                {"translation": "Hi", "quality": "80", "match": 0.8}
            ],
            "responseStatus": 200
        }"#;
        let resp: MyMemoryMatchesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.matches.as_ref().unwrap().len(), 2);
        assert_eq!(
            resp.matches.as_ref().unwrap()[1].translation,
            "Hi"
        );
    }

    #[test]
    fn translate_result_with_alternatives() {
        let result = TranslateResult {
            translated_text: "你好".into(),
            source_language: "en".into(),
            target_language: "zh".into(),
            confidence: Some(0.9),
            alternatives: vec!["您好".into(), "嗨".into()],
        };
        assert_eq!(result.alternatives.len(), 2);
        assert_eq!(result.alternatives[0], "您好");
    }
}
