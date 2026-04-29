#[cfg(target_arch = "wasm32")]
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use serde::de::DeserializeOwned;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestCredentials, RequestInit, RequestMode, Response};

#[cfg(target_arch = "wasm32")]
pub async fn get_json<T>(path: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    request_json::<(), T>("GET", path, None).await
}

#[cfg(target_arch = "wasm32")]
pub async fn post_json<B, T>(path: &str, body: &B) -> Result<T, String>
where
    B: Serialize,
    T: DeserializeOwned,
{
    request_json("POST", path, Some(body)).await
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_empty(path: &str) -> Result<(), String> {
    request_empty("DELETE", path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn post_empty<B>(path: &str, body: &B) -> Result<(), String>
where
    B: Serialize,
{
    let _: serde_json::Value = request_json("POST", path, Some(body)).await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn request_json<B, T>(method: &str, path: &str, body: Option<&B>) -> Result<T, String>
where
    B: Serialize,
    T: DeserializeOwned,
{
    let response = send_request(method, path, body).await?;
    let text = response_text(response).await?;
    serde_json::from_str(&text).map_err(|err| format!("解析响应失败：{err}"))
}

#[cfg(target_arch = "wasm32")]
async fn request_empty(method: &str, path: &str) -> Result<(), String> {
    let response = send_request::<()>(method, path, None).await?;
    if response.ok() {
        Ok(())
    } else {
        Err(response_text(response).await?)
    }
}

#[cfg(target_arch = "wasm32")]
async fn send_request<B>(method: &str, path: &str, body: Option<&B>) -> Result<Response, String>
where
    B: Serialize,
{
    let init = RequestInit::new();
    init.set_method(method);
    init.set_mode(RequestMode::Cors);
    init.set_credentials(RequestCredentials::Include);

    if let Some(body) = body {
        let encoded = serde_json::to_string(body).map_err(|err| format!("编码请求失败：{err}"))?;
        init.set_body(&JsValue::from_str(&encoded));
    }

    let request = Request::new_with_str_and_init(path, &init).map_err(js_error_to_string)?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(js_error_to_string)?;

    let window = web_sys::window().ok_or_else(|| "无法访问浏览器窗口".to_string())?;
    let promise = window.fetch_with_request(&request);
    let response = JsFuture::from(promise).await.map_err(js_error_to_string)?;
    let response: Response = response
        .dyn_into()
        .map_err(|_| "无法解析 HTTP 响应".to_string())?;

    if response.ok() {
        Ok(response)
    } else {
        Err(response_text(response).await?)
    }
}

#[cfg(target_arch = "wasm32")]
async fn response_text(response: Response) -> Result<String, String> {
    let text_promise = response.text().map_err(js_error_to_string)?;
    let text = JsFuture::from(text_promise)
        .await
        .map_err(js_error_to_string)?;
    Ok(text.as_string().unwrap_or_else(|| "请求失败".to_string()))
}

#[cfg(target_arch = "wasm32")]
fn js_error_to_string(err: JsValue) -> String {
    err.as_string()
        .unwrap_or_else(|| "浏览器请求失败".to_string())
}
