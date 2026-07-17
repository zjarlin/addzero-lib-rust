use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use az_aio_platform::core::{
    api_error::{ApiError, ApiForm, ApiJson, ApiMultipart},
    upload::{
        DEFAULT_UPLOAD_LIMIT_BYTES, MultipartUploadOptions, save_single_multipart_upload,
        upload_file_service,
    },
};
use az_algorithm::spi::AlgorithmCatalogServiceRef;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const UPLOAD_URL_PREFIX: &str = "/api/algorithm-center/uploads";
const UPLOAD_FIELD_NAME: &str = "video";
const FALLBACK_UPLOAD_FILE_NAME: &str = "input-video.mp4";

#[derive(Clone)]
pub struct AlgorithmCenterApiState {
    pub catalog: AlgorithmCatalogServiceRef,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub ok: bool,
    pub component_count: usize,
    pub process_endpoint: String,
    pub upload_endpoint: String,
    pub mode: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessVideoRequest {
    pub video_url: String,
    #[serde(default)]
    pub algorithms: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessVideoResponse {
    pub ok: bool,
    pub mode: String,
    pub job_id: String,
    pub input_video_url: String,
    pub processed_video_url: String,
    pub algorithms: Vec<AlgorithmSelection>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlgorithmSelection {
    pub code: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadVideoResponse {
    pub ok: bool,
    pub mode: String,
    pub file_name: Option<String>,
    pub uploaded_video_url: String,
    pub process_endpoint: String,
    pub message: String,
}

pub fn algorithm_center_router(state: AlgorithmCenterApiState) -> Router {
    let upload_dir = upload_storage_dir();
    Router::new()
        .route("/api/algorithm-center/status", get(status_handler))
        .route("/api/algorithm-center/components", get(components_handler))
        .route("/api/algorithm-center/process", post(process_handler))
        .route("/api/algorithm-center/ui-action", post(ui_action_handler))
        .route(
            "/api/algorithm-center/upload",
            post(upload_handler).layer(DefaultBodyLimit::max(DEFAULT_UPLOAD_LIMIT_BYTES)),
        )
        .with_state(state)
        .merge(upload_file_service(UPLOAD_URL_PREFIX, upload_dir))
}

async fn status_handler(State(state): State<AlgorithmCenterApiState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        ok: true,
        component_count: state.catalog.components().len(),
        process_endpoint: "/api/algorithm-center/process".to_string(),
        upload_endpoint: "/api/algorithm-center/upload".to_string(),
        mode: "contract_preview".to_string(),
    })
}

async fn components_handler(
    State(state): State<AlgorithmCenterApiState>,
) -> Json<Vec<az_algorithm::catalog::model::AlgorithmComponentDescriptor>> {
    Json(state.catalog.components())
}

async fn process_handler(
    State(state): State<AlgorithmCenterApiState>,
    ApiJson(request): ApiJson<ProcessVideoRequest>,
) -> Result<Json<ProcessVideoResponse>, ApiError> {
    process_video(request, &state.catalog).map(Json)
}

async fn upload_handler(
    ApiMultipart(multipart): ApiMultipart,
) -> Result<Json<UploadVideoResponse>, ApiError> {
    let upload = save_single_multipart_upload(
        multipart,
        MultipartUploadOptions {
            field_name: UPLOAD_FIELD_NAME.to_string(),
            storage_dir: upload_storage_dir(),
            public_url_prefix: UPLOAD_URL_PREFIX.to_string(),
            fallback_file_name: FALLBACK_UPLOAD_FILE_NAME.to_string(),
        },
    )
    .await
    .map_err(|err| api_error(StatusCode::BAD_REQUEST, format!("上传视频失败: {err:#}")))?;

    Ok(Json(UploadVideoResponse {
        ok: true,
        mode: "contract_preview".to_string(),
        file_name: upload.original_file_name,
        uploaded_video_url: upload.public_url,
        process_endpoint: "/api/algorithm-center/process".to_string(),
        message: format!(
            "上传成功，已接收 {} 字节，可将 URL 传给 process。",
            upload.byte_len
        ),
    }))
}

async fn ui_action_handler(
    State(state): State<AlgorithmCenterApiState>,
    ApiForm(form): ApiForm<ProcessVideoForm>,
) -> Response {
    let redirect = match process_video(
        ProcessVideoRequest {
            video_url: form.video_url,
            algorithms: form.algorithms,
        },
        &state.catalog,
    ) {
        Ok(result) => process_redirect(result),
        Err(error) => format!(
            "/?route=/algorithms&error={}",
            urlencoding::encode(error.message())
        ),
    };
    Redirect::to(&redirect).into_response()
}

fn upload_storage_dir() -> PathBuf {
    std::env::var_os("AZ_AIO_UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir()
                .join("az-aio")
                .join("algorithm-center")
                .join("uploads")
        })
}

fn process_video(
    request: ProcessVideoRequest,
    catalog: &AlgorithmCatalogServiceRef,
) -> Result<ProcessVideoResponse, ApiError> {
    let video_url = request.video_url.trim();
    if video_url.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "video_url 不能为空"));
    }

    let algorithms = selected_algorithms(&request.algorithms, catalog)?;
    let algorithm_codes = algorithms
        .iter()
        .map(|algorithm| algorithm.code.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let job_id = format!(
        "job-{:x}",
        md5::compute(format!("{video_url}|{algorithm_codes}"))
    );

    Ok(ProcessVideoResponse {
        ok: true,
        mode: "contract_preview".to_string(),
        job_id: job_id.clone(),
        input_video_url: video_url.to_string(),
        processed_video_url: format!("/api/algorithm-center/results/{job_id}/processed.mp4"),
        algorithms,
        message: "已完成多算法叠加调用契约校验；真实视频加工执行器后续接入。".to_string(),
    })
}

