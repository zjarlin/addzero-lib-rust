use az_dioxus_components::az_grammar_search::{AzGrammarSearchField, parse_grammar_search_query};

#[test]
fn parser_splits_semicolon_conditions_and_comma_values() {
    let query =
        parse_grammar_search_query("keyword:addhost；tag:rust,java,go;def:fun,export,alias");

    assert_eq!(query.values_for("keyword"), vec!["addhost"]);
    assert_eq!(query.values_for("tag"), vec!["rust", "java", "go"]);
    assert_eq!(query.values_for("def"), vec!["fun", "export", "alias"]);
}

#[test]
fn parser_keeps_github_style_comparators_as_values() {
    let query = parse_grammar_search_query("gitdb language:Java stars:>100 owner:openai");

    assert_eq!(query.terms[0].value, "gitdb");
    assert_eq!(query.values_for("language"), vec!["Java"]);
    assert_eq!(query.values_for("stars"), vec![">100"]);
    assert_eq!(query.values_for("owner"), vec!["openai"]);
}

#[test]
fn field_hint_preserves_key_and_label() {
    let field = AzGrammarSearchField::new("tag", "标签");

    assert_eq!(field.key, "tag");
    assert_eq!(field.label, "标签");
}
