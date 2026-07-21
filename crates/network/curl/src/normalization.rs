pub(crate) fn normalize_command(command: &str) -> String {
    let mut normalized = String::with_capacity(command.len());
    let mut chars = command.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\' {
            let mut buffered = String::new();
            while chars
                .peek()
                .is_some_and(|next| *next == ' ' || *next == '\t')
            {
                if let Some(space) = chars.next() {
                    buffered.push(space);
                }
            }
            match chars.peek() {
                Some('\n') => {
                    chars.next();
                    normalized.push(' ');
                }
                Some('\r') => {
                    chars.next();
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    normalized.push(' ');
                }
                _ => {
                    normalized.push(character);
                    normalized.push_str(&buffered);
                }
            }
        } else {
            normalized.push(character);
        }
    }
    normalized
}

pub(crate) fn normalize_header_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub(crate) fn looks_like_json(value: &str) -> bool {
    let trimmed = value.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}
