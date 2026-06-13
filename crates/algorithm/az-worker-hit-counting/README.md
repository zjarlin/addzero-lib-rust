# 工人敲击计数算法

独立工人有效敲击计数算法 crate。

该 crate 只处理纯视觉结构化观测，不读取音频。调用方需要先完成：

- 人员检测与稳定跟踪，提供 `person_id`
- 工具/手部末端接触点定位
- 悬挂金属板、流水线台体边缘、支架等目标识别
- 悬挂金属板被敲后的视觉响应评分

只有“接触点命中悬挂金属板”且“目标出现足够响应”的动作才计入有效敲击。
乱敲流水线台体边缘会被记录为无效候选，不增加有效敲击次数。

## 输入

- 已结构化视觉观测：`record_worker_hit_timeline_from_visual_observations(...)`
- 真实视频 + YOLO pose + ROI 规则：`analyze_worker_hits_in_video_from_path(...)`

视频入口需要调用方配置：

- `pose_model_path`：`resources/models/yolov8n_pose.onnx`
- `ffmpeg_path`：例如 `/opt/homebrew/bin/ffmpeg`
- `target_roi`：现场中需要计为有效敲击的目标区域
- `pose_score_threshold` / `keypoint_score_threshold` / `WorkerHitCountConfig`：按现场视频调参

## 输出

视频分析会输出到调用方配置的目录：

- `source_input.mp4`：原始输入视频副本
- `extracted_frames/`：ffmpeg 抽帧
- `annotated_frames/`：pose、ROI 标注帧
- `pose_frames.json`：逐帧 pose 候选和关键点
- `action_observations.json`：由 pose + ROI 规则生成的动作观测
- `worker_hit_timeline.json`：每帧动作状态、有效敲击事件、最终计数
- `annotated_worker_hits.mp4`：标注后视频

当前真实视频测试使用 `/Users/zjarlin/Desktop/246f5787eca62dc0b462dbc041da756f.mp4` 的前 6 个 1fps 抽帧。它验证的是“真实 pose 推理 + ROI 规则 + 时间线落盘”链路，不代表已经完成现场最终精度。低阈值会引入误检，正式使用前需要按工位 ROI 和动作样本标定。

## 模型

- `resources/models/yolov8n_pose.onnx`：来自 `Xenova/yolov8-pose-onnx`，Hugging Face 元数据标注为 `agpl-3.0`。

## 测试

```shell
cargo test -p az-worker-hit-counting --test worker_hit_counting -- --nocapture
```
