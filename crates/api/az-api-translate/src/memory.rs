//! MyMemory 免费翻译 API 客户端。
//!
//! [MyMemory](https://mymemory.translated.net/) 提供带日配额的免费翻译 API。
//! 该服务支持多种语言对，不要求 API key；传入有效邮箱可提高每日字符额度。

use reqwest::Client;

use crate::model::{DetectedLanguage, TranslateOptions, TranslateResult};
use crate::TranslateClient;
use anyhow::{Result, bail};

const BASE_URL: &str = "https://api.mymemory.translated.net/get";

/// MyMemory 翻译服务客户端。
///
/// 该客户端持有可注入的 [`reqwest::Client`]，便于调用方统一配置代理、超时和测试替身。
pub struct MyMemoryClient {
    client: Client,
    email: String,
}

impl MyMemoryClient {
    /// 使用默认 HTTP client 创建 MyMemory 客户端。
    ///
    /// 传入有效邮箱可提高 MyMemory 免费日配额。
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            email: email.into(),
        }
    }

    /// 使用外部提供的 [`reqwest::Client`] 创建 MyMemory 客户端。
    ///
    /// 这是翻译 provider 的依赖注入边界，适合复用全局连接池或在测试中注入 mock transport。
    pub fn with_client(client: Client, email: impl Into<String>) -> Self {
        Self {
            client,
            email: email.into(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct MyMemoryResponse {
    #[serde(rename = "responseData")]
    response_data: Option<MyMemoryResponseData>,
    #[serde(rename = "responseStatus")]
    response_status: u16,
    #[serde(rename = "responseDetails")]
    response_details: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct MyMemoryResponseData {
    #[serde(rename = "translatedText")]
    translated_text: String,
    #[serde(rename = "match")]
    match_score: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct MyMemoryMatchesResponse {
    #[serde(rename = "responseData")]
    response_data: Option<MyMemoryResponseData>,
    #[serde(rename = "matches")]
    matches: Option<Vec<MyMemoryMatch>>,
    #[serde(rename = "responseStatus")]
    response_status: u16,
}

#[derive(Debug, serde::Deserialize)]
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
    ) -> Result<TranslateResult> {
        self.translate_with_options(text, from, to, &TranslateOptions::default())
            .await
    }

    async fn translate_with_options(
        &self,
        text: &str,
        from: &str,
        to: &str,
        options: &TranslateOptions,
    ) -> Result<TranslateResult> {
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
            let status = resp.status();
            bail!("provider error: HTTP {status}");
        }

        if max_alts > 0 {
            let raw: MyMemoryMatchesResponse = resp.json().await?;

            if raw.response_status != 200 {
                let message = raw
                    .response_data
                    .map(|d| d.translated_text)
                    .unwrap_or_else(|| "unknown error".into());
                bail!("provider error: {message}");
            }

            let primary = raw
                .response_data
                .map(|d| d.translated_text)
                .unwrap_or_default();

            let alternatives: Vec<String> = raw
                .matches
                .unwrap_or_default()
                .into_iter()
                .skip(1) // 第一条 match 与主译文重复，这里只保留真正的候选项。
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
                let message = raw
                    .response_details
                    .or_else(|| raw.response_data.map(|d| d.translated_text))
                    .unwrap_or_else(|| "unknown error".into());
                bail!("provider error: {message}");
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

    async fn detect_language(&self, text: &str) -> Result<DetectedLanguage> {
        // MyMemory 没有专门的语言检测端点；当前保守返回 und，避免伪造检测结果。
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
        assert_eq!(resp.matches.as_ref().unwrap()[1].translation, "Hi");
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
