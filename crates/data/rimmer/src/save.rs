use crate::draft::{Draft, DraftField, ErasedDraft};
use crate::expression::quote_identifier;
use crate::metadata::FieldKind;
use crate::value::ScalarValue;
use anyhow::{anyhow, bail};

/// 保存模式。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SaveMode {
    /// 根据主键是否出现推导插入或更新。
    #[default]
    Upsert,
    /// 只允许插入。
    InsertOnly,
    /// 只允许更新。
    UpdateOnly,
}

/// Jimmer 风格保存命令。
pub struct SaveCommand<E> {
    draft: Draft<E>,
    mode: SaveMode,
}

impl<E> SaveCommand<E> {
    /// 创建保存命令。
    pub fn new(draft: Draft<E>) -> Self {
        Self {
            draft,
            mode: SaveMode::Upsert,
        }
    }

    /// 设置保存模式。
    pub fn set_mode(mut self, mode: SaveMode) -> Self {
        self.mode = mode;
        self
    }

    /// 构建保存 SQL plan。
    pub fn build(self) -> anyhow::Result<SavePlan> {
        build_save_plan(self.draft.into_erased(), self.mode)
    }
}

fn build_save_plan(draft: ErasedDraft, requested_mode: SaveMode) -> anyhow::Result<SavePlan> {
    let mode = actual_mode(&draft, requested_mode);
    let mut plan = match mode {
        SaveMode::InsertOnly => build_insert_plan(&draft)?,
        SaveMode::UpdateOnly => build_update_plan(&draft)?,
        SaveMode::Upsert => unreachable!("actual_mode never returns upsert"),
    };
    plan.children = build_child_plans(&draft, requested_mode)?;
    Ok(plan)
}

fn actual_mode(draft: &ErasedDraft, requested_mode: SaveMode) -> SaveMode {
    match requested_mode {
        SaveMode::Upsert => {
            if id_draft_field(draft).is_some() {
                SaveMode::UpdateOnly
            } else {
                SaveMode::InsertOnly
            }
        }
        mode => mode,
    }
}

fn build_insert_plan(draft: &ErasedDraft) -> anyhow::Result<SavePlan> {
    let writable_fields = draft
        .fields()
        .iter()
        .filter(|field| field.kind().is_persistent_column())
        .collect::<Vec<_>>();
    if writable_fields.is_empty() {
        bail!("save command has no writable fields");
    }

    let columns = writable_fields
        .iter()
        .map(|field| quote_identifier(field.column_name()))
        .collect::<Vec<_>>();
    let placeholders = vec!["?"; writable_fields.len()].join(", ");
    let params = writable_fields
        .iter()
        .map(|field| field.value().clone())
        .collect::<Vec<_>>();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_identifier(draft.entity().table_name()),
        columns.join(", "),
        placeholders
    );

    Ok(SavePlan {
        sql,
        params,
        mode: SaveMode::InsertOnly,
        children: Vec::new(),
    })
}

fn build_update_plan(draft: &ErasedDraft) -> anyhow::Result<SavePlan> {
    let id_metadata = draft.entity().id_field().ok_or_else(|| {
        anyhow!("entity '{}' has no id field", draft.entity().type_name())
    })?;
    let id_field = id_draft_field(draft)
        .ok_or_else(|| anyhow!("save command requires id field for update"))?;
    let writable_fields = draft
        .fields()
        .iter()
        .filter(|field| field.kind().is_persistent_column() && field.kind() != FieldKind::Id)
        .collect::<Vec<_>>();
    if writable_fields.is_empty() {
        bail!("save command has no writable fields");
    }

    let assignments = writable_fields
        .iter()
        .map(|field| format!("{} = ?", quote_identifier(field.column_name())))
        .collect::<Vec<_>>();
    let mut params = writable_fields
        .iter()
        .map(|field| field.value().clone())
        .collect::<Vec<_>>();
    params.push(id_field.value().clone());
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        quote_identifier(draft.entity().table_name()),
        assignments.join(", "),
        quote_identifier(id_metadata.column_name())
    );

    Ok(SavePlan {
        sql,
        params,
        mode: SaveMode::UpdateOnly,
        children: Vec::new(),
    })
}

fn build_child_plans(draft: &ErasedDraft, mode: SaveMode) -> anyhow::Result<Vec<SavePlan>> {
    let mut plans = Vec::new();
    for collection in draft.collections() {
        let parent_value = draft
            .field_by_column(collection.source_column())
            .ok_or_else(|| {
                anyhow!(
                    "graph save requires parent value for entity '{}', column '{}', collection '{}'",
                    draft.entity().type_name(),
                    collection.source_column(),
                    collection.name()
                )
            })?
            .value()
            .clone();
        for child in collection.children() {
            let mut child = child.clone();
            child.set_raw(
                collection.target_rust_name(),
                collection.target_column(),
                collection.target_kind(),
                parent_value.clone(),
            );
            let child_plan = build_save_plan(child, mode)?;
            plans.push(child_plan);
        }
    }
    Ok(plans)
}

fn id_draft_field(draft: &ErasedDraft) -> Option<&DraftField> {
    draft
        .fields()
        .iter()
        .find(|field| field.kind() == FieldKind::Id)
}

/// 可执行前的保存 SQL 计划。
#[derive(Clone, Debug, PartialEq)]
pub struct SavePlan {
    /// 参数化 SQL。
    pub sql: String,
    /// SQL 参数。
    pub params: Vec<ScalarValue>,
    /// 实际保存模式。
    pub mode: SaveMode,
    /// 子对象保存计划。
    pub children: Vec<SavePlan>,
}
