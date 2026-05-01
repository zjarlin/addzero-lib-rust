use std::collections::BTreeMap;

use chrono::Utc;
use sea_orm::{
    ActiveValue::NotSet, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait, sea_query::OnConflict,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entity::{software_entry, software_install_method},
    model::{
        InstallerKind, SoftwareCatalogError, SoftwareCatalogResult, SoftwareEntryDto,
        SoftwareEntryInput, SoftwareInstallMethodDto, SoftwarePlatform, normalize_input,
        parse_uuid, validate_input,
    },
};

#[derive(Clone)]
pub(crate) struct SoftwareCatalogRepository {
    db: DatabaseConnection,
}

impl SoftwareCatalogRepository {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub(crate) async fn count_entries(&self) -> SoftwareCatalogResult<u64> {
        software_entry::Entity::find()
            .count(&self.db)
            .await
            .map_err(SoftwareCatalogError::query)
    }

    pub(crate) async fn list_entries(&self) -> SoftwareCatalogResult<Vec<SoftwareEntryDto>> {
        let entries = software_entry::Entity::find()
            .order_by_asc(software_entry::Column::Title)
            .order_by_asc(software_entry::Column::Slug)
            .all(&self.db)
            .await
            .map_err(SoftwareCatalogError::query)?;
        let methods = software_install_method::Entity::find()
            .order_by_desc(software_install_method::Column::Priority)
            .order_by_asc(software_install_method::Column::Label)
            .order_by_asc(software_install_method::Column::InstallerKind)
            .all(&self.db)
            .await
            .map_err(SoftwareCatalogError::query)?;

        let mut methods_by_software = methods.into_iter().fold(
            BTreeMap::<Uuid, Vec<software_install_method::Model>>::new(),
            |mut acc, method| {
                acc.entry(method.software_id).or_default().push(method);
                acc
            },
        );

        Ok(entries
            .into_iter()
            .map(|entry| {
                let methods = methods_by_software.remove(&entry.id).unwrap_or_default();
                entry_from_models(entry, methods)
            })
            .collect())
    }

    pub(crate) async fn get_entry(
        &self,
        id: Uuid,
    ) -> SoftwareCatalogResult<Option<SoftwareEntryDto>> {
        let Some(entry) = software_entry::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(SoftwareCatalogError::query)?
        else {
            return Ok(None);
        };
        let methods = self.list_methods_for(&self.db, id).await?;
        Ok(Some(entry_from_models(entry, methods)))
    }

