//! engine 公共 API。
//!
//! 本模块集中暴露低代码引擎的元数据模型、持久化 store、执行管道和 API 常量。

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use anyhow::{Context, anyhow, bail};
use evalexpr::{ContextWithMutableVariables, HashMapContext, Value as EvalValue};
use rhai::{Dynamic, Engine as RhaiEngine, Scope};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use toasty::stmt::{List, Query};

use crate::validation::text;

/// engine Toasty 表名前缀。
pub const TABLE_NAME_PREFIX: &str = "engine_";

/// engine REST API 前缀。
pub const API_PREFIX: &str = "/api/engine";

/// 模型集合 API 路径。
pub const MODELS_PATH: &str = "/api/engine/models";

/// 字段集合 API 路径模板说明。
pub const FIELDS_PATH_TEMPLATE: &str = "/api/engine/models/{model_name}/fields";

/// 钩子集合 API 路径模板说明。
pub const HOOKS_PATH_TEMPLATE: &str = "/api/engine/models/{model_name}/hooks";

/// 记录集合 API 路径模板说明。
pub const RECORDS_PATH_TEMPLATE: &str = "/api/engine/models/{model_name}/records";

/// 模型列表操作 ID。
pub const OP_MODELS_LIST: &str = "engine.models.list";

/// 模型创建操作 ID。
pub const OP_MODELS_CREATE: &str = "engine.models.create";

/// 模型更新操作 ID。
pub const OP_MODELS_UPDATE: &str = "engine.models.update";

/// 字段列表操作 ID。
pub const OP_FIELDS_LIST: &str = "engine.fields.list";

/// 字段创建操作 ID。
pub const OP_FIELDS_CREATE: &str = "engine.fields.create";

/// 字段更新操作 ID。
pub const OP_FIELDS_UPDATE: &str = "engine.fields.update";

/// 钩子列表操作 ID。
pub const OP_HOOKS_LIST: &str = "engine.hooks.list";

/// 钩子创建操作 ID。
pub const OP_HOOKS_CREATE: &str = "engine.hooks.create";

/// 钩子更新操作 ID。
pub const OP_HOOKS_UPDATE: &str = "engine.hooks.update";

/// 记录列表操作 ID。
pub const OP_RECORDS_LIST: &str = "engine.records.list";

/// 记录创建操作 ID。
pub const OP_RECORDS_CREATE: &str = "engine.records.create";

/// 记录更新操作 ID。
pub const OP_RECORDS_UPDATE: &str = "engine.records.update";

/// 元模型，描述一类动态业务记录。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "engine_meta_models"]
pub struct MetaModel {
    #[key]
    pub id: String,
    #[index]
    pub name: String,
    pub display_name: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// 元字段，描述动态记录 payload 中的一个字段。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "engine_meta_fields"]
pub struct MetaField {
    #[key]
    pub id: String,
    #[index]
    pub model_name: String,
    pub name: String,
    pub display_name: String,
    pub field_type: String,
    pub is_required: bool,
    pub expression: Option<String>,
    pub dependency_json: Option<String>,
    pub order_index: i32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// 生命周期钩子定义。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "engine_hook_definitions"]
pub struct HookDefinition {
    #[key]
    pub id: String,
    #[index]
    pub model_name: String,
    pub trigger_event: String,
    pub script_content: String,
    pub is_active: bool,
    pub order_index: i32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// 动态业务记录，payload 使用 Toasty JSON 文档字段承载。
#[derive(Clone, Debug, PartialEq, toasty::Model)]
#[table = "engine_data_records"]
pub struct DataRecord {
    #[key]
    pub id: String,
    #[index]
    pub model_name: String,
    pub payload: toasty::Json<Value>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// 计算字段的显式跨模型依赖。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComputedDependency {
    pub alias: String,
    pub source_model_name: String,
    pub local_field: String,
    pub source_payload_field: String,
}

/// 页式查询结果，遵循 `d/t/p` 约定。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PageData<T> {
    pub d: Vec<T>,
    pub t: u64,
    pub p: PageParams,
}

/// 页式查询参数，遵循 `o/s` 约定。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PageParams {
    pub o: usize,
    pub s: usize,
}

impl Default for PageParams {
    fn default() -> Self {
        Self { o: 0, s: 50 }
    }
}

/// 对外返回的记录视图。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataRecordView {
    pub id: String,
    pub model_name: String,
    pub payload: Value,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl From<DataRecord> for DataRecordView {
    fn from(record: DataRecord) -> Self {
        Self {
            id: record.id,
            model_name: record.model_name,
            payload: record.payload.0,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

/// 新建模型输入。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInput {
    pub name: String,
    pub display_name: String,
}

/// 新建或更新字段输入。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FieldInput {
    pub name: String,
    pub display_name: String,
    pub field_type: String,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub dependency_json: Option<String>,
    #[serde(default)]
    pub order_index: i32,
}

/// 新建或更新钩子输入。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HookInput {
    pub trigger_event: String,
    pub script_content: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub order_index: i32,
}

/// after 钩子可提交的受控副作用命令。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HookCommand {
    UpdateRecordField {
        model_name: String,
        record_id: String,
        field: String,
        value: Value,
    },
    MergeRecordPayload {
        model_name: String,
        record_id: String,
        patch: Value,
    },
}

/// engine 持久化 store。
#[derive(Clone)]
pub struct EngineStore {
    db: Arc<tokio::sync::Mutex<toasty::Db>>,
}

/// engine 执行器。
#[derive(Clone)]
pub struct EngineExecutor {
    store: EngineStore,
}

/// 集合化计算字段求值器。
#[derive(Clone)]
pub struct BatchComputedEvaluator {
    store: EngineStore,
}

