//! 系统字典 PostgreSQL 读写服务。

use std::collections::BTreeMap;

use anyhow::{Context, anyhow, bail};
use serde_json::Value;
use toasty::stmt::{List, Query};

use crate::{
    core::db,
    system::{
        dictionary_model::{
            DictionaryItemInput, DictionaryItemPage, DictionaryItemRecord,
            DictionaryItemSummary, DictionaryPagination, DictionaryTypeInput,
            DictionaryTypeRecord, DictionaryTypeSummary,
        },
        store::SystemAdminStore,
    },
};

impl SystemAdminStore {
    /// 按作用域、排序和名称返回字典类型树数据。
    pub async fn list_dictionary_types(&self) -> anyhow::Result<Vec<DictionaryTypeSummary>> {
        let mut database = self.db.lock().await;
        let type_records = Query::<List<DictionaryTypeRecord>>::all()
            .exec(&mut *database)
            .await
            .context("读取字典类型失败")?;
        let item_records = Query::<List<DictionaryItemRecord>>::all()
            .exec(&mut *database)
            .await
            .context("读取字典项数量失败")?;
        drop(database);

        let mut item_counts = BTreeMap::<String, usize>::new();
        for item in item_records {
            *item_counts.entry(item.dictionary_type_id).or_default() += 1;
        }

        let mut types = type_records
            .into_iter()
            .map(|record| {
                let item_count = item_counts.get(&record.id).copied().unwrap_or_default();
                record.summary(item_count)
            })
            .collect::<Vec<_>>();
        types.sort_by(|left, right| {
            left.scope
                .cmp(&right.scope)
                .then_with(|| left.sort_index.cmp(&right.sort_index))
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(types)
    }

    /// 创建字典类型并校验编码唯一性。
    pub async fn create_dictionary_type(
        &self,
        input: DictionaryTypeInput,
    ) -> anyhow::Result<DictionaryTypeSummary> {
        let input = normalize_dictionary_type_input(input)?;
        self.ensure_dictionary_type_code_available(&input.code, None)
            .await?;

        let now = db::timestamp_ms();
        let mut database = self.db.lock().await;
        let record = DictionaryTypeRecord::create()
            .id(db::new_uuid_id())
            .code(input.code)
            .name(input.name)
            .description(input.description)
            .scope(input.scope)
            .raw_value_kind(input.raw_value_kind)
            .open_enum(input.open_enum)
            .sort_index(input.sort_index)
            .status(input.status)
            .created_at_ms(now)
            .updated_at_ms(now)
            .exec(&mut *database)
            .await
            .context("创建字典类型失败")?;
        Ok(record.summary(0))
    }

    /// 更新字典类型。
    pub async fn update_dictionary_type(
        &self,
        id: &str,
        input: DictionaryTypeInput,
    ) -> anyhow::Result<DictionaryTypeSummary> {
        let id = required_id(id, "字典类型")?;
        let input = normalize_dictionary_type_input(input)?;
        self.dictionary_type_record(id).await?;
        self.ensure_dictionary_type_code_available(&input.code, Some(id))
            .await?;

        let now = db::timestamp_ms();
        let mut database = self.db.lock().await;
        DictionaryTypeRecord::filter(DictionaryTypeRecord::fields().id().eq(id))
            .update()
            .code(input.code)
            .name(input.name)
            .description(input.description)
            .scope(input.scope)
            .raw_value_kind(input.raw_value_kind)
            .open_enum(input.open_enum)
            .sort_index(input.sort_index)
            .status(input.status)
            .updated_at_ms(now)
            .exec(&mut *database)
            .await
            .context("更新字典类型失败")?;
        drop(database);

        let record = self.dictionary_type_record(id).await?;
        let item_count = self.dictionary_item_count(id).await?;
        Ok(record.summary(item_count))
    }

    /// 删除字典类型及其全部字典项。
    pub async fn delete_dictionary_type(&self, id: &str) -> anyhow::Result<()> {
        let id = required_id(id, "字典类型")?;
        self.dictionary_type_record(id).await?;

        let mut database = self.db.lock().await;
        DictionaryItemRecord::filter(
            DictionaryItemRecord::fields().dictionary_type_id().eq(id),
        )
        .delete()
        .exec(&mut *database)
        .await
        .context("删除字典类型下的字典项失败")?;
        DictionaryTypeRecord::filter(DictionaryTypeRecord::fields().id().eq(id))
            .delete()
            .exec(&mut *database)
            .await
            .context("删除字典类型失败")?;
        Ok(())
    }

    /// 分页读取选中字典类型的字典项。
    pub async fn list_dictionary_items(
        &self,
        dictionary_type_id: &str,
        query: Option<&str>,
        offset: usize,
        size: usize,
    ) -> anyhow::Result<DictionaryItemPage> {
        let dictionary_type_id = required_id(dictionary_type_id, "字典类型")?;
        self.dictionary_type_record(dictionary_type_id).await?;

        let size = size.clamp(1, 100);
        let query = query.map(str::trim).filter(|value| !value.is_empty());
        let mut database = self.db.lock().await;
        let records = Query::<List<DictionaryItemRecord>>::filter(
            DictionaryItemRecord::fields()
                .dictionary_type_id()
                .eq(dictionary_type_id),
        )
        .exec(&mut *database)
        .await
        .context("读取字典项失败")?;
        drop(database);

        let mut items = records
            .into_iter()
            .filter(|record| query.is_none_or(|value| dictionary_item_matches(record, value)))
            .map(DictionaryItemSummary::from)
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.sort_index
                .cmp(&right.sort_index)
                .then_with(|| left.code.cmp(&right.code))
        });
        let total = items.len();
        let data = items.into_iter().skip(offset).take(size).collect();

        Ok(DictionaryItemPage {
            d: data,
            t: total,
            p: DictionaryPagination { o: offset, s: size },
        })
    }

