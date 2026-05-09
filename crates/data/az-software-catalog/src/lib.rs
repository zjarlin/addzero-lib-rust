pub mod model;

#[cfg(not(target_arch = "wasm32"))]
mod entity;
#[cfg(not(target_arch = "wasm32"))]
mod import;
#[cfg(not(target_arch = "wasm32"))]
mod repository;
#[cfg(not(target_arch = "wasm32"))]
pub mod service;

pub use model::{
    InstallerKind, SoftwareCatalogDto, SoftwareCatalogError, SoftwareCatalogResult,
    SoftwareDraftInput, SoftwareEntryDto, SoftwareEntryInput, SoftwareInstallMethodDto,
    SoftwareMetadataDto, SoftwareMetadataFetchInput, SoftwarePlatform, current_platform,
};
#[cfg(not(target_arch = "wasm32"))]
pub use service::SoftwareCatalogService;