impl EngineStore {
    /// 连接 PostgreSQL 并迁移 engine schema。
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let database_url = verify_database_url(database_url)?;
        let db = toasty::Db::builder()
            .models(engine_models())
            .connect(database_url)
            .await
            .with_context(|| format!("连接 engine 数据库失败: {database_url}"))?;
        ensure_schema(&db).await?;
        Ok(Self::new(db))
    }

    /// Wraps a fully configured Toasty database.
    pub fn new(db: toasty::Db) -> Self {
        Self {
            db: Arc::new(tokio::sync::Mutex::new(db)),
        }
    }

    /// Reuses the application-wide Toasty executor singleton.
    pub fn from_shared_db(db: Arc<tokio::sync::Mutex<toasty::Db>>) -> Self {
        Self { db }
    }

    /// 创建执行器。
    pub fn executor(&self) -> EngineExecutor {
        EngineExecutor {
            store: self.clone(),
        }
    }

    /// 新建模型。
    pub async fn create_model(&self, input: ModelInput) -> anyhow::Result<MetaModel> {
        validate_identifier(&input.name, "模型名")?;
        let now = timestamp_ms();
        let model = MetaModel {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name,
            display_name: input.display_name,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let mut db = self.db.lock().await;
        let created = MetaModel::create()
            .id(&model.id)
            .name(&model.name)
            .display_name(&model.display_name)
            .created_at_ms(model.created_at_ms)
            .updated_at_ms(model.updated_at_ms)
            .exec(&mut *db)
            .await
            .context("创建 engine 模型失败")?;
        Ok(created)
    }

    /// 查询模型列表。
    pub async fn list_models(&self, page: PageParams) -> anyhow::Result<PageData<MetaModel>> {
        let mut db = self.db.lock().await;
        let total = Query::<List<MetaModel>>::all()
            .count()
            .exec(&mut *db)
            .await
            .context("统计 engine 模型失败")?;
        let mut query = Query::<List<MetaModel>>::all();
        query.limit(page.s);
        query.offset(page.o);
        let rows = query.exec(&mut *db).await.context("查询 engine 模型失败")?;
        Ok(PageData {
            d: rows,
            t: total,
            p: page,
        })
    }

    /// 按模型名查询模型。
    pub async fn get_model(&self, model_name: &str) -> anyhow::Result<Option<MetaModel>> {
        let mut db = self.db.lock().await;
        Query::<List<MetaModel>>::filter(MetaModel::fields().name().eq(model_name))
            .first()
            .exec(&mut *db)
            .await
            .context("查询 engine 模型失败")
    }

    /// 更新模型显示信息，模型名作为路由身份不在此处重命名。
    pub async fn update_model(
        &self,
        model_name: &str,
        input: ModelInput,
    ) -> anyhow::Result<MetaModel> {
        validate_identifier(&input.name, "模型名")?;
        if input.name != model_name {
            bail!("模型名不支持通过 update 改名: {model_name} -> {}", input.name);
        }
        let now = timestamp_ms();
        {
            let mut db = self.db.lock().await;
            MetaModel::filter(MetaModel::fields().name().eq(model_name))
                .update()
                .display_name(&input.display_name)
                .updated_at_ms(now)
                .exec(&mut *db)
                .await
                .context("更新 engine 模型失败")?;
        }
        self.get_model(model_name)
            .await?
            .ok_or_else(|| anyhow!("模型不存在: {model_name}"))
    }

    /// 删除模型以及其字段、钩子、记录。
    pub async fn delete_model(&self, model_name: &str) -> anyhow::Result<()> {
        let mut db = self.db.lock().await;
        DataRecord::filter(DataRecord::fields().model_name().eq(model_name))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 engine 模型记录失败")?;
        HookDefinition::filter(HookDefinition::fields().model_name().eq(model_name))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 engine 模型钩子失败")?;
        MetaField::filter(MetaField::fields().model_name().eq(model_name))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 engine 模型字段失败")?;
        MetaModel::filter(MetaModel::fields().name().eq(model_name))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 engine 模型失败")?;
        Ok(())
    }

    /// 创建字段。
    pub async fn create_field(
        &self,
        model_name: &str,
        input: FieldInput,
    ) -> anyhow::Result<MetaField> {
        self.ensure_model(model_name).await?;
        validate_identifier(&input.name, "字段名")?;
        validate_field_type(&input.field_type)?;
        validate_dependency_json(input.dependency_json.as_deref())?;
        let now = timestamp_ms();
        let field = MetaField {
            id: uuid::Uuid::new_v4().to_string(),
            model_name: model_name.to_string(),
            name: input.name,
            display_name: input.display_name,
            field_type: normalize_field_type(&input.field_type),
            is_required: input.is_required,
            expression: empty_to_none(input.expression),
            dependency_json: empty_to_none(input.dependency_json),
            order_index: input.order_index,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let mut db = self.db.lock().await;
        let created = MetaField::create()
            .id(&field.id)
            .model_name(&field.model_name)
            .name(&field.name)
            .display_name(&field.display_name)
            .field_type(&field.field_type)
            .is_required(field.is_required)
            .expression(&field.expression)
            .dependency_json(&field.dependency_json)
            .order_index(field.order_index)
            .created_at_ms(field.created_at_ms)
            .updated_at_ms(field.updated_at_ms)
            .exec(&mut *db)
            .await
            .context("创建 engine 字段失败")?;
        Ok(created)
    }

    /// 查询模型字段。
    pub async fn list_fields(&self, model_name: &str) -> anyhow::Result<Vec<MetaField>> {
        let mut db = self.db.lock().await;
        let mut fields =
            Query::<List<MetaField>>::filter(MetaField::fields().model_name().eq(model_name))
                .exec(&mut *db)
                .await
                .context("查询 engine 字段失败")?;
        fields.sort_by(|left, right| {
            left.order_index
                .cmp(&right.order_index)
                .then(left.name.cmp(&right.name))
        });
        Ok(fields)
    }

    /// 按字段 ID 查询字段。
    pub async fn get_field(&self, field_id: &str) -> anyhow::Result<Option<MetaField>> {
        let mut db = self.db.lock().await;
        Query::<List<MetaField>>::filter(MetaField::fields().id().eq(field_id))
            .first()
            .exec(&mut *db)
            .await
            .context("查询 engine 字段失败")
    }

    /// 更新字段定义。
    pub async fn update_field(
        &self,
        field_id: &str,
        input: FieldInput,
    ) -> anyhow::Result<MetaField> {
        validate_identifier(&input.name, "字段名")?;
        validate_field_type(&input.field_type)?;
        validate_dependency_json(input.dependency_json.as_deref())?;
        let now = timestamp_ms();
        let expression = empty_to_none(input.expression);
        let dependency_json = empty_to_none(input.dependency_json);
        {
            let mut db = self.db.lock().await;
            MetaField::filter(MetaField::fields().id().eq(field_id))
                .update()
                .name(&input.name)
                .display_name(&input.display_name)
                .field_type(normalize_field_type(&input.field_type))
                .is_required(input.is_required)
                .expression(&expression)
                .dependency_json(&dependency_json)
                .order_index(input.order_index)
                .updated_at_ms(now)
                .exec(&mut *db)
                .await
                .context("更新 engine 字段失败")?;
        }
        self.get_field(field_id)
            .await?
            .ok_or_else(|| anyhow!("字段不存在: {field_id}"))
    }

    /// 删除字段。
    pub async fn delete_field(&self, field_id: &str) -> anyhow::Result<()> {
        let mut db = self.db.lock().await;
        MetaField::filter(MetaField::fields().id().eq(field_id))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 engine 字段失败")?;
        Ok(())
    }

    /// 创建钩子。
    pub async fn create_hook(
        &self,
        model_name: &str,
        input: HookInput,
    ) -> anyhow::Result<HookDefinition> {
        self.ensure_model(model_name).await?;
        validate_hook_event(&input.trigger_event)?;
        let now = timestamp_ms();
        let hook = HookDefinition {
            id: uuid::Uuid::new_v4().to_string(),
            model_name: model_name.to_string(),
            trigger_event: input.trigger_event,
            script_content: input.script_content,
            is_active: input.is_active,
            order_index: input.order_index,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let mut db = self.db.lock().await;
        let created = HookDefinition::create()
            .id(&hook.id)
            .model_name(&hook.model_name)
            .trigger_event(&hook.trigger_event)
            .script_content(&hook.script_content)
            .is_active(hook.is_active)
            .order_index(hook.order_index)
            .created_at_ms(hook.created_at_ms)
            .updated_at_ms(hook.updated_at_ms)
            .exec(&mut *db)
            .await
            .context("创建 engine 钩子失败")?;
        Ok(created)
    }

    /// 查询模型钩子。
    pub async fn list_hooks(&self, model_name: &str) -> anyhow::Result<Vec<HookDefinition>> {
        let mut db = self.db.lock().await;
        let mut hooks = Query::<List<HookDefinition>>::filter(
            HookDefinition::fields().model_name().eq(model_name),
        )
        .exec(&mut *db)
        .await
        .context("查询 engine 钩子失败")?;
        hooks.sort_by(|left, right| {
            left.order_index
                .cmp(&right.order_index)
                .then(left.id.cmp(&right.id))
        });
        Ok(hooks)
    }

    /// 按钩子 ID 查询钩子。
    pub async fn get_hook(&self, hook_id: &str) -> anyhow::Result<Option<HookDefinition>> {
        let mut db = self.db.lock().await;
        Query::<List<HookDefinition>>::filter(HookDefinition::fields().id().eq(hook_id))
            .first()
            .exec(&mut *db)
            .await
            .context("查询 engine 钩子失败")
    }

    /// 更新钩子定义。
    pub async fn update_hook(
        &self,
        hook_id: &str,
        input: HookInput,
    ) -> anyhow::Result<HookDefinition> {
        validate_hook_event(&input.trigger_event)?;
        let now = timestamp_ms();
        {
            let mut db = self.db.lock().await;
            HookDefinition::filter(HookDefinition::fields().id().eq(hook_id))
                .update()
                .trigger_event(&input.trigger_event)
                .script_content(&input.script_content)
                .is_active(input.is_active)
                .order_index(input.order_index)
                .updated_at_ms(now)
                .exec(&mut *db)
                .await
                .context("更新 engine 钩子失败")?;
        }
        self.get_hook(hook_id)
            .await?
            .ok_or_else(|| anyhow!("钩子不存在: {hook_id}"))
    }

    /// 删除钩子。
    pub async fn delete_hook(&self, hook_id: &str) -> anyhow::Result<()> {
        let mut db = self.db.lock().await;
        HookDefinition::filter(HookDefinition::fields().id().eq(hook_id))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 engine 钩子失败")?;
        Ok(())
    }

    /// 按模型和记录 ID 查询记录。
    pub async fn get_record(
        &self,
        model_name: &str,
        record_id: &str,
    ) -> anyhow::Result<Option<DataRecord>> {
        let mut db = self.db.lock().await;
        Query::<List<DataRecord>>::filter(
            DataRecord::fields()
                .model_name()
                .eq(model_name)
                .and(DataRecord::fields().id().eq(record_id)),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("查询 engine 记录失败")
    }

    /// 查询模型原始记录，不执行计算字段。
    pub async fn list_raw_records(&self, model_name: &str) -> anyhow::Result<Vec<DataRecord>> {
        let mut db = self.db.lock().await;
        Query::<List<DataRecord>>::filter(DataRecord::fields().model_name().eq(model_name))
            .exec(&mut *db)
            .await
            .context("查询 engine 原始记录失败")
    }

    /// 分页查询模型原始记录，不执行计算字段。
    pub async fn list_raw_records_page(
        &self,
        model_name: &str,
        page: PageParams,
    ) -> anyhow::Result<PageData<DataRecord>> {
        let mut db = self.db.lock().await;
        let base =
            Query::<List<DataRecord>>::filter(DataRecord::fields().model_name().eq(model_name));
        let total = base
            .clone()
            .count()
            .exec(&mut *db)
            .await
            .context("统计 engine 记录失败")?;
        let mut query = base;
        query.limit(page.s);
        query.offset(page.o);
        let records = query.exec(&mut *db).await.context("分页查询 engine 记录失败")?;
        Ok(PageData {
            d: records,
            t: total,
            p: page,
        })
    }

    async fn ensure_model(&self, model_name: &str) -> anyhow::Result<MetaModel> {
        self.get_model(model_name)
            .await?
            .ok_or_else(|| anyhow!("模型不存在: {model_name}"))
    }

    async fn persist_record(
        &self,
        model_name: &str,
        payload: Value,
    ) -> anyhow::Result<DataRecord> {
        let now = timestamp_ms();
        let id = uuid::Uuid::new_v4().to_string();
        let mut db = self.db.lock().await;
        let record = DataRecord::create()
            .id(&id)
            .model_name(model_name)
            .payload(payload)
            .created_at_ms(now)
            .updated_at_ms(now)
            .exec(&mut *db)
            .await
            .context("创建 engine 记录失败")?;
        Ok(record)
    }

    async fn replace_record_payload(
        &self,
        model_name: &str,
        record_id: &str,
        payload: Value,
    ) -> anyhow::Result<DataRecord> {
        let now = timestamp_ms();
        {
            let mut db = self.db.lock().await;
            DataRecord::filter(
                DataRecord::fields()
                    .model_name()
                    .eq(model_name)
                    .and(DataRecord::fields().id().eq(record_id)),
            )
            .update()
            .payload(payload)
            .updated_at_ms(now)
            .exec(&mut *db)
            .await
            .context("更新 engine 记录失败")?;
        }
        self.get_record(model_name, record_id)
            .await?
            .ok_or_else(|| anyhow!("记录不存在: {model_name}/{record_id}"))
    }

    async fn delete_record(&self, model_name: &str, record_id: &str) -> anyhow::Result<()> {
        let mut db = self.db.lock().await;
        DataRecord::filter(
            DataRecord::fields()
                .model_name()
                .eq(model_name)
                .and(DataRecord::fields().id().eq(record_id)),
        )
        .delete()
        .exec(&mut *db)
        .await
        .context("删除 engine 记录失败")?;
        Ok(())
    }
}

impl EngineExecutor {
    /// 插入记录并执行完整写入管道。
    pub async fn insert_record(
        &self,
        model_name: &str,
        raw_payload: Value,
    ) -> anyhow::Result<DataRecordView> {
        let fields = self.store.list_fields(model_name).await?;
        let hooks = self.active_hooks(model_name, "before_insert").await?;
        let mut payload = value_to_object(raw_payload)?;
        validate_payload(&fields, &payload, false)?;
        payload = self.run_before_hooks(hooks, payload).await?;
        let mut values = vec![Value::Object(payload)];
        BatchComputedEvaluator::new(self.store.clone())
            .evaluate(model_name, &fields, &mut values)
            .await?;
        let payload = value_to_object(values.remove(0))?;
        validate_payload(&fields, &payload, true)?;
        let record = self
            .store
            .persist_record(model_name, Value::Object(payload.clone()))
            .await?;
        let after_hooks = self.active_hooks(model_name, "after_insert").await?;
        self.run_after_hooks(after_hooks, model_name, &record.id, Value::Object(payload))
            .await;
        Ok(record.into())
    }

    /// 更新记录并执行完整写入管道。
    pub async fn update_record(
        &self,
        model_name: &str,
        record_id: &str,
        raw_payload: Value,
    ) -> anyhow::Result<DataRecordView> {
        let existing = self
            .store
            .get_record(model_name, record_id)
            .await?
            .ok_or_else(|| anyhow!("记录不存在: {model_name}/{record_id}"))?;
        let fields = self.store.list_fields(model_name).await?;
        let mut payload = value_to_object(existing.payload.0)?;
        for (key, value) in value_to_object(raw_payload)? {
            payload.insert(key, value);
        }
        validate_payload(&fields, &payload, false)?;
        let hooks = self.active_hooks(model_name, "before_update").await?;
        payload = self.run_before_hooks(hooks, payload).await?;
        let mut values = vec![Value::Object(payload)];
        BatchComputedEvaluator::new(self.store.clone())
            .evaluate(model_name, &fields, &mut values)
            .await?;
        let payload = value_to_object(values.remove(0))?;
        validate_payload(&fields, &payload, true)?;
        let record = self
            .store
            .replace_record_payload(model_name, record_id, Value::Object(payload.clone()))
            .await?;
        let after_hooks = self.active_hooks(model_name, "after_update").await?;
        self.run_after_hooks(after_hooks, model_name, record_id, Value::Object(payload))
            .await;
        Ok(record.into())
    }

    /// 查询单条记录并执行 computed 字段求值。
    pub async fn get_record(
        &self,
        model_name: &str,
        record_id: &str,
    ) -> anyhow::Result<DataRecordView> {
        self.store.ensure_model(model_name).await?;
        let fields = self.store.list_fields(model_name).await?;
        let record = self
            .store
            .get_record(model_name, record_id)
            .await?
            .ok_or_else(|| anyhow!("记录不存在: {model_name}/{record_id}"))?;
        let mut payloads = vec![record.payload.0.clone()];
        BatchComputedEvaluator::new(self.store.clone())
            .evaluate(model_name, &fields, &mut payloads)
            .await?;
        let payload = payloads.remove(0);
        Ok(DataRecordView {
            id: record.id,
            model_name: record.model_name,
            payload,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        })
    }

    /// 删除记录。
    pub async fn delete_record(&self, model_name: &str, record_id: &str) -> anyhow::Result<()> {
        self.store.delete_record(model_name, record_id).await
    }

    /// 查询记录并执行集合化计算字段。
    pub async fn list_records(
        &self,
        model_name: &str,
        page: PageParams,
    ) -> anyhow::Result<PageData<DataRecordView>> {
        self.store.ensure_model(model_name).await?;
        let fields = self.store.list_fields(model_name).await?;
        let raw = self.store.list_raw_records_page(model_name, page).await?;
        let mut payloads = raw
            .d
            .iter()
            .map(|record| record.payload.0.clone())
            .collect::<Vec<_>>();
        BatchComputedEvaluator::new(self.store.clone())
            .evaluate(model_name, &fields, &mut payloads)
            .await?;
        let rows = raw
            .d
            .into_iter()
            .zip(payloads.into_iter())
            .map(|(record, payload)| DataRecordView {
                id: record.id,
                model_name: record.model_name,
                payload,
                created_at_ms: record.created_at_ms,
                updated_at_ms: record.updated_at_ms,
            })
            .collect::<Vec<_>>();
        Ok(PageData {
            d: rows,
            t: raw.t,
            p: raw.p,
        })
    }

    async fn active_hooks(
        &self,
        model_name: &str,
        event: &str,
    ) -> anyhow::Result<Vec<HookDefinition>> {
        let hooks = self.store.list_hooks(model_name).await?;
        Ok(hooks
            .into_iter()
            .filter(|hook| hook.is_active && hook.trigger_event == event)
            .collect())
    }

    async fn run_before_hooks(
        &self,
        hooks: Vec<HookDefinition>,
        payload: Map<String, Value>,
    ) -> anyhow::Result<Map<String, Value>> {
        let mut current = Value::Object(payload);
        for hook in hooks {
            current = run_before_script(&hook.script_content, current)
                .with_context(|| format!("执行 before 钩子失败: {}", hook.id))?;
        }
        value_to_object(current)
    }

    async fn run_after_hooks(
        &self,
        hooks: Vec<HookDefinition>,
        model_name: &str,
        record_id: &str,
        payload: Value,
    ) {
        for hook in hooks {
            match run_after_script(&hook.script_content, model_name, record_id, payload.clone()) {
                Ok(commands) => {
                    for command in commands {
                        if let Err(error) = self.apply_hook_command(command).await {
                            tracing::warn!(error = %error, hook_id = %hook.id, "engine after 钩子命令失败");
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(error = %error, hook_id = %hook.id, "engine after 钩子执行失败");
                }
            }
        }
    }

    async fn apply_hook_command(&self, command: HookCommand) -> anyhow::Result<()> {
        match command {
            HookCommand::UpdateRecordField {
                model_name,
                record_id,
                field,
                value,
            } => {
                let record = self
                    .store
                    .get_record(&model_name, &record_id)
                    .await?
                    .ok_or_else(|| anyhow!("副作用记录不存在: {model_name}/{record_id}"))?;
                let mut payload = value_to_object(record.payload.0)?;
                payload.insert(field, value);
                self.store
                    .replace_record_payload(&model_name, &record_id, Value::Object(payload))
                    .await?;
            }
            HookCommand::MergeRecordPayload {
                model_name,
                record_id,
                patch,
            } => {
                let record = self
                    .store
                    .get_record(&model_name, &record_id)
                    .await?
                    .ok_or_else(|| anyhow!("副作用记录不存在: {model_name}/{record_id}"))?;
                let mut payload = value_to_object(record.payload.0)?;
                let patch = value_to_object(patch)?;
                for (key, value) in patch {
                    payload.insert(key, value);
                }
                self.store
                    .replace_record_payload(&model_name, &record_id, Value::Object(payload))
                    .await?;
            }
        }
        Ok(())
    }
}

impl BatchComputedEvaluator {
    /// 创建集合化计算字段求值器。
    pub fn new(store: EngineStore) -> Self {
        Self { store }
    }

    /// 对一批 payload 执行 computed 字段求值。
    pub async fn evaluate(
        &self,
        model_name: &str,
        fields: &[MetaField],
        records: &mut [Value],
    ) -> anyhow::Result<()> {
        if records.is_empty() {
            return Ok(());
        }
        let computed_fields = fields
            .iter()
            .filter(|field| field.field_type == "computed")
            .collect::<Vec<_>>();
        if computed_fields.is_empty() {
            return Ok(());
        }
        let dependencies = self
            .load_dependency_cache(model_name, &computed_fields, records)
            .await?;
        for (record_index, record) in records.iter_mut().enumerate() {
            let payload = record
                .as_object_mut()
                .ok_or_else(|| anyhow!("计算字段只能处理 JSON 对象记录"))?;
            for field in &computed_fields {
                let Some(expression) = field.expression.as_deref() else {
                    continue;
                };
                let deps = match dependencies
                    .get(&field.id)
                    .and_then(|items| items.get(record_index))
                {
                    Some(value) => value.clone(),
                    None => HashMap::new(),
                };
                let value = evaluate_expression(expression, payload, &deps)
                    .with_context(|| format!("计算字段失败: {}.{}", model_name, field.name))?;
                payload.insert(field.name.clone(), eval_value_to_json(value));
            }
        }
        Ok(())
    }

    async fn load_dependency_cache(
        &self,
        _model_name: &str,
        computed_fields: &[&MetaField],
        records: &[Value],
    ) -> anyhow::Result<HashMap<String, Vec<HashMap<String, Value>>>> {
        let mut output = HashMap::new();
        for field in computed_fields {
            let deps = parse_dependencies(field.dependency_json.as_deref())?;
            if deps.is_empty() {
                continue;
            }
            let mut aliases_by_record = vec![HashMap::new(); records.len()];
            for dep in deps {
                let cache = self.load_one_dependency(&dep, records).await?;
                for (record_index, record) in records.iter().enumerate() {
                    let Some(object) = record.as_object() else {
                        continue;
                    };
                    let Some(record_id) = object.get(&dep.local_field).and_then(Value::as_str) else {
                        continue;
                    };
                    if let Some(source_payload) = cache.get(record_id)
                        && let Some(value) = source_payload.get(&dep.source_payload_field)
                    {
                        aliases_by_record[record_index].insert(dep.alias.clone(), value.clone());
                    }
                }
            }
            output.insert(field.id.clone(), aliases_by_record);
        }
        Ok(output)
    }

    async fn load_one_dependency(
        &self,
        dependency: &ComputedDependency,
        records: &[Value],
    ) -> anyhow::Result<HashMap<String, Map<String, Value>>> {
        let ids = records
            .iter()
            .filter_map(Value::as_object)
            .filter_map(|object| object.get(&dependency.local_field))
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let wanted = ids.into_iter().collect::<Vec<_>>();
        let mut db = self.store.db.lock().await;
        let rows = Query::<List<DataRecord>>::filter(
            DataRecord::fields()
                .model_name()
                .eq(&dependency.source_model_name)
                .and(DataRecord::fields().id().in_list(wanted)),
        )
        .exec(&mut *db)
        .await
        .with_context(|| {
            format!(
                "批量加载 computed 依赖失败: {}",
                dependency.source_model_name
            )
        })?;
        let mut cache = HashMap::new();
        for row in rows {
            let payload = value_to_object(row.payload.0)?;
            cache.insert(row.id, payload);
        }
        Ok(cache)
    }
}

/// 创建 API 成功响应。
pub fn ok_response<T: Serialize>(data: T) -> Value {
    json!({ "code": 200, "data": data })
}

/// 创建 API 错误响应。
pub fn error_response(code: u16, message: impl Into<String>) -> Value {
    json!({ "code": code, "msg": message.into() })
}

/// 生成当前 Unix 毫秒时间戳。
pub fn timestamp_ms() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as i64,
        Err(_) => 0,
    }
}

/// Returns the lowcode engine Toasty model set.
pub fn engine_models() -> toasty::ModelSet {
    toasty::models!(MetaModel, MetaField, HookDefinition, DataRecord)
}

/// Bootstrap DDL used only when Toasty reports existing PostgreSQL relations during startup.
pub const ENGINE_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS engine_meta_models (id TEXT PRIMARY KEY, name TEXT NOT NULL, display_name TEXT NOT NULL, created_at_ms BIGINT NOT NULL, updated_at_ms BIGINT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS engine_meta_models_name_idx ON engine_meta_models (name)",
    "CREATE TABLE IF NOT EXISTS engine_meta_fields (id TEXT PRIMARY KEY, model_name TEXT NOT NULL, name TEXT NOT NULL, display_name TEXT NOT NULL, field_type TEXT NOT NULL, is_required BOOLEAN NOT NULL, expression TEXT NULL, dependency_json TEXT NULL, order_index INTEGER NOT NULL, created_at_ms BIGINT NOT NULL, updated_at_ms BIGINT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS engine_meta_fields_model_name_idx ON engine_meta_fields (model_name)",
    "CREATE TABLE IF NOT EXISTS engine_hook_definitions (id TEXT PRIMARY KEY, model_name TEXT NOT NULL, trigger_event TEXT NOT NULL, script_content TEXT NOT NULL, is_active BOOLEAN NOT NULL, order_index INTEGER NOT NULL, created_at_ms BIGINT NOT NULL, updated_at_ms BIGINT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS engine_hook_definitions_model_name_idx ON engine_hook_definitions (model_name)",
    "CREATE TABLE IF NOT EXISTS engine_data_records (id TEXT PRIMARY KEY, model_name TEXT NOT NULL, payload TEXT NOT NULL, created_at_ms BIGINT NOT NULL, updated_at_ms BIGINT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS engine_data_records_model_name_idx ON engine_data_records (model_name)",
];

/// 校验数据库连接串。
pub fn verify_database_url(value: &str) -> anyhow::Result<&str> {
    text(value)
        .trim()
        .not_blank("engine 需要 PostgreSQL DATABASE_URL")?
        .starts_with_any(
            &["postgres://", "postgresql://"],
            "engine 只支持 PostgreSQL DATABASE_URL",
        )
        .map(|value| value.value())
}

/// 幂等确认 engine 的 Toasty schema 已经可用。
async fn ensure_schema(db: &toasty::Db) -> anyhow::Result<()> {
    match db.push_schema().await {
        Ok(()) => Ok(()),
        Err(error) if schema_already_exists(&error) => verify_existing_schema(db).await,
        Err(error) => Err(error).context("迁移 engine schema 失败"),
    }
}

/// Toasty PostgreSQL driver 当前会在表已存在时返回错误；这里只识别该幂等场景。
fn schema_already_exists(error: &toasty::Error) -> bool {
    format!("{error:?}").contains("already exists")
}

/// 表已存在时，通过真实 Toasty 查询确认四张核心表都可读。
async fn verify_existing_schema(db: &toasty::Db) -> anyhow::Result<()> {
    let mut db = db.clone();
    let mut models = Query::<List<MetaModel>>::all();
    models.limit(1);
    models
        .exec(&mut db)
        .await
        .context("校验 engine_meta_models 表失败")?;

    let mut fields = Query::<List<MetaField>>::all();
    fields.limit(1);
    fields
        .exec(&mut db)
        .await
        .context("校验 engine_meta_fields 表失败")?;

    let mut hooks = Query::<List<HookDefinition>>::all();
    hooks.limit(1);
    hooks
        .exec(&mut db)
        .await
        .context("校验 engine_hook_definitions 表失败")?;

    let mut records = Query::<List<DataRecord>>::all();
    records.limit(1);
    records
        .exec(&mut db)
        .await
        .context("校验 engine_data_records 表失败")?;
    Ok(())
}

fn default_true() -> bool {
    true
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let item = item.trim().to_string();
        if item.is_empty() {
            None
        } else {
            Some(item)
        }
    })
}

fn validate_identifier(value: &str, label: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        bail!("{label}不能为空");
    }
    let valid = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
    if !valid {
        bail!("{label}只能包含 ASCII 字母、数字和下划线: {value}");
    }
    Ok(())
}