    /// 创建字典项并校验编码和原始值唯一性。
    pub async fn create_dictionary_item(
        &self,
        input: DictionaryItemInput,
    ) -> anyhow::Result<DictionaryItemSummary> {
        let type_record = self.dictionary_type_record(&input.dictionary_type_id).await?;
        let input = normalize_dictionary_item_input(input, &type_record.raw_value_kind)?;
        self.ensure_dictionary_item_values_available(&input, None)
            .await?;

        let now = db::timestamp_ms();
        let mut database = self.db.lock().await;
        let record = DictionaryItemRecord::create()
            .id(db::new_uuid_id())
            .dictionary_type_id(input.dictionary_type_id)
            .code(input.code)
            .label(input.label)
            .description(input.description)
            .raw_value(input.raw_value)
            .sort_index(input.sort_index)
            .status(input.status)
            .meta_json(input.meta_json)
            .created_at_ms(now)
            .updated_at_ms(now)
            .exec(&mut *database)
            .await
            .context("创建字典项失败")?;
        Ok(record.into())
    }

    /// 更新字典项。
    pub async fn update_dictionary_item(
        &self,
        id: &str,
        input: DictionaryItemInput,
    ) -> anyhow::Result<DictionaryItemSummary> {
        let id = required_id(id, "字典项")?;
        self.dictionary_item_record(id).await?;
        let type_record = self.dictionary_type_record(&input.dictionary_type_id).await?;
        let input = normalize_dictionary_item_input(input, &type_record.raw_value_kind)?;
        self.ensure_dictionary_item_values_available(&input, Some(id))
            .await?;

        let now = db::timestamp_ms();
        let mut database = self.db.lock().await;
        DictionaryItemRecord::filter(DictionaryItemRecord::fields().id().eq(id))
            .update()
            .dictionary_type_id(input.dictionary_type_id)
            .code(input.code)
            .label(input.label)
            .description(input.description)
            .raw_value(input.raw_value)
            .sort_index(input.sort_index)
            .status(input.status)
            .meta_json(input.meta_json)
            .updated_at_ms(now)
            .exec(&mut *database)
            .await
            .context("更新字典项失败")?;
        drop(database);

        self.dictionary_item_record(id).await.map(Into::into)
    }

    /// 删除字典项。
    pub async fn delete_dictionary_item(&self, id: &str) -> anyhow::Result<()> {
        let id = required_id(id, "字典项")?;
        self.dictionary_item_record(id).await?;

        let mut database = self.db.lock().await;
        DictionaryItemRecord::filter(DictionaryItemRecord::fields().id().eq(id))
            .delete()
            .exec(&mut *database)
            .await
            .context("删除字典项失败")?;
        Ok(())
    }

