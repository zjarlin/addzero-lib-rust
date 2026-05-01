use std::{future::Future, pin::Pin, rc::Rc};

#[cfg(target_arch = "wasm32")]
use addzero_software_catalog::SoftwareCatalogError;
#[cfg(not(target_arch = "wasm32"))]
use addzero_software_catalog::SoftwareCatalogService;
pub use addzero_software_catalog::{
    InstallerKind, SoftwareCatalogDto, SoftwareCatalogResult, SoftwareDraftInput, SoftwareEntryDto,
    SoftwareEntryInput, SoftwareInstallMethodDto, SoftwareMetadataDto, SoftwareMetadataFetchInput,
    SoftwarePlatform, current_platform,
};

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait SoftwareCatalogApi: 'static {
    fn catalog(&self) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareCatalogDto>>;
    fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareEntryDto>>;
    fn delete_entry(&self, id: String) -> LocalBoxFuture<'_, SoftwareCatalogResult<()>>;
    fn fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareMetadataDto>>;
    fn build_draft(
        &self,
        input: SoftwareDraftInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareEntryInput>>;
}

pub type SharedSoftwareCatalogApi = Rc<dyn SoftwareCatalogApi>;

pub fn default_software_catalog_api() -> SharedSoftwareCatalogApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserSoftwareCatalogApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedSoftwareCatalogApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserSoftwareCatalogApi;

#[cfg(target_arch = "wasm32")]
impl SoftwareCatalogApi for BrowserSoftwareCatalogApi {
    fn catalog(&self) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareCatalogDto>> {
        Box::pin(async move {
            let mut catalog: SoftwareCatalogDto =
                super::browser_http::get_json("/api/software-catalog")
                    .await
                    .map_err(SoftwareCatalogError::Message)?;
            catalog.host_platform = runtime_host_platform();
            Ok(catalog)
        })
    }

    fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareEntryDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/software-catalog/upsert", &input)
                .await
                .map_err(SoftwareCatalogError::Message)
        })
    }

    fn delete_entry(&self, id: String) -> LocalBoxFuture<'_, SoftwareCatalogResult<()>> {
        Box::pin(async move {
            super::browser_http::delete_empty(&format!("/api/software-catalog/{id}"))
                .await
                .map_err(SoftwareCatalogError::Message)
        })
    }

    fn fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareMetadataDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/software-catalog/fetch", &input)
                .await
                .map_err(SoftwareCatalogError::Message)
        })
    }

    fn build_draft(
        &self,
        input: SoftwareDraftInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareEntryInput>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/software-catalog/draft", &input)
                .await
                .map_err(SoftwareCatalogError::Message)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
struct EmbeddedSoftwareCatalogApi;

#[cfg(not(target_arch = "wasm32"))]
impl SoftwareCatalogApi for EmbeddedSoftwareCatalogApi {
    fn catalog(&self) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareCatalogDto>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let service = backend.software_catalog.as_ref().ok_or_else(|| {
                addzero_software_catalog::SoftwareCatalogError::Message(
                    "software catalog backend unavailable".to_string(),
                )
            })?;
            service.catalog().await
        })
    }

    fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareEntryDto>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let service = backend.software_catalog.as_ref().ok_or_else(|| {
                addzero_software_catalog::SoftwareCatalogError::Message(
                    "software catalog backend unavailable".to_string(),
                )
            })?;
            service.save_entry(input).await
        })
    }

    fn delete_entry(&self, id: String) -> LocalBoxFuture<'_, SoftwareCatalogResult<()>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let service = backend.software_catalog.as_ref().ok_or_else(|| {
                addzero_software_catalog::SoftwareCatalogError::Message(
                    "software catalog backend unavailable".to_string(),
                )
            })?;
            service.delete_entry(&id).await
        })
    }

    fn fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareMetadataDto>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let service = backend.software_catalog.as_ref().ok_or_else(|| {
                addzero_software_catalog::SoftwareCatalogError::Message(
                    "software catalog backend unavailable".to_string(),
                )
            })?;
            service.fetch_metadata(input).await
        })
    }

    fn build_draft(
        &self,
        input: SoftwareDraftInput,
    ) -> LocalBoxFuture<'_, SoftwareCatalogResult<SoftwareEntryInput>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let service = backend.software_catalog.as_ref().ok_or_else(|| {
                addzero_software_catalog::SoftwareCatalogError::Message(
                    "software catalog backend unavailable".to_string(),
                )
            })?;
            service.build_draft(input).await
        })
    }
}

#[cfg(target_arch = "wasm32")]
fn runtime_host_platform() -> SoftwarePlatform {
    let user_agent = web_sys::window()
        .and_then(|window| window.navigator().user_agent().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let platform = web_sys::window()
        .and_then(|window| window.navigator().platform().ok())
        .unwrap_or_default();
    let platform = platform.to_ascii_lowercase();
    let fingerprint = format!("{user_agent} {platform}");
    if fingerprint.contains("win") {
        SoftwarePlatform::Windows
    } else if fingerprint.contains("linux")
        || fingerprint.contains("x11")
        || fingerprint.contains("ubuntu")
    {
        SoftwarePlatform::Linux
    } else {
        SoftwarePlatform::Macos
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type EmbeddedSoftwareCatalogService = SoftwareCatalogService;
