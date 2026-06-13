#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
model_dir="$(cd -- "${script_dir}/.." && pwd)/resources/models"

mkdir -p "${model_dir}"

download() {
  local url="$1"
  local file_name="$2"
  local target_file="${model_dir}/${file_name}"

  if [[ -s "${target_file}" ]]; then
    printf 'exists %s\n' "${target_file}"
    return
  fi

  printf 'download %s\n' "${target_file}"
  curl --fail --location --retry 3 --output "${target_file}.tmp" "${url}"
  mv "${target_file}.tmp" "${target_file}"
}

download 'https://huggingface.co/RuteNL/SCRFD-face-detection-ONNX/resolve/3d9a1b3bc9f8a50635817929118fb9184f5bc30b/500m.onnx' 'face_detection_scrfd_500m.onnx'
download 'https://huggingface.co/onnxmodelzoo/arcfaceresnet100-11-int8/resolve/c0ec783c5907f34e089495d6d0428e847fcededa/arcfaceresnet100-11-int8.onnx' 'face_recognition_arcface_resnet100_int8.onnx'
download 'https://huggingface.co/onnxmodelzoo/ssd_mobilenet_v1_10/resolve/338a91b8e06061536f22129b4bf5227a3d496e8c/ssd_mobilenet_v1_10.onnx' 'coco_ssd_mobilenet_v1_10.onnx'
download 'https://huggingface.co/prithivMLmods/Fire-Detection-Engine-ONNX/resolve/02bd7f981aac3e27a75f83e0a3b97dfadaffc228/onnx/model_int8.onnx' 'fire_detection_vit_int8.onnx'
download 'https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/detection/v3/det.onnx' 'ocr_paddle_v3_det.onnx'
download 'https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/chinese/rec.onnx' 'ocr_paddle_chinese_rec.onnx'
download 'https://huggingface.co/monkt/paddleocr-onnx/resolve/7b02d0a30a07ba2b92ad1ff5a8941ae2c633de65/languages/chinese/dict.txt' 'ocr_paddle_chinese_dict.txt'
download 'https://huggingface.co/nduka1999/nd_ppe_yolo11s/resolve/90f3e8915ef403dbbc77bb6ba713916321e2970f/best.onnx' 'safety_helmet_detection_ppe_yolo11s.onnx'

printf 'model resources ready: %s\n' "${model_dir}"
