//! Axum 统一错误边界。#{7}
//!
//! Rust 业务层继续使用 `anyhow::Result` 透明传播错误；本模块只在 HTTP
//! 边界把业务错误、提取器拒绝和 Tower 中间件错误统一转换成 `{ code, msg, data }`
//! JSON 响应，承担类似 Spring `@RestControllerAdvice` 的出口职责。

use std::{fmt::Display, time::Duration};

use axum::{
    BoxError, Form, Json, Router,
    error_handling::HandleErrorLayer,
    extract::{
        FromRequest, FromRequestParts, Multipart, Path, Query, Request,
        multipart::MultipartRejection,
        rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tower::{ServiceBuilder, timeout::TimeoutLayer};

/// 默认 API 超时时间，避免接口静默挂死。
pub const DEFAULT_API_TIMEOUT_SECS: u64 = 10;

/// 统一 API 响应体，错误时 `data` 固定为空。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// 构造成功响应。
    pub fn ok(data: T) -> Self {
        Self {
            code: StatusCode::OK.as_u16(),
            msg: "ok".to_string(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    /// 构造错误响应。
    pub fn error(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: status.as_u16(),
            msg: message.into(),
            data: None,
        }
    }
}

/// HTTP 边界错误，不作为业务层运行时错误模型使用。
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    /// 使用明确 HTTP 状态构造边界错误。
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    /// 构造 400 参数或请求体错误。
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// 构造 404 资源不存在错误。
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// 构造 503 依赖未就绪错误。
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }

    /// 构造 500 未知服务错误。
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    /// 保留已有 `anyhow` 错误链的文本，并只在 HTTP 出口做状态推断。
    pub fn from_anyhow(error: anyhow::Error) -> Self {
        let message = error.to_string();
        Self::new(api_status_for_message(&message), message)
    }

    /// 返回 HTTP 状态码。
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// 返回客户端可读错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::from_anyhow(value)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!(
                status = self.status.as_u16(),
                message = %self.message,
                "API 全局错误边界捕获服务端错误",
            );
        }

        let body = ApiResponse::error(self.status, self.message);
        (self.status, Json(body)).into_response()
    }
}

/// 构造统一成功 JSON。
pub fn ok_json<T>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse::ok(data))
}

/// 把 `anyhow::Result` 转换为统一 HTTP 响应。
pub fn into_api_response<T: Serialize>(result: anyhow::Result<T>) -> Response {
    match result {
        Ok(data) => ok_json(data).into_response(),
        Err(error) => ApiError::from_anyhow(error).into_response(),
    }
}

/// 给 Axum Router 挂载全局兜底层。
///
/// 该层捕获 Tower service error，例如超时；业务错误和提取器错误需要通过
/// [`ApiError`]、[`ApiJson`]、[`ApiQuery`] 等边界类型进入统一响应。
pub fn with_global_api_error_layer(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_box_error))
            .layer(TimeoutLayer::new(Duration::from_secs(
                DEFAULT_API_TIMEOUT_SECS,
            ))),
    )
}

/// Tower 层错误兜底处理。
pub async fn handle_box_error(error: BoxError) -> Response {
    if error.is::<tower::timeout::error::Elapsed>() {
        return ApiError::new(StatusCode::GATEWAY_TIMEOUT, "接口处理超时").into_response();
    }

    tracing::error!(error = ?error, "API 全局错误边界捕获 Tower 错误");
    ApiError::internal("服务未知错误").into_response()
}

/// 使用统一错误响应的 Query 提取器。
pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(query_rejection)
    }
}

/// 使用统一错误响应的 Path 提取器。
pub struct ApiPath<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Path::<T>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(path_rejection)
    }
}

/// 使用统一错误响应的 JSON 提取器。
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(req, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(json_rejection)
    }
}

/// 使用统一错误响应的表单提取器。
pub struct ApiForm<T>(pub T);

impl<T, S> FromRequest<S> for ApiForm<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Form::<T>::from_request(req, state)
            .await
            .map(|Form(value)| Self(value))
            .map_err(form_rejection)
    }
}

/// 使用统一错误响应的 multipart 提取器。
pub struct ApiMultipart(pub Multipart);

impl<S> FromRequest<S> for ApiMultipart
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Multipart::from_request(req, state)
            .await
            .map(Self)
            .map_err(multipart_rejection)
    }
}

/// 解析数字参数并生成接近 Java `NumberFormatException` 的友好提示。
pub fn parse_i64_param(name: &str, value: &str) -> Result<i64, ApiError> {
    parse_number_param(name, value)
}