    pub(crate) async fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> SoftwareCatalogResult<SoftwareEntryDto> {
        validate_input(&input)?;
        let preferred_id = input.id.clone();
        let entry = normalize_input(input);
        let tx = self.db.begin().await.map_err(SoftwareCatalogError::query)?;
        let persisted_id = self
            .resolve_entry_id(
                &tx,
                &entry.slug,
                preferred_id.as_deref().map(parse_uuid).transpose()?,
            )
            .await?;

        let active = software_entry::ActiveModel {
            id: Set(persisted_id),
            slug: Set(entry.slug.clone()),
            title: Set(entry.title.clone()),
            vendor: Set(entry.vendor.clone()),
            summary: Set(entry.summary.clone()),
            homepage_url: Set(entry.homepage_url.clone()),
            icon_url: Set(entry.icon_url.clone()),
            tags: Set(json!(entry.tags)),
            trial_platforms: Set(json!(
                entry
                    .trial_platforms
                    .iter()
                    .map(|platform| platform.code())
                    .collect::<Vec<_>>()
            )),
            raw: Set(json!({})),
            created_at: NotSet,
            updated_at: Set(Utc::now()),
        };

        software_entry::Entity::insert(active)
            .on_conflict(
                OnConflict::column(software_entry::Column::Id)
                    .update_columns([
                        software_entry::Column::Slug,
                        software_entry::Column::Title,
                        software_entry::Column::Vendor,
                        software_entry::Column::Summary,
                        software_entry::Column::HomepageUrl,
                        software_entry::Column::IconUrl,
                        software_entry::Column::Tags,
                        software_entry::Column::TrialPlatforms,
                        software_entry::Column::Raw,
                        software_entry::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&tx)
            .await
            .map_err(SoftwareCatalogError::query)?;

        self.replace_methods(&tx, persisted_id, &entry.methods)
            .await?;
        tx.commit().await.map_err(SoftwareCatalogError::query)?;

        self.get_entry(persisted_id)
            .await?
            .ok_or_else(|| SoftwareCatalogError::Message("保存后未找到软件条目".to_string()))
    }

    pub(crate) async fn delete_entry(&self, id: &str) -> SoftwareCatalogResult<()> {
        software_entry::Entity::delete_many()
            .filter(software_entry::Column::Id.eq(parse_uuid(id)?))
            .exec(&self.db)
            .await
            .map_err(SoftwareCatalogError::query)?;
        Ok(())
    }

    async fn resolve_entry_id<C>(
        &self,
        db: &C,
        slug: &str,
        preferred: Option<Uuid>,
    ) -> SoftwareCatalogResult<Uuid>
    where
        C: ConnectionTrait,
    {
        if let Some(id) = preferred {
            return Ok(id);
        }

        let existing = software_entry::Entity::find()
            .filter(software_entry::Column::Slug.eq(slug.to_string()))
            .one(db)
            .await
            .map_err(SoftwareCatalogError::query)?;

        Ok(existing.map(|entry| entry.id).unwrap_or_else(Uuid::new_v4))
    }

    async fn replace_methods<C>(
        &self,
        db: &C,
        software_id: Uuid,
        methods: &[SoftwareInstallMethodDto],
    ) -> SoftwareCatalogResult<()>
    where
        C: ConnectionTrait,
    {
        software_install_method::Entity::delete_many()
            .filter(software_install_method::Column::SoftwareId.eq(software_id))
            .exec(db)
            .await
            .map_err(SoftwareCatalogError::query)?;

        let total = methods.len();
        for (index, method) in methods.iter().enumerate() {
            let active = software_install_method::ActiveModel {
                id: Set(parse_uuid(&method.id)?),
                software_id: Set(software_id),
                platform: Set(method.platform.code().to_string()),
                installer_kind: Set(method.kind.code().to_string()),
                label: Set(method.label.clone()),
                package_id: Set(method.package_id.clone()),
                asset_item_id: Set(method.asset_item_id.clone()),
                command_text: Set(method.command.clone()),
                note: Set(method.note.clone()),
                priority: Set(i32::try_from(total.saturating_sub(index)).unwrap_or_default()),
            };

            software_install_method::Entity::insert(active)
                .exec(db)
                .await
                .map_err(SoftwareCatalogError::query)?;
        }

        Ok(())
    }

    async fn list_methods_for<C>(
        &self,
        db: &C,
        software_id: Uuid,
    ) -> SoftwareCatalogResult<Vec<software_install_method::Model>>
    where
        C: ConnectionTrait,
    {
        software_install_method::Entity::find()
            .filter(software_install_method::Column::SoftwareId.eq(software_id))
            .order_by_desc(software_install_method::Column::Priority)
            .order_by_asc(software_install_method::Column::Label)
            .order_by_asc(software_install_method::Column::InstallerKind)
            .all(db)
            .await
            .map_err(SoftwareCatalogError::query)
    }
}

fn entry_from_models(
    entry: software_entry::Model,
    methods: Vec<software_install_method::Model>,
) -> SoftwareEntryDto {
    let tags = serde_json::from_value::<Vec<String>>(entry.tags).unwrap_or_default();
    let trial_codes =
        serde_json::from_value::<Vec<String>>(entry.trial_platforms).unwrap_or_default();

    SoftwareEntryDto {
        id: entry.id.to_string(),
        slug: entry.slug,
        title: entry.title,
        vendor: entry.vendor,
        summary: entry.summary,
        homepage_url: entry.homepage_url,
        icon_url: entry.icon_url,
        trial_platforms: trial_codes
            .iter()
            .filter_map(|code| SoftwarePlatform::from_code(code))
            .collect(),
        tags,
        methods: methods.into_iter().map(method_from_model).collect(),
    }
}

fn method_from_model(model: software_install_method::Model) -> SoftwareInstallMethodDto {
    SoftwareInstallMethodDto {
        id: model.id.to_string(),
        platform: SoftwarePlatform::from_code(&model.platform).unwrap_or(SoftwarePlatform::Macos),
        kind: InstallerKind::from_code(&model.installer_kind).unwrap_or(InstallerKind::Custom),
        label: model.label,
        package_id: model.package_id,
        asset_item_id: model.asset_item_id,
        command: model.command_text,
        note: model.note,
    }
}