#[derive(Clone, Debug, Deserialize)]
struct ProcessVideoForm {
    pub video_url: String,
    #[serde(default)]
    pub algorithms: Vec<String>,
}

fn process_redirect(result: ProcessVideoResponse) -> String {
    let mut parts = vec![
        "route=/algorithms".to_string(),
        "run=1".to_string(),
        format!("video_url={}", urlencoding::encode(&result.input_video_url)),
        format!(
            "processed_video_url={}",
            urlencoding::encode(&result.processed_video_url)
        ),
        format!("job_id={}", urlencoding::encode(&result.job_id)),
        format!("message={}", urlencoding::encode(&result.message)),
    ];
    for algorithm in result.algorithms {
        parts.push(format!(
            "algorithm={}",
            urlencoding::encode(&algorithm.code)
        ));
        parts.push(format!("active={}", urlencoding::encode(&algorithm.code)));
    }
    format!("/?{}", parts.join("&"))
}

fn selected_algorithms(
    requested: &[String],
    catalog: &AlgorithmCatalogServiceRef,
) -> Result<Vec<AlgorithmSelection>, ApiError> {
    let descriptors = catalog.components();
    let codes = requested
        .iter()
        .map(|code| code.trim())
        .filter(|code| !code.is_empty())
        .collect::<Vec<_>>();

    let selected = if codes.is_empty() {
        descriptors
            .into_iter()
            .take(1)
            .map(|descriptor| AlgorithmSelection {
                code: descriptor.code,
                label: descriptor.label,
            })
            .collect()
    } else {
        let mut selected = Vec::with_capacity(codes.len());
        for code in codes {
            let Some(descriptor) = descriptors
                .iter()
                .find(|descriptor| descriptor.code == code)
            else {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    format!("未知算法 code: {code}"),
                ));
            };
            selected.push(AlgorithmSelection {
                code: descriptor.code.clone(),
                label: descriptor.label.clone(),
            });
        }
        selected
    };

    Ok(selected)
}

fn api_error(status: StatusCode, error: impl Into<String>) -> ApiError {
    ApiError::new(status, error)
}

#[cfg(test)]
mod tests {
    use az_algorithm::di::{create_algorithm_context, resolve_algorithm_catalog};

    use super::*;
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    fn test_router() -> Router {
        let mut context = create_algorithm_context();
        let catalog =
            resolve_algorithm_catalog(&mut context).expect("测试必须能解析算法目录 Rudi provider");
        algorithm_center_router(AlgorithmCenterApiState { catalog })
    }

    #[tokio::test]
    async fn status_reports_ok_with_nine_components() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/algorithm-center/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let status: StatusResponse = serde_json::from_slice(&body).unwrap();
        assert!(status.ok);
        assert_eq!(status.component_count, 9);
        assert_eq!(status.mode, "contract_preview");
    }

    #[tokio::test]
    async fn components_contains_known_algorithms() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/algorithm-center/components")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        let descriptors: Vec<az_algorithm::catalog::model::AlgorithmComponentDescriptor> =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(descriptors.len(), 9);
        let codes: Vec<&str> = descriptors.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"face_detection"));
        assert!(codes.contains(&"person_detection"));
        assert!(codes.contains(&"ocr_text_recognition"));
    }

    #[tokio::test]
    async fn process_accepts_multiple_algorithms_and_returns_processed_url() {
        let app = test_router();
        let body = serde_json::json!({
            "video_url": "https://example.test/fire.mp4",
            "algorithms": ["flame_detection", "face_detection"]
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/algorithm-center/process")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        let process: ProcessVideoResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(process.mode, "contract_preview");
        assert_eq!(process.algorithms.len(), 2);
        assert!(process.processed_video_url.ends_with("/processed.mp4"));
    }

    #[tokio::test]
    async fn process_rejects_unknown_algorithm_code() {
        let app = test_router();
        let body = serde_json::json!({
            "video_url": "https://example.test/fire.mp4",
            "algorithms": ["not_exist"]
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/algorithm-center/process")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_accepts_multipart_video_and_returns_url() {
        let app = test_router();
        let boundary = "az-aio-test-boundary";
        let body = multipart_video_body(boundary, "demo clip.mp4", b"video-bytes");
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/algorithm-center/upload")
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap();
        let upload: UploadVideoResponse = serde_json::from_slice(&body).unwrap();
        assert!(upload.ok);
        assert_eq!(upload.file_name.as_deref(), Some("demo clip.mp4"));
        assert!(
            upload
                .uploaded_video_url
                .contains("/api/algorithm-center/uploads/")
        );
        assert!(upload.uploaded_video_url.ends_with("-demo-clip.mp4"));
    }

    #[tokio::test]
    async fn upload_accepts_video_larger_than_default_body_limit() {
        let app = test_router();
        let boundary = "az-aio-large-video-boundary";
        let video = vec![b'x'; 3 * 1024 * 1024];
        let body = multipart_video_body(boundary, "large.mp4", &video);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/algorithm-center/upload")
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
    }

    fn multipart_video_body(boundary: &str, file_name: &str, content: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"video\"; filename=\"{file_name}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: video/mp4\r\n\r\n");
        body.extend_from_slice(content);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        body
    }
}
