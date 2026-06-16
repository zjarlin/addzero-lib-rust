//! 视频算法告警动作规划。

use std::collections::BTreeMap;

use serde_json::json;

use crate::video_pipeline::model::VideoAlgorithmFrameResult;

/// 告警输出通道。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlarmOutputTarget {
    /// 发布 MQTT 消息。
    Mqtt {
        /// MQTT topic。
        topic: String,
        /// QoS 等级，取值由外层 MQTT 执行器解释。
        qos: u8,
    },
    /// 发送 HTTP POST。
    HttpPost {
        /// HTTP URL。
        url: String,
    },
    /// 触发继电器脉冲。
    RelayPulse {
        /// 继电器通道号。
        channel: u8,
        /// 脉冲持续时间，单位毫秒。
        duration_ms: u64,
    },
    /// 写 RS-485 帧。
    Rs485Write {
        /// 串口名或设备路径。
        port: String,
        /// 十六进制帧内容，例如 `01 05 00 00 FF 00`。
        frame_hex: String,
    },
}

/// 单条告警规则。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AlarmRule {
    /// 稳定规则 code。
    pub code: String,
    /// 匹配的算法事件 code。
    pub event_code: String,
    /// 最低事件分数。
    pub min_score: f32,
    /// 告警输出通道。
    pub targets: Vec<AlarmOutputTarget>,
}

/// 单个待执行告警动作。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AlarmAction {
    /// 来源规则 code。
    pub rule_code: String,
    /// 来源算法 code。
    pub algorithm_code: String,
    /// 来源帧序号。
    pub frame_index: u64,
    /// 来源帧时间戳，单位毫秒。
    pub timestamp_ms: u64,
    /// 来源事件 code。
    pub event_code: String,
    /// 事件分数。
    pub score: f32,
    /// 动作目标。
    pub target: AlarmOutputTarget,
    /// 结构化 payload，供外层执行器直接序列化。
    pub payload: serde_json::Value,
}

/// 根据算法帧结果和规则生成告警动作计划。
///
/// # Errors
/// 规则阈值无效、规则 code 为空或输出目标为空时返回错误。
pub fn plan_alarm_actions(
    frame_results: &[VideoAlgorithmFrameResult],
    rules: &[AlarmRule],
) -> anyhow::Result<Vec<AlarmAction>> {
    validate_rules(rules)?;
    let mut actions = Vec::new();
    for result in frame_results {
        for event in &result.events {
            for rule in rules
                .iter()
                .filter(|rule| rule.event_code == event.event_code && event.score >= rule.min_score)
            {
                for target in &rule.targets {
                    actions.push(AlarmAction {
                        rule_code: rule.code.clone(),
                        algorithm_code: result.algorithm_code.clone(),
                        frame_index: result.frame_index,
                        timestamp_ms: result.timestamp_ms,
                        event_code: event.event_code.clone(),
                        score: event.score,
                        target: target.clone(),
                        payload: alarm_payload(result, event),
                    });
                }
            }
        }
    }
    Ok(actions)
}

fn validate_rules(rules: &[AlarmRule]) -> anyhow::Result<()> {
    for rule in rules {
        if rule.code.trim().is_empty() {
            anyhow::bail!("alarm rule code cannot be blank");
        }
        if rule.event_code.trim().is_empty() {
            anyhow::bail!("alarm rule event_code cannot be blank");
        }
        if !rule.min_score.is_finite() || !(0.0..=1.0).contains(&rule.min_score) {
            anyhow::bail!(
                "alarm rule `{}` min_score must be within 0..=1, got {}",
                rule.code,
                rule.min_score
            );
        }
        if rule.targets.is_empty() {
            anyhow::bail!("alarm rule `{}` must contain at least one target", rule.code);
        }
        validate_targets(rule)?;
    }
    Ok(())
}

fn validate_targets(rule: &AlarmRule) -> anyhow::Result<()> {
    for target in &rule.targets {
        match target {
            AlarmOutputTarget::Mqtt { topic, qos } => {
                if topic.trim().is_empty() {
                    anyhow::bail!("alarm rule `{}` mqtt topic cannot be blank", rule.code);
                }
                if *qos > 2 {
                    anyhow::bail!("alarm rule `{}` mqtt qos must be 0, 1 or 2", rule.code);
                }
            }
            AlarmOutputTarget::HttpPost { url } => {
                if url.trim().is_empty() {
                    anyhow::bail!("alarm rule `{}` http url cannot be blank", rule.code);
                }
            }
            AlarmOutputTarget::RelayPulse {
                channel: _,
                duration_ms,
            } => {
                if *duration_ms == 0 {
                    anyhow::bail!("alarm rule `{}` relay duration_ms must be positive", rule.code);
                }
            }
            AlarmOutputTarget::Rs485Write { port, frame_hex } => {
                if port.trim().is_empty() {
                    anyhow::bail!("alarm rule `{}` rs485 port cannot be blank", rule.code);
                }
                if parse_hex_frame(frame_hex).is_none() {
                    anyhow::bail!("alarm rule `{}` rs485 frame_hex is invalid", rule.code);
                }
            }
        }
    }
    Ok(())
}

fn alarm_payload(
    result: &VideoAlgorithmFrameResult,
    event: &crate::video_pipeline::model::VideoAlgorithmEvent,
) -> serde_json::Value {
    let mut details = BTreeMap::new();
    details.insert("algorithm_code", json!(result.algorithm_code));
    details.insert("frame_index", json!(result.frame_index));
    details.insert("timestamp_ms", json!(result.timestamp_ms));
    details.insert("event_code", json!(event.event_code));
    details.insert("score", json!(event.score));
    details.insert("message", json!(event.message));
    details.insert("extra", event.extra.clone());
    json!(details)
}

fn parse_hex_frame(value: &str) -> Option<Vec<u8>> {
    let normalized = value
        .split(|ch: char| ch.is_ascii_whitespace() || ch == ':' || ch == '-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        return None;
    }
    normalized
        .iter()
        .map(|part| u8::from_str_radix(part, 16).ok())
        .collect()
}
