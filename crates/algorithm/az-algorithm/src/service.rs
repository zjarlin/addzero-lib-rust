//! 默认算法服务实现。

use std::path::Path;

use crate::{
    catalog::{
        model::{AlgorithmComponentDescriptor, AlgorithmTargetKind, AlgorithmTaskKind},
        query::{
            algorithm_component_by_code, algorithm_component_descriptors,
            algorithm_components_by_target, algorithm_components_by_task,
        },
    },
    pipeline::image::{
        assist::run_image_pipeline_from_path,
        model::{ImagePipelineOptions, ImagePipelineRun},
    },
    spi::{AlgorithmCatalogService, ImagePipelineService, VideoPipelineService},
    video_pipeline::{
        model::{VideoAlgorithmBinding, VideoFrame, VideoPipelineOptions, VideoPipelineRun},
        pipeline::run_video_frame_pipeline,
    },
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DefaultAlgorithmCatalogService;

impl AlgorithmCatalogService for DefaultAlgorithmCatalogService {
    fn components(&self) -> Vec<AlgorithmComponentDescriptor> {
        algorithm_component_descriptors()
    }

    fn component_by_code(&self, code: &str) -> Option<AlgorithmComponentDescriptor> {
        algorithm_component_by_code(code).map(|component| component.to_descriptor())
    }

    fn components_by_task(&self, task: AlgorithmTaskKind) -> Vec<AlgorithmComponentDescriptor> {
        algorithm_components_by_task(task)
            .map(|component| component.to_descriptor())
            .collect()
    }

    fn components_by_target(
        &self,
        target: AlgorithmTargetKind,
    ) -> Vec<AlgorithmComponentDescriptor> {
        algorithm_components_by_target(target)
            .map(|component| component.to_descriptor())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DefaultImagePipelineService;

impl ImagePipelineService for DefaultImagePipelineService {
    fn run_from_path(
        &self,
        image_path: &Path,
        options: &ImagePipelineOptions,
    ) -> anyhow::Result<ImagePipelineRun> {
        run_image_pipeline_from_path(image_path, options)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DefaultVideoPipelineService;

impl VideoPipelineService for DefaultVideoPipelineService {
    fn run_frames(
        &self,
        frames: Vec<VideoFrame>,
        algorithms: &mut [VideoAlgorithmBinding<'_>],
        options: &VideoPipelineOptions,
    ) -> anyhow::Result<VideoPipelineRun> {
        run_video_frame_pipeline(frames, algorithms, options)
    }
}
