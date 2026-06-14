use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub ok: bool,
    pub component_count: usize,
}

pub fn algorithm_center_router() -> Router {
    Router::new()
        .route("/api/algorithm-center/status", get(status_handler))
        .route("/api/algorithm-center/components", get(components_handler))
}

async fn status_handler() -> Json<StatusResponse> {
    Json(StatusResponse {
        ok: true,
        component_count: az_algorithm::catalog::algorithm_component_descriptors().len(),
    })
}

async fn components_handler(
) -> Json<Vec<az_algorithm::catalog::AlgorithmComponentDescriptor>> {
    Json(az_algorithm::catalog::algorithm_component_descriptors())
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
}