    async fn dictionary_type_record(&self, id: &str) -> anyhow::Result<DictionaryTypeRecord> {
        let mut database = self.db.lock().await;
        Query::<List<DictionaryTypeRecord>>::filter(DictionaryTypeRecord::fields().id().eq(id))
            .first()
            .exec(&mut *database)
            .await
            .context("读取字典类型失败")?
            .ok_or_else(|| anyhow!("字典类型不存在：{id}"))
    }

    async fn dictionary_item_record(&self, id: &str) -> anyhow::Result<DictionaryItemRecord> {
        let mut database = self.db.lock().await;
        Query::<List<DictionaryItemRecord>>::filter(DictionaryItemRecord::fields().id().eq(id))
            .first()
            .exec(&mut *database)
            .await
            .context("读取字典项失败")?
            .ok_or_else(|| anyhow!("字典项不存在：{id}"))
    }

    async fn dictionary_item_count(&self, dictionary_type_id: &str) -> anyhow::Result<usize> {
        let mut database = self.db.lock().await;
        let records = Query::<List<DictionaryItemRecord>>::filter(
            DictionaryItemRecord::fields()
                .dictionary_type_id()
                .eq(dictionary_type_id),
        )
        .exec(&mut *database)
        .await
        .context("读取字典项数量失败")?;
        Ok(records.len())
    }

    async fn ensure_dictionary_type_code_available(
        &self,
        code: &str,
        excluded_id: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut database = self.db.lock().await;
        let records = Query::<List<DictionaryTypeRecord>>::filter(
            DictionaryTypeRecord::fields().code().eq(code),
        )
        .exec(&mut *database)
        .await
        .context("校验字典类型编码失败")?;
        if records
            .iter()
            .any(|record| excluded_id.is_none_or(|id| record.id != id))
        {
            bail!("conflict 字典类型编码已存在：{code}");
        }
        Ok(())
    }

    async fn ensure_dictionary_item_values_available(
        &self,
        input: &DictionaryItemInput,
        excluded_id: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut database = self.db.lock().await;
        let records = Query::<List<DictionaryItemRecord>>::filter(
            DictionaryItemRecord::fields()
                .dictionary_type_id()
                .eq(&input.dictionary_type_id),
        )
        .exec(&mut *database)
        .await
        .context("校验字典项唯一性失败")?;
        for record in records {
            if excluded_id.is_some_and(|id| record.id == id) {
                continue;
            }
            if record.code == input.code {
                bail!("conflict 当前字典下编码已存在：{}", input.code);
            }
            if record.raw_value == input.raw_value {
                bail!("conflict 当前字典下原始值已存在：{}", input.raw_value);
            }
        }
        Ok(())
    }
}

fn normalize_dictionary_type_input(
    input: DictionaryTypeInput,
) -> anyhow::Result<DictionaryTypeInput> {
    let code = normalize_identifier(&input.code, "字典类型编码")?;
    let name = normalize_text(&input.name, "字典类型名称", 80)?;
    let description = normalize_optional_text(&input.description, 500)?;
    let scope = if input.scope.trim().is_empty() {
        "system".to_string()
    } else {
        normalize_identifier(&input.scope, "字典作用域")?
    };
    let raw_value_kind = match input.raw_value_kind.trim() {
        "" | "string" => "string".to_string(),
        "int" => "int".to_string(),
        other => bail!("不支持的字典原始值类型：{other}"),
    };
    let status = normalize_status(&input.status)?;

    Ok(DictionaryTypeInput {
        code,
        name,
        description,
        scope,
        raw_value_kind,
        open_enum: input.open_enum,
        sort_index: input.sort_index,
        status,
    })
}

fn normalize_dictionary_item_input(
    input: DictionaryItemInput,
    raw_value_kind: &str,
) -> anyhow::Result<DictionaryItemInput> {
    let dictionary_type_id = required_id(&input.dictionary_type_id, "字典类型")?.to_string();
    let code = normalize_identifier(&input.code, "字典项编码")?;
    let label = normalize_text(&input.label, "字典项名称", 120)?;
    let description = normalize_optional_text(&input.description, 500)?;
    let raw_value = normalize_raw_value(&input.raw_value, raw_value_kind)?;
    let status = normalize_status(&input.status)?;
    let meta_json = normalize_meta_json(&input.meta_json)?;

    Ok(DictionaryItemInput {
        dictionary_type_id,
        code,
        label,
        description,
        raw_value,
        sort_index: input.sort_index,
        status,
        meta_json,
    })
}

