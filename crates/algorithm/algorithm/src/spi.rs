//! 可由 Rudi 注入的算法服务接口。

use std::{path::Path, sync::Arc};

use crate::{
    catalog::model::{AlgorithmComponentDescriptor, AlgorithmTargetKind, AlgorithmTaskKind},
    pipeline::image::model::{ImagePipelineOptions, ImagePipelineRun},
    video_pipeline::model::{
        VideoAlgorithmBinding, VideoFrame, VideoPipelineOptions, VideoPipelineRun,
    },
};

/// 算法目录查询服务。
///
/// 返回可序列化描述，适合作为 API、CLI 和管理端的统一数据来源。
pub trait AlgorithmCatalogService: Send + Sync {
    /// 返回全部算法组件描述。
    fn components(&self) -> Vec<AlgorithmComponentDescriptor>;

    /// 根据稳定 code 查询组件描述。
    fn component_by_code(&self, code: &str) -> Option<AlgorithmComponentDescriptor>;

    /// 按任务类型过滤组件描述。
    fn components_by_task(&self, task: AlgorithmTaskKind) -> Vec<AlgorithmComponentDescriptor>;

    /// 按目标类型过滤组件描述。
    fn components_by_target(
        &self,
        target: AlgorithmTargetKind,
    ) -> Vec<AlgorithmComponentDescriptor>;
}

/// 存入 Rudi 的算法目录服务引用。
pub type AlgorithmCatalogServiceRef = Arc<dyn AlgorithmCatalogService + Send + Sync>;

/// 图片算法流水线服务。
pub trait ImagePipelineService: Send + Sync {
    /// 对单张图片执行配置中的算法并写出汇总结果。
    ///
    /// # Errors
    /// 输入文件、模型加载、推理或输出写入失败时返回错误。
    fn run_from_path(
        &self,
        image_path: &Path,
        options: &ImagePipelineOptions,
    ) -> anyhow::Result<ImagePipelineRun>;
}

/// 存入 Rudi 的图片流水线服务引用。
pub type ImagePipelineServiceRef = Arc<dyn ImagePipelineService + Send + Sync>;

/// 视频帧流水线服务。
pub trait VideoPipelineService: Send + Sync {
    /// 在同一组视频帧上运行多个常驻算法实例。
    ///
    /// # Errors
    /// 调度配置、算法推理或输出写入失败时返回错误。
    fn run_frames(
        &self,
        frames: Vec<VideoFrame>,
        algorithms: &mut [VideoAlgorithmBinding<'_>],
        options: &VideoPipelineOptions,
    ) -> anyhow::Result<VideoPipelineRun>;
}

/// 存入 Rudi 的视频流水线服务引用。
pub type VideoPipelineServiceRef = Arc<dyn VideoPipelineService + Send + Sync>;
