# 视频实时算法叠加编排

本 crate 负责把同一条视频帧流分发给多个常驻算法实例，解决“人脸检测、抽烟检测、安全帽检测、人员动作状态”等算法如何在视频实时计算中叠加运行的问题。

边界很明确：

- 单个识别算法仍然保持一个算法一个 crate。
- 模型加载、真实推理、后处理仍然放在对应算法 crate。
- 本 crate 只负责帧模型、按帧率调度、多算法叠加、结构化结果落盘。
- 算法实例由调用方创建并常驻传入，避免每帧重复加载模型。
- 输入可以来自 ffmpeg 解码、摄像头、RTSP、WebRTC 或测试中的内存帧。

基本用法：

```rust,no_run
use az_algorithm_video_pipeline::logic_algorithm_video_pipeline::assist::run_video_frame_pipeline;
use az_algorithm_video_pipeline::logic_algorithm_video_pipeline::model::{
    VideoAlgorithmBinding, VideoAlgorithmFrameResult, VideoAlgorithmSchedule, VideoFrame,
    VideoFrameAlgorithm, VideoPipelineOptions,
};

struct MyDetector;

impl VideoFrameAlgorithm for MyDetector {
    fn code(&self) -> &'static str {
        "my_detector"
    }

    fn process_frame(
        &mut self,
        frame: &VideoFrame,
    ) -> anyhow::Result<VideoAlgorithmFrameResult> {
        Ok(VideoAlgorithmFrameResult::empty(self.code(), frame))
    }
}

# fn main() -> anyhow::Result<()> {
let frames = Vec::<VideoFrame>::new();
let mut detector = MyDetector;
let mut algorithms = [VideoAlgorithmBinding {
    algorithm: &mut detector,
    schedule: VideoAlgorithmSchedule::EveryFrame,
}];

let run = run_video_frame_pipeline(
    frames,
    &mut algorithms,
    &VideoPipelineOptions {
        output_dir: "/absolute/path/to/output".into(),
        source_fps: 30.0,
    },
)?;
println!("{}", run.files.summary_json.display());
# Ok(())
# }
```

实际视频工程里通常把视频处理拆成三层：

1. 解码层：把视频、摄像头或 RTSP 流转成 `VideoFrame`。
2. 算法层：每个算法实现 `VideoFrameAlgorithm`，模型常驻内存。
3. 输出层：消费 `pipeline_frame_results.jsonl`，叠加画框、事件标签或告警。

这样做比“每个算法单独抽帧、单独读视频、单独写视频”更适合实时场景，因为同一帧只解码一次，多个算法共享帧数据，并且每个算法可以按自己的频率运行。

## 内置适配器

本 crate 已提供两个通用实时适配器：

- `logic_algorithm_video_pipeline::assist::onnx_raw_image_video_algorithm::OnnxRawImageVideoAlgorithm`
- `logic_algorithm_video_pipeline::assist::qr_code_video_algorithm::QrCodeVideoAlgorithm`

`OnnxRawImageVideoAlgorithm` 适合先把已有图片 ONNX 模型挂到视频流中，例如安全帽、车辆、火焰、人脸识别、OCR 文字检测。它只输出真实 ONNX raw 摘要，不会把张量伪造成检测框。后续要显示安全帽框、车辆框、火焰框或 OCR 文本，需要在对应算法 crate 内实现模型后处理，再替换成专用 `VideoFrameAlgorithm`。

`QrCodeVideoAlgorithm` 已经能直接输出二维码 payload、角点和视频目标框。

当前真实测试覆盖的输出位置：

- `target/az-algorithm-results/video_pipeline_all_frame_image_algorithms/pipeline_frame_results.jsonl`
- `target/az-algorithm-results/video_pipeline_all_frame_image_algorithms/pipeline_summary.json`
- `target/az-algorithm-results/video_pipeline_real_person_face_stack/pipeline_frame_results.jsonl`
- `target/az-algorithm-results/video_pipeline_real_safety_helmet_raw/pipeline_summary.json`

## 真实测试

```shell
cargo test -p az-algorithm-video-pipeline --test video_pipeline -- --nocapture
```

其中 `video_pipeline_should_stack_all_frame_image_algorithms_on_one_frame_stream` 会把人员、人脸、二维码、安全帽、车辆、火焰、人脸识别、OCR 文字检测同时挂到同一帧流上运行。人员和人脸会返回真实检测框，二维码会返回真实 payload；安全帽、车辆、火焰、人脸识别、OCR 文字检测当前只验证真实 ONNX raw 输出。