fn validate_field_type(value: &str) -> anyhow::Result<()> {
    match normalize_field_type(value).as_str() {
        "string" | "int" | "decimal" | "boolean" | "datetime" | "json" | "computed" => Ok(()),
        _ => bail!("不支持的字段类型: {value}"),
    }
}

fn normalize_field_type(value: &str) -> String {
    match value {
        "String" => "string",
        "Integer" => "int",
        "Float" => "decimal",
        "Boolean" => "boolean",
        "DateTime" => "datetime",
        "Json" => "json",
        "Computed" => "computed",
        other => other,
    }
    .to_ascii_lowercase()
}

fn validate_hook_event(value: &str) -> anyhow::Result<()> {
    match value {
        "before_insert" | "after_insert" | "before_update" | "after_update" => Ok(()),
        _ => bail!("不支持的钩子事件: {value}"),
    }
}

fn validate_dependency_json(value: Option<&str>) -> anyhow::Result<()> {
    let _ = parse_dependencies(value)?;
    Ok(())
}

fn parse_dependencies(value: Option<&str>) -> anyhow::Result<Vec<ComputedDependency>> {
    let Some(value) = value.map(str::trim).filter(|item| !item.is_empty()) else {
        return Ok(Vec::new());
    };
    serde_json::from_str(value).context("解析 computed dependency_json 失败")
}

