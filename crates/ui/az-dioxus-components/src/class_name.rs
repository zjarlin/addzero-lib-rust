pub(crate) fn compose_class(base: &str, extra: &str, modifiers: &[(&str, bool)]) -> String {
    let extra = extra.trim();
    let mut classes = Vec::with_capacity(1 + modifiers.len() + usize::from(!extra.is_empty()));
    classes.push(base.to_string());
    classes.extend(
        modifiers
            .iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| (*name).to_string()),
    );

    if !extra.is_empty() {
        classes.push(extra.to_string());
    }

    classes.join(" ")
}
