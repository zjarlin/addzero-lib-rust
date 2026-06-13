use az_sql::{JoinType, SortOrder};

#[test]
fn sort_order_keeps_sql_display_and_exposes_codes() {
    assert_eq!(SortOrder::Asc.to_string(), "ASC");
    assert_eq!(SortOrder::Desc.code(), "desc");
    assert_eq!(SortOrder::from_code("asc"), Some(SortOrder::Asc));
}

#[test]
fn join_type_keeps_sql_display_and_exposes_codes() {
    assert_eq!(JoinType::Inner.to_string(), "INNER JOIN");
    assert_eq!(JoinType::FullOuter.code(), "full_outer");
    assert_eq!(JoinType::from_code("left"), Some(JoinType::Left));
}