fn validate_payload(
    fields: &[MetaField],
    payload: &Map<String, Value>,
    include_computed: bool,
) -> anyhow::Result<()> {
    for field in fields {
        if field.field_type == "computed" && !include_computed {
            continue;
        }
        if field.is_required && !payload.contains_key(&field.name) {
            bail!("缺少必填字段: {}", field.name);
        }
        if let Some(value) = payload.get(&field.name) {
            validate_json_type(field, value)?;
        }
    }
    Ok(())
}

fn validate_json_type(field: &MetaField, value: &Value) -> anyhow::Result<()> {
    if value.is_null() {
        if field.is_required {
            bail!("字段不能为空: {}", field.name);
        }
        return Ok(());
    }
    let matched = match field.field_type.as_str() {
        "string" => value.is_string(),
        "int" | "datetime" => value.is_i64() || value.is_u64(),
        "decimal" => value.is_number(),
        "boolean" => value.is_boolean(),
        "json" | "computed" => true,
        _ => false,
    };
    if !matched {
        bail!("字段类型不匹配: {}", field.name);
    }
    Ok(())
}

fn value_to_object(value: Value) -> anyhow::Result<Map<String, Value>> {
    value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow!("payload 必须是 JSON 对象"))
}

fn evaluate_expression(
    expression: &str,
    payload: &Map<String, Value>,
    dependencies: &HashMap<String, Value>,
) -> anyhow::Result<EvalValue> {
    let mut context = HashMapContext::new();
    for (key, value) in payload {
        set_eval_value(&mut context, key, value)?;
    }
    for (key, value) in dependencies {
        set_eval_value(&mut context, key, value)?;
    }
    evalexpr::eval_with_context_mut(expression, &mut context)
        .map_err(|error| anyhow!("表达式求值失败: {error}"))
}

