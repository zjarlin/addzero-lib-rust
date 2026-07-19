//! `az-algorithm` 的 Rudi provider。

use std::sync::Arc;

use rudi::{Context, DynProvider, Module, Singleton, modules, providers, singleton};

use crate::{
    service::{
        DefaultAlgorithmCatalogService, DefaultImagePipelineService, DefaultVideoPipelineService,
    },
    spi::{AlgorithmCatalogServiceRef, ImagePipelineServiceRef, VideoPipelineServiceRef},
};

/// 注册算法目录 singleton。
#[Singleton]
pub fn algorithm_catalog_service() -> AlgorithmCatalogServiceRef {
    Arc::new(DefaultAlgorithmCatalogService)
}

/// 注册图片流水线 singleton。
#[Singleton]
pub fn image_pipeline_service() -> ImagePipelineServiceRef {
    Arc::new(DefaultImagePipelineService)
}

/// 注册视频流水线 singleton。
#[Singleton]
pub fn video_pipeline_service() -> VideoPipelineServiceRef {
    Arc::new(DefaultVideoPipelineService)
}

/// `az-algorithm` 的独立 Rudi 模块。
pub struct AlgorithmModule;

impl Module for AlgorithmModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| {
                Arc::new(DefaultAlgorithmCatalogService) as AlgorithmCatalogServiceRef
            }),
            singleton(|_| Arc::new(DefaultImagePipelineService) as ImagePipelineServiceRef),
            singleton(|_| Arc::new(DefaultVideoPipelineService) as VideoPipelineServiceRef),
        ]
    }
}

/// 创建只包含算法服务 provider 的独立上下文。
///
/// 应用级项目仍可把 [`crate::enable`] 放进自身的 `rudi::enable!` 聚合入口，
/// 再通过应用的全局上下文解析相同服务类型。
#[must_use]
pub fn create_algorithm_context() -> Context {
    Context::create(modules![AlgorithmModule])
}

/// 从上下文解析算法目录服务。
///
/// # Errors
/// 未启用 `az-algorithm` provider 时返回错误。
pub fn resolve_algorithm_catalog(
    context: &mut Context,
) -> anyhow::Result<AlgorithmCatalogServiceRef> {
    context
        .resolve_option::<AlgorithmCatalogServiceRef>()
        .ok_or_else(|| missing_provider("AlgorithmCatalogServiceRef"))
}

/// 从上下文解析图片流水线服务。
///
/// # Errors
/// 未启用 `az-algorithm` provider 时返回错误。
pub fn resolve_image_pipeline(context: &mut Context) -> anyhow::Result<ImagePipelineServiceRef> {
    context
        .resolve_option::<ImagePipelineServiceRef>()
        .ok_or_else(|| missing_provider("ImagePipelineServiceRef"))
}

/// 从上下文解析视频流水线服务。
///
/// # Errors
/// 未启用 `az-algorithm` provider 时返回错误。
pub fn resolve_video_pipeline(context: &mut Context) -> anyhow::Result<VideoPipelineServiceRef> {
    context
        .resolve_option::<VideoPipelineServiceRef>()
        .ok_or_else(|| missing_provider("VideoPipelineServiceRef"))
}

fn missing_provider(provider: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "missing Rudi provider for `{provider}`; call `az_algorithm::enable()` before creating the context"
    )
}

#[cfg(test)]
mod tests {
    use crate::catalog::model::{AlgorithmTargetKind, AlgorithmTaskKind};

    use super::*;

    #[test]
    fn rudi_context_resolves_algorithm_services() -> anyhow::Result<()> {
        let mut context = create_algorithm_context();

        let catalog = resolve_algorithm_catalog(&mut context)?;
        let image_pipeline = resolve_image_pipeline(&mut context)?;
        let video_pipeline = resolve_video_pipeline(&mut context)?;
        let image_pipeline_again = resolve_image_pipeline(&mut context)?;
        let video_pipeline_again = resolve_video_pipeline(&mut context)?;

        // 三个服务必须由同一个 Rudi 上下文完整提供，调用方不再自行拼装实现。
        assert_eq!(catalog.components().len(), 9);
        assert!(Arc::ptr_eq(&image_pipeline, &image_pipeline_again));
        assert!(Arc::ptr_eq(&video_pipeline, &video_pipeline_again));
        Ok(())
    }

    #[test]
    fn injected_catalog_filters_component_contracts() -> anyhow::Result<()> {
        let mut context = create_algorithm_context();
        let catalog = resolve_algorithm_catalog(&mut context)?;

        let recognition = catalog.components_by_task(AlgorithmTaskKind::Recognition);
        let person = catalog.components_by_target(AlgorithmTargetKind::Person);
        let qr_code = catalog.component_by_code("qr_code_recognition");

        // 注入后的目录服务必须保持原有静态目录的查询语义。
        assert_eq!(recognition.len(), 3);
        assert_eq!(person.len(), 1);
        assert_eq!(
            qr_code.map(|item| item.label),
            Some("二维码识别".to_string())
        );
        Ok(())
    }

    #[test]
    fn missing_provider_returns_boundary_error() {
        let mut context = Context::default();

        let error = match resolve_algorithm_catalog(&mut context) {
            Ok(_) => panic!("expected missing Rudi provider error"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("AlgorithmCatalogServiceRef"));
    }
}
