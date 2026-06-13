//! Stable TOML rendering for version catalogs.

use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

use crate::model::{LibraryEntry, PluginEntry, VersionCatalog};

pub(crate) fn render_pretty_catalog(catalog: &VersionCatalog) -> String {
    let mut doc = DocumentMut::new();

    if !catalog.versions.is_empty() {
        let mut table = Table::new();
        let mut entries = catalog.versions.iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.version_ref.cmp(&right.version_ref));
        for entry in entries {
            table.insert(&entry.version_ref, value_item(entry.version.clone()));
        }
        doc["versions"] = Item::Table(table);
    }

    if !catalog.libraries.is_empty() {
        let mut table = Table::new();
        let mut entries = catalog.libraries.iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.cmp(&right.key));
        for entry in entries {
            table.insert(&entry.key, build_library_item(entry));
        }
        doc["libraries"] = Item::Table(table);
    }

    if !catalog.plugins.is_empty() {
        let mut table = Table::new();
        let mut entries = catalog.plugins.iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.cmp(&right.key));
        for entry in entries {
            table.insert(&entry.key, build_plugin_item(entry));
        }
        doc["plugins"] = Item::Table(table);
    }

    if !catalog.bundles.is_empty() {
        let mut table = Table::new();
        let mut entries = catalog.bundles.iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.cmp(&right.key));
        for entry in entries {
            let mut array = Array::default();
            for library in &entry.libraries {
                array.push(library.as_str());
            }
            table.insert(&entry.key, Item::Value(Value::Array(array)));
        }
        doc["bundles"] = Item::Table(table);
    }

    doc.to_string()
}

fn value_item(value: impl Into<Value>) -> Item {
    Item::Value(value.into())
}

fn build_library_item(entry: &LibraryEntry) -> Item {
    let mut table = InlineTable::new();
    table.insert("group", Value::from(entry.group.clone()));
    table.insert("name", Value::from(entry.name.clone()));
    if let Some(version) = &entry.version {
        table.insert("version", Value::from(version.clone()));
    }
    if let Some(version_ref) = &entry.version_ref {
        table.insert("version", version_ref_item(version_ref));
    }
    Item::Value(Value::InlineTable(table))
}

fn build_plugin_item(entry: &PluginEntry) -> Item {
    let mut table = InlineTable::new();
    table.insert("id", Value::from(entry.id.clone()));
    if let Some(version) = &entry.version {
        table.insert("version", Value::from(version.clone()));
    }
    if let Some(version_ref) = &entry.version_ref {
        table.insert("version", version_ref_item(version_ref));
    }
    Item::Value(Value::InlineTable(table))
}

fn version_ref_item(version_ref: &str) -> Value {
    let mut version_table = InlineTable::new();
    version_table.insert("ref", Value::from(version_ref.to_owned()));
    version_table.set_dotted(true);
    Value::InlineTable(version_table)
}
