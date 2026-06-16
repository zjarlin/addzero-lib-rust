use std::sync::Arc;

use crate::contract::{
    DynLowcodeMetadataProvider, LowcodeFieldDescriptor, LowcodeMenuContribution,
    LowcodeMetadataProvider, LowcodeModelDescriptor, LowcodeRelationDescriptor,
};
use rudi::Singleton;

use crate::backend::{model::MetaFieldView, store::LowcodeStore};

pub struct StoreLowcodeMetadataProvider;

impl LowcodeMetadataProvider for StoreLowcodeMetadataProvider {
    fn models(&self) -> anyhow::Result<Vec<LowcodeModelDescriptor>> {
        let store = LowcodeStore::global();
        store.seed_demo();
        Ok(store
            .list_models_sync()
            .into_iter()
            .map(|model| {
                let fields = store
                    .list_fields_sync(&model.id)
                    .into_iter()
                    .map(field_descriptor_from_view)
                    .collect();
                LowcodeModelDescriptor {
                    id: model.id,
                    name: model.name,
                    label: model.label,
                    description: model.description,
                    fields,
                }
            })
            .collect())
    }

    fn menus(&self) -> anyhow::Result<Vec<LowcodeMenuContribution>> {
        Ok(configurable_lowcode_menus())
    }
}

#[Singleton(name = "lowcode-store-metadata")]
pub fn store_lowcode_metadata_provider() -> DynLowcodeMetadataProvider {
    Arc::new(StoreLowcodeMetadataProvider)
}

pub fn metadata_provider() -> DynLowcodeMetadataProvider {
    Arc::new(StoreLowcodeMetadataProvider)
}

pub fn configurable_lowcode_menus() -> Vec<LowcodeMenuContribution> {
    vec![
        lowcode_menu("lowcode.root", None, "低代码", "/lowcode", "▣", 10),
        lowcode_menu(
            "lowcode.models",
            Some("lowcode.root"),
            "元数据模型",
            "/lowcode",
            "▤",
            10,
        ),
        lowcode_menu(
            "lowcode.screens",
            Some("lowcode.root"),
            "页面配置",
            "/lowcode?mode=screens",
            "☷",
            20,
        ),
    ]
}

fn lowcode_menu(
    id: &str,
    parent_id: Option<&str>,
    label: &str,
    route: &str,
    icon: &str,
    order: i32,
) -> LowcodeMenuContribution {
    LowcodeMenuContribution {
        id: id.to_string(),
        parent_id: parent_id.map(str::to_string),
        label: label.to_string(),
        route: route.to_string(),
        icon: icon.to_string(),
        order,
        visible: true,
        permissions_any_of: Vec::new(),
        metadata: serde_json::json!({ "source": "lowcode-config" }),
    }
}

pub fn field_descriptor_from_view(field: MetaFieldView) -> LowcodeFieldDescriptor {
    LowcodeFieldDescriptor {
        id: field.id,
        name: field.name,
        label: field.label,
        field_type: field.field_type,
        order: field.order,
        required: field.is_required,
        unique: field.is_unique,
        relation: match (field.relation_type, field.relation_model_id) {
            (Some(relation_type), Some(target_model_id)) => Some(LowcodeRelationDescriptor {
                relation_type,
                target_model_id,
            }),
            _ => None,
        },
        default_value: field.default_value,
        enum_options: field
            .enum_options
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.trim().to_string())
            .collect(),
    }
}