/// 解析 32 位数字参数并生成统一参数错误。
pub fn parse_i32_param(name: &str, value: &str) -> Result<i32, ApiError> {
    parse_number_param(name, value)
}

/// 解析无符号分页数字参数并生成统一参数错误。
pub fn parse_usize_param(name: &str, value: &str) -> Result<usize, ApiError> {
    parse_number_param(name, value)
}

fn parse_number_param<T>(name: &str, value: &str) -> Result<T, ApiError>
where
    T: std::str::FromStr,
    T::Err: Display,
{
    value
        .trim()
        .parse::<T>()
        .map_err(|_| ApiError::bad_request(format!("参数[{name}]必须为数字，非法输入：{value}")))
}

fn query_rejection(error: QueryRejection) -> ApiError {
    ApiError::bad_request(format!("查询参数解析失败: {}", error.body_text()))
}

fn path_rejection(error: PathRejection) -> ApiError {
    ApiError::bad_request(format!("路径参数解析失败: {}", error.body_text()))
}

fn json_rejection(error: JsonRejection) -> ApiError {
    ApiError::bad_request(format!("JSON 请求体解析失败: {}", error.body_text()))
}

fn form_rejection(error: FormRejection) -> ApiError {
    ApiError::bad_request(format!("表单请求体解析失败: {}", error.body_text()))
}

fn multipart_rejection(error: MultipartRejection) -> ApiError {
    ApiError::bad_request(format!("multipart 请求体解析失败: {}", error.body_text()))
}

fn api_status_for_message(message: &str) -> StatusCode {
    let normalized = message.trim().to_ascii_lowercase();
    if normalized.contains("missing ") && normalized.contains(" database url") {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    if normalized.starts_with("not found")
        || normalized.contains(" was not found")
        || message.contains("不存在")
    {
        return StatusCode::NOT_FOUND;
    }
    if normalized.starts_with("unauthorized")
        || message.contains("未登录")
        || message.contains("登录已失效")
    {
        return StatusCode::UNAUTHORIZED;
    }
    if normalized.starts_with("forbidden") {
        return StatusCode::FORBIDDEN;
    }
    if normalized.starts_with("conflict") || normalized.contains("duplicate") {
        return StatusCode::CONFLICT;
    }
    if normalized.starts_with("bad request")
        || normalized.starts_with("invalid ")
        || normalized.contains("must not be blank")
        || normalized.contains("is required")
        || normalized.contains("mismatch")
        || message.contains("不能为空")
        || message.contains("必须")
        || message.contains("缺少")
        || message.contains("不支持")
        || message.contains("类型不匹配")
    {
        return StatusCode::BAD_REQUEST;
    }
    if normalized.starts_with("timeout") || normalized.contains("timed out") {
        return StatusCode::GATEWAY_TIMEOUT;
    }
    StatusCode::INTERNAL_SERVER_ERROR
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, routing::get};
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    #[derive(Deserialize)]
    struct DetailQuery {
        id: i64,
    }

    async fn detail_handler(ApiQuery(query): ApiQuery<DetailQuery>) -> Json<ApiResponse<i64>> {
        ok_json(query.id)
    }

    #[tokio::test]
    async fn query_rejection_returns_unified_json() -> anyhow::Result<()> {
        let app = Router::new().route("/detail", get(detail_handler));
        let request = axum::http::Request::builder()
            .uri("/detail?id=NaN")
            .body(Body::empty())?;
        let response = match app.oneshot(request).await {
            Ok(response) => response,
            Err(error) => match error {},
        };

        // 关键断言：参数类型错误不再走 Axum 默认纯文本响应。
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 4096).await?;
        let payload: ApiResponse<Value> = serde_json::from_slice(&body)?;
        assert_eq!(payload.code, StatusCode::BAD_REQUEST.as_u16());
        assert!(payload.msg.contains("查询参数解析失败"));

        Ok(())
    }

    #[test]
    fn missing_database_url_maps_to_service_unavailable() {
        let error = ApiError::from(anyhow::anyhow!("missing lowcode database url"));

        // 关键断言：依赖未配置属于服务不可用，而不是参数错误。
        assert_eq!(error.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn parse_i64_param_reports_original_value() {
        let error = parse_i64_param("id", "NaN").err();

        // 关键断言：数字转换错误保留非法原始入参，方便前端定位。
        assert_eq!(
            error.map(|error| error.message().to_string()),
            Some("参数[id]必须为数字，非法输入：NaN".to_string()),
        );
    }
}
