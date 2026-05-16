use toasty_core::driver::{Capability, SchemaMutations, StorageTypes};
use toasty_core::schema::db;

/// Storage type defaults tailored for gitdb's current SQL/type support.
pub(crate) const STORAGE_TYPES_GITDB: StorageTypes = StorageTypes {
    default_string_type: db::Type::Text,
    varchar: Some(1_000_000_000),
    default_uuid_type: db::Type::Text,
    default_bytes_type: db::Type::Blob,
    default_decimal_type: db::Type::Text,
    default_bigdecimal_type: db::Type::Text,
    default_timestamp_type: db::Type::Text,
    default_zoned_type: db::Type::Text,
    default_date_type: db::Type::Text,
    default_time_type: db::Type::Text,
    default_datetime_type: db::Type::Text,
    max_unsigned_integer: Some(i64::MAX as u64),
};

/// Schema mutation support currently matches gitdb's create-table-first model.
pub(crate) const SCHEMA_MUTATIONS_GITDB: SchemaMutations = SchemaMutations {
    alter_column_type: false,
    alter_column_properties_atomic: false,
};

/// Capability definition used by the Toasty planner for gitdb.
pub(crate) const CAPABILITY_GITDB: Capability = Capability {
    sql: true,
    storage_types: STORAGE_TYPES_GITDB,
    schema_mutations: SCHEMA_MUTATIONS_GITDB,
    cte_with_update: false,
    select_for_update: false,
    returning_from_mutation: false,
    primary_key_ne_predicate: true,
    auto_increment: false,
    native_varchar: true,
    native_timestamp: false,
    native_date: false,
    native_time: false,
    native_datetime: false,
    native_enum: false,
    named_enum_types: false,
    native_decimal: false,
    bigdecimal_implemented: false,
    decimal_arbitrary_precision: false,
    index_or_predicate: true,
    native_starts_with: false,
    native_like: true,
    scan: true,
    scan_supports_sort: true,
    test_connection_pool: false,
    backward_pagination: true,
    bind_list_param: true,
    predicate_match_any: false,
    native_array: false,
    vec_scalar: true,
    native_array_set_predicates: false,
    vec_remove: false,
    vec_pop: false,
    vec_remove_at: false,
};