fn normalize_identifier(value: &str, field: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{field}不能为空");
    }
    if value.len() > 100 {
        bail!("{field}不能超过 100 个字符");
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        bail!("{field}不能为空");
    };
    if !first.is_ascii_lowercase() {
        bail!("{field}必须以小写字母开头");
    }
    if chars.any(|character| {
        !character.is_ascii_lowercase() && !character.is_ascii_digit() && character != '_'
    }) {
        bail!("{field}只能包含小写字母、数字和下划线");
    }
    Ok(value.to_string())
}

fn normalize_text(value: &str, field: &str, max_chars: usize) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{field}不能为空");
    }
    if value.chars().count() > max_chars {
        bail!("{field}不能超过 {max_chars} 个字符");
    }
    Ok(value.to_string())
}

fn normalize_optional_text(value: &str, max_chars: usize) -> anyhow::Result<String> {
    let value = value.trim();
    if value.chars().count() > max_chars {
        bail!("描述不能超过 {max_chars} 个字符");
    }
    Ok(value.to_string())
}

fn normalize_status(value: &str) -> anyhow::Result<String> {
    match value.trim() {
        "" | "enabled" => Ok("enabled".to_string()),
        "disabled" => Ok("disabled".to_string()),
        other => bail!("不支持的字典状态：{other}"),
    }
}

fn normalize_raw_value(value: &str, raw_value_kind: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("字典项原始值不能为空");
    }
    match raw_value_kind {
        "string" => Ok(value.to_string()),
        "int" => value
            .parse::<i64>()
            .map(|number| number.to_string())
            .map_err(|error| anyhow!("字典项原始值必须是整数：{error}")),
        other => bail!("不支持的字典原始值类型：{other}"),
    }
}

fn normalize_meta_json(value: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok("{}".to_string());
    }
    let metadata = serde_json::from_str::<Value>(value).context("字典项元数据不是有效 JSON")?;
    serde_json::to_string(&metadata).context("序列化字典项元数据失败")
}

fn required_id<'a>(value: &'a str, resource: &str) -> anyhow::Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{resource} ID 不能为空");
    }
    Ok(value)
}

fn dictionary_item_matches(record: &DictionaryItemRecord, query: &str) -> bool {
    let query = query.to_lowercase();
    record.code.to_lowercase().contains(&query)
        || record.label.to_lowercase().contains(&query)
        || record.raw_value.to_lowercase().contains(&query)
        || record.description.to_lowercase().contains(&query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_input_defaults_to_string_system_dictionary() -> anyhow::Result<()> {
        let input = normalize_dictionary_type_input(DictionaryTypeInput {
            code: "note_type".to_string(),
            name: "笔记类型".to_string(),
            description: String::new(),
            scope: String::new(),
            raw_value_kind: String::new(),
            open_enum: false,
            sort_index: 0,
            status: String::new(),
        })?;

        // 空白可选项必须收敛为统一的系统字典默认值。
        assert_eq!(input.scope, "system");
        assert_eq!(input.raw_value_kind, "string");
        assert_eq!(input.status, "enabled");
        Ok(())
    }

    #[test]
    fn integer_dictionary_normalizes_raw_value() -> anyhow::Result<()> {
        let input = normalize_dictionary_item_input(
            DictionaryItemInput {
                dictionary_type_id: "type-id".to_string(),
                code: "enabled".to_string(),
                label: "启用".to_string(),
                description: String::new(),
                raw_value: "01".to_string(),
                sort_index: 0,
                status: String::new(),
                meta_json: String::new(),
            },
            "int",
        )?;

        // 整数字典规范化后才能可靠执行唯一性校验。
        assert_eq!(input.raw_value, "1");
        assert_eq!(input.meta_json, "{}");
        Ok(())
    }

    #[test]
    fn identifier_rejects_non_snake_case_value() {
        let error = normalize_identifier("HeartbeatLost", "字典项编码");

        // 构建期枚举与运行时字典共享 snake_case 编码约束。
        assert!(error.is_err());
    }
}
