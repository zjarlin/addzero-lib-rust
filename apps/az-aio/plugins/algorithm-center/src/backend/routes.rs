use axum::{
    Json, Router,
    extract::Multipart,
    http::StatusCode,
    routing::{get, post},
};
use az_str::sanitize::sanitize_path_segment;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub ok: bool,
    pub error: String,
}

pub fn algorithm_center_router() -> Router {
    Router::new()
        .route("/api/algorithm-center/status", get(status_handler))
        .route("/api/algorithm-center/components", get(components_handler))
        .route("/api/algorithm-center/process", post(process_handler))
        .route("/api/algorithm-center/upload", post(upload_handler))
}

async fn status_handler() -> Json<StatusResponse> {
    Json(StatusResponse {
        ok: true,
        component_count: az_algorithm::catalog::algorithm_component_descriptors().len(),
        process_endpoint: "/api/algorithm-center/process".to_string(),
        upload_endpoint: "/api/algorithm-center/upload".to_string(),
        mode: "contract_preview".to_string(),
    })
}

async fn components_handler(
) -> Json<Vec<az_algorithm::catalog::AlgorithmComponentDescriptor>> {
    Json(az_algorithm::catalog::algorithm_component_descriptors())
}

async fn process_handler(
    Json(request): Json<ProcessVideoRequest>,
) -> Result<Json<ProcessVideoResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    process_video(request).map(Json)
}

async fn upload_handler(
    mut multipart: Multipart,
) -> Result<Json<UploadVideoResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    let mut file_name = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        api_error(
            StatusCode::BAD_REQUEST,
            format!("读取 multipart 表单失败: {err}"),
        )
    })? {
        if field.name() == Some("video") {
            file_name = field.file_name().map(ToOwned::to_owned);
            let _ = field.bytes().await.map_err(|err| {
                api_error(StatusCode::BAD_REQUEST, format!("读取视频字段失败: {err}"))
            })?;
            break;
        }
    }

    let uploaded_video_url = file_name
        .as_deref()
        .map(|name| format!("/api/algorithm-center/uploads/{}", sanitize_path_segment(name)))
        .unwrap_or_else(|| "/api/algorithm-center/uploads/sample-video.mp4".to_string());

    Ok(Json(UploadVideoResponse {
        ok: true,
        mode: "contract_preview".to_string(),
        file_name,
        uploaded_video_url,
        process_endpoint: "/api/algorithm-center/process".to_string(),
        message: "上传接口已固定契约；当前版本只返回可传给 process 的视频 URL 占位。".to_string(),
    }))
}

fn process_video(
    request: ProcessVideoRequest,
) -> Result<ProcessVideoResponse, (StatusCode, Json<ApiErrorResponse>)> {
    let video_url = request.video_url.trim();
    if video_url.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "video_url 不能为空"));
    }

    let algorithms = selected_algorithms(&request.algorithms)?;
    let algorithm_codes = algorithms
        .iter()
        .map(|algorithm| algorithm.code.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let job_id = format!("job-{:x}", md5::compute(format!("{video_url}|{algorithm_codes}")));

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

fn selected_algorithms(
    requested: &[String],
) -> Result<Vec<AlgorithmSelection>, (StatusCode, Json<ApiErrorResponse>)> {
    let descriptors = az_algorithm::catalog::algorithm_component_descriptors();
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
            let Some(descriptor) = descriptors.iter().find(|descriptor| descriptor.code == code)
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

fn api_error(
    status: StatusCode,
    error: impl Into<String>,
) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        status,
        Json(ApiErrorResponse {
            ok: false,
            error: error.into(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn status_reports_ok_with_nine_components() {
        let app = algorithm_center_router();
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
        let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
        let status: StatusResponse = serde_json::from_slice(&body).unwrap();
        assert!(status.ok);
        assert_eq!(status.component_count, 9);
        assert_eq!(status.mode, "contract_preview");
    }

    #[tokio::test]
    async fn components_contains_known_algorithms() {
        let app = algorithm_center_router();
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
        let body = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
        let descriptors: Vec<az_algorithm::catalog::AlgorithmComponentDescriptor> =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(descriptors.len(), 9);
        let codes: Vec<&str> = descriptors.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"face_detection"));
        assert!(codes.contains(&"person_detection"));
        assert!(codes.contains(&"ocr_text_recognition"));
    }

    #[tokio::test]
    async fn process_accepts_multiple_algorithms_and_returns_processed_url() {
        let app = algorithm_center_router();
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
        let body = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
        let process: ProcessVideoResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(process.mode, "contract_preview");
        assert_eq!(process.algorithms.len(), 2);
        assert!(process.processed_video_url.ends_with("/processed.mp4"));
    }

    #[tokio::test]
    async fn process_rejects_unknown_algorithm_code() {
        let app = algorithm_center_router();
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
}
