
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RelationKind {
    /// One-to-one (1:1).
    OneToOne,
    /// One-to-many (1:N).
    OneToMany,
    /// Many-to-one (N:1).
    ManyToOne,
    /// Many-to-many (N:N).
    ManyToMany,
}

impl RelationKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Relation {
    /// Name of this relation (e.g., "fk_user_order").
    pub name: String,
    /// The source table.
    pub from_table: String,
    /// The source column.
    pub from_column: String,
    /// The target table.
    pub to_table: String,
    /// The target column.
    pub to_column: String,
    /// The type of relationship.
    pub kind: RelationKind,
    /// Whether to cascade on delete.
    pub on_delete_cascade: bool,
    /// Whether to cascade on update.
    pub on_update_cascade: bool,
}

impl Relation {
    /// Create a new relation.
    pub fn new(
        name: impl Into<String>,
        from_table: impl Into<String>,
        from_column: impl Into<String>,
        to_table: impl Into<String>,
        to_column: impl Into<String>,
        kind: RelationKind,
    ) -> Self {
        Self {
            name: name.into(),
            from_table: from_table.into(),
            from_column: from_column.into(),
            to_table: to_table.into(),
            to_column: to_column.into(),
            kind,
            on_delete_cascade: false,
            on_update_cascade: false,
        }
    }

    /// Enable cascading delete.
    pub fn on_delete_cascade(mut self) -> Self {
        self.on_delete_cascade = true;
        self
    }

    /// Enable cascading update.
    pub fn on_update_cascade(mut self) -> Self {
        self.on_update_cascade = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::relation::{Relation, RelationKind};

    #[test]
    fn relation_builder() {
        let rel = Relation::new(
            "fk_order_user",
            "orders",
            "user_id",
            "users",
            "id",
            RelationKind::ManyToOne,
        )
        .on_delete_cascade()
        .on_update_cascade();

        assert_eq!(rel.name, "fk_order_user");
        assert_eq!(rel.from_table, "orders");
        assert_eq!(rel.to_table, "users");
        assert_eq!(rel.kind, RelationKind::ManyToOne);
        assert!(rel.on_delete_cascade);
        assert!(rel.on_update_cascade);
    }

    #[test]
    fn relation_serialization_roundtrip() {
        let rel = Relation::new(
            "fk_post_author",
            "posts",
            "author_id",
            "users",
            "id",
            RelationKind::ManyToOne,
        );

        let json = serde_json::to_string(&rel).unwrap();
        let deserialized: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "fk_post_author");
        assert_eq!(deserialized.kind, RelationKind::ManyToOne);
    }

    #[test]
    fn relation_kind_uses_snake_case_wire_codes() {
        assert_eq!(RelationKind::ManyToMany.code(), "many_to_many");
        assert_eq!(
            serde_json::to_string(&RelationKind::ManyToOne).unwrap(),
            "\"many_to_one\""
        );
        assert_eq!(
            RelationKind::from_code("one_to_one"),
            Some(RelationKind::OneToOne)
        );
    }

    #[test]
    fn many_to_many_relation() {
        let rel = Relation::new(
            "fk_student_course",
            "students",
            "id",
            "courses",
            "id",
            RelationKind::ManyToMany,
        );
        assert_eq!(rel.kind, RelationKind::ManyToMany);
    }
}