fn set_eval_value(context: &mut HashMapContext, key: &str, value: &Value) -> anyhow::Result<()> {
    let eval_value = match value {
        Value::Bool(value) => EvalValue::Boolean(*value),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                EvalValue::Int(value)
            } else if let Some(value) = value.as_f64() {
                EvalValue::Float(value)
            } else {
                return Ok(());
            }
        }
        Value::String(value) => EvalValue::String(value.clone()),
        _ => return Ok(()),
    };
    context
        .set_value(key.to_string(), eval_value)
        .map_err(|error| anyhow!("注入表达式变量失败: {key}: {error}"))
}

fn eval_value_to_json(value: EvalValue) -> Value {
    match value {
        EvalValue::String(value) => Value::String(value),
        EvalValue::Float(value) => match serde_json::Number::from_f64(value) {
            Some(number) => Value::Number(number),
            None => Value::Null,
        },
        EvalValue::Int(value) => Value::Number(value.into()),
        EvalValue::Boolean(value) => Value::Bool(value),
        EvalValue::Tuple(values) => Value::Array(values.into_iter().map(eval_value_to_json).collect()),
        EvalValue::Empty => Value::Null,
    }
}

fn run_before_script(source: &str, payload: Value) -> anyhow::Result<Value> {
    let mut engine = RhaiEngine::new();
    engine.set_max_operations(1_000_000);
    let mut scope = Scope::new();
    let dynamic = serde_json::from_value::<Dynamic>(payload).context("注入 before payload 失败")?;
    scope.push_dynamic("payload", dynamic);
    let _ = engine
        .eval_with_scope::<Dynamic>(&mut scope, source)
        .map_err(|error| anyhow!("Rhai before 脚本失败: {error}"))?;
    let dynamic = scope
        .get_value::<Dynamic>("payload")
        .ok_or_else(|| anyhow!("before 脚本未返回 payload"))?;
    serde_json::to_value(dynamic).context("导出 before payload 失败")
}

