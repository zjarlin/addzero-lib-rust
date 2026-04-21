use addzero_reflection::*;
use serde::Serialize;
use serde_json::json;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
struct Address {
    street: String,
    zip_code: String,
}

#[derive(Debug, Clone, Serialize)]
struct Role {
    code: String,
}

#[derive(Debug, Clone, Serialize)]
struct User {
    id: Option<u64>,
    name: String,
    address: Address,
    roles: Vec<Role>,
}

reflect_meta!(
    Address,
    description = "Address value object",
    [
        field_info!(street: String => "Street line"),
        field_info!(zip_code: String => "Zip code", column = "zip_code"),
    ]
);

reflect_meta!(Role, [field_info!(code: String => "Role code"),]);

reflect_meta!(
    User,
    description = "User aggregate",
    [
        field_info!(id: Option<u64> => "Database id", column = "id"),
        field_info!(name: String => "Display name"),
        field_info!(address: Address => "Postal address", nested = Address),
        field_info!(roles: Vec<Role> => "Granted roles", collection = Role),
    ]
);

#[test]
fn contains_ignore_order_matches_bidirectional_substrings() {
    assert!(contains_ignore_order("hello-world", "world"));
    assert!(contains_ignore_order("world", "hello-world"));
    assert!(!contains_ignore_order("hello", "rust"));
}

#[test]
fn is_new_matches_blank_or_empty_fields() {
    let blank = User {
        id: None,
        name: "   ".to_owned(),
        address: Address {
            street: String::new(),
            zip_code: String::new(),
        },
        roles: Vec::new(),
    };
    let populated = User {
        id: Some(1),
        name: "Alice".to_owned(),
        address: Address {
            street: "Main St".to_owned(),
            zip_code: "10001".to_owned(),
        },
        roles: vec![Role {
            code: "admin".to_owned(),
        }],
    };

    assert!(!is_new(&blank));
    assert!(is_not_new(&populated));
}

#[test]
fn value_kind_helpers_detect_arrays_and_objects() {
    assert!(is_collection_value(&json!(["a", "b"])));
    assert!(is_custom_object_value(&json!({"key": "value"})));
    assert!(!is_collection_value(&json!("plain")));
}

#[test]
fn extract_table_name_and_guess_column_name_follow_expected_rules() {
    assert_eq!(
        extract_table_name("select * from user_profile where id = ?").as_deref(),
        Some("user_profile")
    );
    assert_eq!(guess_column_name("userName"), "user_name");
    assert_eq!(guess_column_name("HTTPStatusCode"), "http_status_code");
}

#[test]
fn metadata_macros_build_nested_field_information() {
    let infos = get_field_infos::<User>();
    let address = infos
        .iter()
        .find(|field| field.field_name == "address")
        .expect("address field should exist");
    let roles = infos
        .iter()
        .find(|field| field.field_name == "roles")
        .expect("roles field should exist");

    assert_eq!(User::type_description(), Some("User aggregate"));
    assert!(address.is_nested_object);
    assert_eq!(address.children.len(), 2);
    assert!(roles.is_nested_object);
    assert_eq!(roles.children[0].field_name, "code");

    let flattened = get_simple_field_info_str::<User>();
    assert!(flattened.contains("address: Postal address"));
    assert!(flattened.contains("street: Street line"));
    assert!(flattened.contains("roles: Granted roles"));
}

#[test]
fn expiring_cache_reuses_values_and_evicts_old_entries() {
    let cache = ExpiringCache::new(Duration::from_secs(5), 2);

    let first = cache.compute_if_absent("alpha", |_| 1usize);
    let second = cache.compute_if_absent("alpha", |_| 2usize);
    cache.compute_if_absent("beta", |_| 3usize);
    cache.compute_if_absent("gamma", |_| 4usize);

    assert_eq!(first, 1);
    assert_eq!(second, 1);
    assert_eq!(cache.len(), 2);
}

#[test]
fn expiring_cache_cleans_up_elapsed_entries() {
    let cache = ExpiringCache::new(Duration::from_millis(5), 4);
    cache.compute_if_absent("alpha", |_| 1usize);
    thread::sleep(Duration::from_millis(10));

    cache.cleanup_expired();

    assert!(cache.is_empty());
}
