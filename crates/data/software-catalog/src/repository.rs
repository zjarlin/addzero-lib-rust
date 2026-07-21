#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use anyhow::{Context, anyhow};
use az_persistence::context::PersistenceDb;
use jiff::Timestamp;
use serde_json::json;
use toasty::stmt::{List, Query};
use uuid::Uuid;

use crate::{
    model::{
        InstallerKind, SoftwareEntryDto, SoftwareEntryInput, SoftwareInstallMethodDto,
        SoftwarePlatform, normalize_input, parse_uuid, validate_input,
    },
    models::{
        software_entry::SoftwareEntryRecord,
        software_install_method::SoftwareInstallMethodRecord,
    },
};

#[derive(Clone)]
pub(crate) struct SoftwareCatalogRepository {
    db: PersistenceDb,
}

impl SoftwareCatalogRepository {
    pub(crate) fn new(db: PersistenceDb) -> Self {
        Self { db }
    }

    pub(crate) async fn count_entries(&self) -> anyhow::Result<u64> {
        let mut db = self.db.lock().await;
        Query::<List<SoftwareEntryRecord>>::all()
            .count()
            .exec(&mut *db)
            .await
            .context("统计软件目录失败")
    }

    pub(crate) async fn list_entries(&self) -> anyhow::Result<Vec<SoftwareEntryDto>> {
        let mut db = self.db.lock().await;
        let mut entries = Query::<List<SoftwareEntryRecord>>::all()
            .exec(&mut *db)
            .await
            .context("查询软件目录失败")?;
        let mut methods = Query::<List<SoftwareInstallMethodRecord>>::all()
            .exec(&mut *db)
            .await
            .context("查询软件安装方式失败")?;
        entries.sort_by(|left, right| {
            left.title
                .cmp(&right.title)
                .then(left.slug.cmp(&right.slug))
        });
        methods.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then(left.label.cmp(&right.label))
                .then(left.installer_kind.cmp(&right.installer_kind))
        });
        let mut methods_by_software = methods.into_iter().fold(
            BTreeMap::<Uuid, Vec<SoftwareInstallMethodRecord>>::new(),
            |mut grouped, method| {
                grouped.entry(method.software_id).or_default().push(method);
                grouped
            },
        );
        Ok(entries
            .into_iter()
            .map(|entry| {
                let methods = methods_by_software.remove(&entry.id).unwrap_or_default();
                entry_from_records(entry, methods)
            })
            .collect())
    }

    pub(crate) async fn get_entry(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Option<SoftwareEntryDto>> {
        let mut db = self.db.lock().await;
        let entry = Query::<List<SoftwareEntryRecord>>::filter(
            SoftwareEntryRecord::fields().id().eq(id),
        )
        .first()
        .exec(&mut *db)
        .await
        .with_context(|| format!("查询软件条目失败: {id}"))?;
        let Some(entry) = entry else {
            return Ok(None);
        };
        let mut methods = Query::<List<SoftwareInstallMethodRecord>>::filter(
            SoftwareInstallMethodRecord::fields()
                .software_id()
                .eq(id),
        )
        .exec(&mut *db)
        .await
        .with_context(|| format!("查询软件安装方式失败: {id}"))?;
        methods.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then(left.label.cmp(&right.label))
                .then(left.installer_kind.cmp(&right.installer_kind))
        });
        Ok(Some(entry_from_records(entry, methods)))
    }

    pub(crate) async fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> anyhow::Result<SoftwareEntryDto> {
        validate_input(&input)?;
        let preferred_id = input.id.as_deref().map(parse_uuid).transpose()?;
        let entry = normalize_input(input);
        let persisted_id = {
            let mut db = self.db.lock().await;
            let mut transaction = db
                .transaction()
                .await
                .context("开启软件目录保存事务失败")?;
            let persisted_id = match preferred_id {
                Some(id) => id,
                None => Query::<List<SoftwareEntryRecord>>::filter(
                    SoftwareEntryRecord::fields().slug().eq(&entry.slug),
                )
                .first()
                .exec(&mut transaction)
                .await
                .with_context(|| format!("查询软件条目失败: {}", entry.slug))?
                .map(|record| record.id)
                .unwrap_or_else(Uuid::new_v4),
            };
            let now = Timestamp::now();
            let existing = Query::<List<SoftwareEntryRecord>>::filter(
                SoftwareEntryRecord::fields().id().eq(persisted_id),
            )
            .first()
            .exec(&mut transaction)
            .await
            .context("查询待保存软件条目失败")?;
            let trial_platforms = entry
                .trial_platforms
                .iter()
                .map(|platform| platform.code().to_string())
                .collect::<Vec<_>>();
            match existing {
                Some(_) => {
                    SoftwareEntryRecord::filter(
                        SoftwareEntryRecord::fields().id().eq(persisted_id),
                    )
                    .update()
                    .slug(&entry.slug)
                    .title(&entry.title)
                    .vendor(&entry.vendor)
                    .summary(&entry.summary)
                    .homepage_url(&entry.homepage_url)
                    .icon_url(&entry.icon_url)
                    .tags(entry.tags.clone())
                    .trial_platforms(trial_platforms)
                    .raw(json!({}))
                    .updated_at(now)
                    .exec(&mut transaction)
                    .await
                    .with_context(|| format!("更新软件条目失败: {}", entry.slug))?;
                }
                None => {
                    SoftwareEntryRecord::create()
                        .id(persisted_id)
                        .slug(&entry.slug)
                        .title(&entry.title)
                        .vendor(&entry.vendor)
                        .summary(&entry.summary)
                        .homepage_url(&entry.homepage_url)
                        .icon_url(&entry.icon_url)
                        .tags(entry.tags.clone())
                        .trial_platforms(trial_platforms)
                        .raw(json!({}))
                        .created_at(now)
                        .updated_at(now)
                        .exec(&mut transaction)
                        .await
                        .with_context(|| format!("创建软件条目失败: {}", entry.slug))?;
                }
            }
            SoftwareInstallMethodRecord::filter(
                SoftwareInstallMethodRecord::fields()
                    .software_id()
                    .eq(persisted_id),
            )
            .delete()
            .exec(&mut transaction)
            .await
            .with_context(|| format!("删除旧安装方式失败: {persisted_id}"))?;
            let total = entry.methods.len();
            for (index, method) in entry.methods.iter().enumerate() {
                SoftwareInstallMethodRecord::create()
                    .id(parse_uuid(&method.id)?)
                    .software_id(persisted_id)
                    .platform(method.platform.code())
                    .installer_kind(method.kind.code())
                    .label(&method.label)
                    .package_id(&method.package_id)
                    .asset_item_id(method.asset_item_id.clone())
                    .command_text(&method.command)
                    .note(&method.note)
                    .priority(i32::try_from(total.saturating_sub(index)).unwrap_or(i32::MAX))
                    .exec(&mut transaction)
                    .await
                    .with_context(|| format!("创建软件安装方式失败: {}", method.label))?;
            }
            transaction
                .commit()
                .await
                .with_context(|| format!("提交软件条目失败: {}", entry.slug))?;
            persisted_id
        };
        self.get_entry(persisted_id)
            .await?
            .ok_or_else(|| anyhow!("保存后未找到软件条目"))
    }

    pub(crate) async fn delete_entry(&self, id: &str) -> anyhow::Result<()> {
        let id = parse_uuid(id)?;
        let mut db = self.db.lock().await;
        let mut transaction = db
            .transaction()
            .await
            .context("开启软件目录删除事务失败")?;
        SoftwareInstallMethodRecord::filter(
            SoftwareInstallMethodRecord::fields()
                .software_id()
                .eq(id),
        )
        .delete()
        .exec(&mut transaction)
        .await
        .with_context(|| format!("删除软件安装方式失败: {id}"))?;
        SoftwareEntryRecord::filter(SoftwareEntryRecord::fields().id().eq(id))
            .delete()
            .exec(&mut transaction)
            .await
            .with_context(|| format!("删除软件条目失败: {id}"))?;
        transaction
            .commit()
            .await
            .with_context(|| format!("提交软件删除失败: {id}"))?;
        Ok(())
    }
}

fn entry_from_records(
    entry: SoftwareEntryRecord,
    methods: Vec<SoftwareInstallMethodRecord>,
) -> SoftwareEntryDto {
    SoftwareEntryDto {
        id: entry.id.to_string(),
        slug: entry.slug,
        title: entry.title,
        vendor: entry.vendor,
        summary: entry.summary,
        homepage_url: entry.homepage_url,
        icon_url: entry.icon_url,
        trial_platforms: entry
            .trial_platforms
            .0
            .iter()
            .filter_map(|code| SoftwarePlatform::from_code(code))
            .collect(),
        tags: entry.tags.0,
        methods: methods.into_iter().map(method_from_record).collect(),
    }
}

fn method_from_record(record: SoftwareInstallMethodRecord) -> SoftwareInstallMethodDto {
    SoftwareInstallMethodDto {
        id: record.id.to_string(),
        platform: SoftwarePlatform::from_code_or_default(&record.platform),
        kind: InstallerKind::from_code_or_default(&record.installer_kind),
        label: record.label,
        package_id: record.package_id,
        asset_item_id: record.asset_item_id,
        command: record.command_text,
        note: record.note,
    }
}
