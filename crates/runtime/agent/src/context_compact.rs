/// 压缩前 agent 对话中的单条消息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactMessage {
    /// 稳定角色标签，例如 `user`、`assistant`、`tool` 或 `system`。
    pub role: String,
    /// 需要压缩的纯文本内容。
    pub content: String,
}

/// 控制确定性本地上下文压缩。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactPolicy {
    /// 压缩后上下文最多保留的字符数。
    pub max_chars: usize,
    /// 原样保留的最新消息数量。
    pub preserve_recent: usize,
}

impl Default for CompactPolicy {
    fn default() -> Self {
        Self {
            max_chars: 8_000,
            preserve_recent: 6,
        }
    }
}

/// 将旧上下文压缩成确定性摘要，并保留最近对话原文。
#[derive(Debug, Clone)]
pub struct ContextCompactor {
    policy: CompactPolicy,
}

impl ContextCompactor {
    /// 使用指定策略创建压缩器。
    pub fn new(policy: CompactPolicy) -> Self {
        Self { policy }
    }

    /// 在不依赖模型调用的情况下生成压缩后上下文。
    pub fn compact(&self, messages: &[CompactMessage]) -> Vec<CompactMessage> {
        if messages.len() <= self.policy.preserve_recent {
            return messages.to_vec();
        }

        let split_at = messages.len() - self.policy.preserve_recent;
        let mut summary = summarize_messages(&messages[..split_at]);
        if summary.len() > self.policy.max_chars {
            summary.truncate(self.policy.max_chars);
        }

        let mut compacted = vec![CompactMessage {
            role: "system".to_string(),
            content: format!("Compacted prior context:\n{summary}"),
        }];
        compacted.extend_from_slice(&messages[split_at..]);
        compacted
    }
}

fn summarize_messages(messages: &[CompactMessage]) -> String {
    messages
        .iter()
        .filter(|message| !message.content.trim().is_empty())
        .map(|message| {
            let content = message.content.replace('\n', " ");
            format!("- {}: {}", message.role, content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compacts_old_messages_and_preserves_recent_turns() {
        let compactor = ContextCompactor::new(CompactPolicy {
            max_chars: 500,
            preserve_recent: 2,
        });
        let messages = vec![
            CompactMessage {
                role: "user".to_string(),
                content: "first".to_string(),
            },
            CompactMessage {
                role: "assistant".to_string(),
                content: "second".to_string(),
            },
            CompactMessage {
                role: "user".to_string(),
                content: "third".to_string(),
            },
            CompactMessage {
                role: "assistant".to_string(),
                content: "fourth".to_string(),
            },
        ];

        let compacted = compactor.compact(&messages);

        assert_eq!(compacted.len(), 3);
        assert!(compacted[0].content.contains("first"));
        assert_eq!(compacted[1].content, "third");
        assert_eq!(compacted[2].content, "fourth");
    }
}
