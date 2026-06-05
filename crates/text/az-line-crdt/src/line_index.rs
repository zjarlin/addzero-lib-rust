#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineSpan {
    pub content_start: usize,
    pub content_end: usize,
    pub has_trailing_newline: bool,
}

impl LineSpan {
    pub fn content_len(self) -> usize {
        self.content_end - self.content_start
    }
}

pub(crate) fn line_spans(text: &str) -> Vec<LineSpan> {
    let mut spans = Vec::new();
    let mut position = 0;
    let mut content_start = 0;

    for character in text.chars() {
        if character == '\n' {
            spans.push(LineSpan {
                content_start,
                content_end: position,
                has_trailing_newline: true,
            });
            position += 1;
            content_start = position;
        } else {
            position += 1;
        }
    }

    spans.push(LineSpan {
        content_start,
        content_end: position,
        has_trailing_newline: false,
    });

    spans
}

pub(crate) fn unicode_len(text: &str) -> usize {
    text.chars().count()
}
