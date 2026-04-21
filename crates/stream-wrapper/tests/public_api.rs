use addzero_stream_wrapper::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct User {
    name: String,
    city: String,
    role: String,
}

fn users() -> Vec<User> {
    vec![
        User {
            name: "Alice".to_owned(),
            city: "Shanghai".to_owned(),
            role: "user".to_owned(),
        },
        User {
            name: "Bob".to_owned(),
            city: "Beijing".to_owned(),
            role: "admin".to_owned(),
        },
        User {
            name: "Alina".to_owned(),
            city: "Shenzhen".to_owned(),
            role: "user".to_owned(),
        },
    ]
}

#[test]
fn eq_and_like_filters_are_chainable() {
    let result = lambdaquery(users())
        .eq(true, |user| user.city.as_str(), "Shanghai")
        .like(true, |user| user.name.as_str(), "ali")
        .list();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Alice");
}

#[test]
fn in_filter_matches_any_candidate() {
    let result = lambdaquery(users())
        .r#in(true, |user| user.role.as_str(), ["admin", "owner"])
        .list();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Bob");
}

#[test]
fn or_only_affects_next_condition_then_resets_to_and() {
    let result = lambdaquery(users())
        .eq(true, |user| user.city.as_str(), "Beijing")
        .or()
        .like(true, |user| user.name.as_str(), "ali")
        .eq(true, |user| user.role.as_str(), "user")
        .list();

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|user| user.role == "user"));
}

#[test]
fn not_only_negates_next_condition() {
    let result = lambdaquery(users())
        .not()
        .eq(true, |user| user.city.as_str(), "Beijing")
        .list();

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|user| user.city != "Beijing"));
}

#[test]
fn negate_flips_the_whole_accumulated_predicate() {
    let result = lambdaquery(users())
        .eq(true, |user| user.role.as_str(), "admin")
        .negate()
        .list();

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|user| user.role != "admin"));
}

#[test]
fn one_returns_first_matching_item() {
    let user = stream_query!(users())
        .like(true, |item| item.name.as_str(), "bob")
        .one();

    assert_eq!(user.map(|item| item.name), Some("Bob".to_owned()));
}