fn run_after_script(
    source: &str,
    model_name: &str,
    record_id: &str,
    payload: Value,
) -> anyhow::Result<Vec<HookCommand>> {
    let commands = Arc::new(std::sync::Mutex::new(Vec::<HookCommand>::new()));
    let mut engine = RhaiEngine::new();
    engine.set_max_operations(1_000_000);
    {
        let commands = commands.clone();
        engine.register_fn(
            "update_record_field",
            move |model_name: String, record_id: String, field: String, value: Dynamic| {
                if let Ok(value) = serde_json::to_value(value)
                    && let Ok(mut commands) = commands.lock()
                {
                    commands.push(HookCommand::UpdateRecordField {
                        model_name,
                        record_id,
                        field,
                        value,
                    });
                }
            },
        );
    }
    {
        let commands = commands.clone();
        engine.register_fn(
            "merge_record_payload",
            move |model_name: String, record_id: String, patch: Dynamic| {
                if let Ok(patch) = serde_json::to_value(patch)
                    && let Ok(mut commands) = commands.lock()
                {
                    commands.push(HookCommand::MergeRecordPayload {
                        model_name,
                        record_id,
                        patch,
                    });
                }
            },
        );
    }
    let mut scope = Scope::new();
    scope.push("model_name", model_name.to_string());
    scope.push("record_id", record_id.to_string());
    let payload = serde_json::from_value::<Dynamic>(payload).context("注入 after payload 失败")?;
    scope.push_dynamic("payload", payload);
    let _ = engine
        .eval_with_scope::<Dynamic>(&mut scope, source)
        .map_err(|error| anyhow!("Rhai after 脚本失败: {error}"))?;
    let commands = commands
        .lock()
        .map_err(|_| anyhow!("after 钩子命令队列锁失败"))?
        .clone();
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn computed_field(name: &str, expression: &str) -> MetaField {
        MetaField {
            id: name.to_string(),
            model_name: "order".to_string(),
            name: name.to_string(),
            display_name: name.to_string(),
            field_type: "computed".to_string(),
            is_required: false,
            expression: Some(expression.to_string()),
            dependency_json: None,
            order_index: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    #[test]
    fn validates_required_fields() {
        let field = MetaField {
            id: "amount".to_string(),
            model_name: "order".to_string(),
            name: "amount".to_string(),
            display_name: "金额".to_string(),
            field_type: "int".to_string(),
            is_required: true,
            expression: None,
            dependency_json: None,
            order_index: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        };
        let payload = Map::new();

        // 必填字段缺失时应在落库前阻断。
        assert!(validate_payload(&[field], &payload, false).is_err());
    }

    #[test]
    fn validates_postgresql_database_url_with_built_ins() {
        let url = match verify_database_url("  postgresql://engine  ") {
            Ok(url) => url,
            Err(error) => panic!("PostgreSQL DATABASE_URL 应通过校验: {error}"),
        };

        // 校验链应返回去除首尾空白后的正式连接串。
        assert_eq!(url, "postgresql://engine");
        // 非 PostgreSQL 协议必须被 starts_with_any 内置阻断。
        assert!(verify_database_url("mysql://engine").is_err());
    }

    #[test]
    fn evaluates_row_computed_expression() {
        let mut payload = Map::new();
        payload.insert("amount".to_string(), json!(100));
        let value = match evaluate_expression("amount * 2", &payload, &HashMap::new()) {
            Ok(value) => value,
            Err(error) => panic!("表达式应求值成功: {error}"),
        };

        // 本行字段应直接注入表达式上下文。
        assert_eq!(eval_value_to_json(value), json!(200));
    }

    #[test]
    fn before_hook_can_mutate_payload() {
        let payload = json!({ "amount": 9 });
        let result = match run_before_script(
            r#"
                payload["amount"] = 10;
                payload
            "#,
            payload,
        ) {
            Ok(value) => value,
            Err(error) => panic!("before 钩子应执行成功: {error}"),
        };

        // before 钩子允许修改 payload，但不暴露数据库能力。
        assert_eq!(result["amount"], json!(10));
    }

    #[test]
    fn before_hook_can_block_payload() {
        let payload = json!({ "amount": -1 });
        let result = run_before_script(r#"throw "amount 必须大于 0";"#, payload);

        // before 钩子可通过 throw 阻断写入管道。
        assert!(result.is_err());
    }

    #[test]
    fn after_hook_collects_whitelisted_command() {
        let commands = match run_after_script(
            r#"
                update_record_field("user", "u1", "vip", true);
            "#,
            "order",
            "o1",
            json!({ "amount": 100 }),
        ) {
            Ok(value) => value,
            Err(error) => panic!("after 钩子应执行成功: {error}"),
        };

        // after 钩子只能提交白名单命令，不能拿到裸数据库。
        assert_eq!(
            commands,
            vec![HookCommand::UpdateRecordField {
                model_name: "user".to_string(),
                record_id: "u1".to_string(),
                field: "vip".to_string(),
                value: json!(true),
            }]
        );
    }

    #[test]
    fn parses_explicit_dependencies() {
        let dependencies = match parse_dependencies(Some(
            r#"[{"alias":"user_vip_level","source_model_name":"user","local_field":"user_id","source_payload_field":"vip_level"}]"#,
        )) {
            Ok(value) => value,
            Err(error) => panic!("依赖配置应解析成功: {error}"),
        };

        // 跨模型计算依赖必须显式配置，避免运行时猜表达式变量。
        assert_eq!(dependencies[0].alias, "user_vip_level");
    }

    #[test]
    fn computed_field_helper_uses_new_field_type() {
        let field = computed_field("bonus", "amount * 2");

        // computed 是新引擎字段类型，不再使用旧渲染配置。
        assert_eq!(field.field_type, "computed");
    }
}
