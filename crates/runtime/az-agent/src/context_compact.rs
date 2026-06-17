/// A single item in an agent conversation before compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactMessage {
    /// Stable role label such as `user`, `assistant`, `tool`, or `system`.
    pub role: String,
    /// Plain text payload to compact.
    pub content: String,
}

/// Controls deterministic local context compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactPolicy {
    /// Maximum characters kept in the compacted context.
    pub max_chars: usize,
    /// Number of newest messages preserved verbatim.
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

/// Compacts old context into a deterministic summary plus recent verbatim turns.
#[derive(Debug, Clone)]
pub struct ContextCompactor {
    policy: CompactPolicy,
}

impl ContextCompactor {
    /// Creates a compactor with the given policy.
    pub fn new(policy: CompactPolicy) -> Self {
        Self { policy }
    }

    /// Produces a compacted context without depending on a model call.
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
