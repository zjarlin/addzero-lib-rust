pub(crate) fn trim_non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub(crate) fn non_blank(value: Option<&str>) -> Option<&str> {
    trim_non_blank(value)
}
