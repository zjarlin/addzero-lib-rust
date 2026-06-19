use crate::http::HttpApiClient;
use crate::util::{
    canonical_query_string, canonical_uri, encode_url_component, hex_string, sha256_hex,
    trim_non_blank,
};
use crate::config::ApiConfig;
use anyhow::{Context, anyhow, bail};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST};
use reqwest::{Method, Url};
use serde_json::Value;
use sha2::Sha256;
use std::collections::BTreeMap;

/// 天眼查普通接口客户端。
///
/// 调用方需要提供已获取的 `Authorization` 和 `X-AUTH-TOKEN`；客户端只负责请求构造、
/// 响应解析和错误映射，不负责凭证获取或刷新。
#[derive(Clone, Debug)]
pub struct TianyanchaApi {
    authorization: String,
    auth_token: String,
    http: HttpApiClient,
}

impl TianyanchaApi {
    /// 使用显式凭证和 API 配置创建天眼查普通接口客户端。
    pub fn new(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
        config: ApiConfig,
    ) -> anyhow::Result<Self> {
        let authorization = authorization.into();
        let auth_token = auth_token.into();
        if trim_non_blank(Some(authorization.as_str())).is_none() {
            bail!("invalid config: tianyancha authorization cannot be blank");
        }
        if trim_non_blank(Some(auth_token.as_str())).is_none() {
            bail!("invalid config: tianyancha auth_token cannot be blank");
        }
        Ok(Self {
            authorization,
            auth_token,
            http: HttpApiClient::new(config)?,
        })
    }

    /// 按公司名称搜索企业列表。
    ///
    /// `page_num` 和 `page_size` 会向上收敛到至少 `1`；空公司名会返回配置错误。
    pub fn search_company(
        &self,
        company_name: impl AsRef<str>,
        page_num: usize,
        page_size: usize,
        sort_type: impl AsRef<str>,
    ) -> anyhow::Result<TianyanchaCompanySearchData> {
        let company_name = trim_non_blank(Some(company_name.as_ref())).ok_or_else(|| {
            anyhow!("invalid config: company_name cannot be blank")
        })?;
        let path = format!(
            "/services/v3/search/sNorV4/{}",
            encode_url_component(company_name)
        );
        let response =
            HttpApiClient::with_headers(self.http.get(path.as_str())?, &self.request_headers())?
                .query(&[
                    ("pageNum", page_num.max(1).to_string()),
                    ("pageSize", page_size.max(1).to_string()),
                    ("sortType", sort_type.as_ref().trim().to_owned()),
                ])
                .send()
                .context("failed to search tianyancha company")?;
        let response: TianyanchaSearchResponse = HttpApiClient::read_json(response)?;
        response.into_data("search tianyancha company")
    }

    /// 获取企业基础详情。
    pub fn get_base_info(&self, company_id: i64) -> anyhow::Result<TianyanchaCompanyDetail> {
        let path = format!("/services/v3/t/common/baseinfoV5/{company_id}");
        let response =
            HttpApiClient::with_headers(self.http.get(path.as_str())?, &self.request_headers())?
                .send()
                .context("failed to get tianyancha base info")?;
        let response: TianyanchaDetailResponse = HttpApiClient::read_json(response)?;
        response.into_data("get tianyancha base info")
    }

    fn request_headers(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("Authorization".to_owned(), self.authorization.clone()),
            ("X-AUTH-TOKEN".to_owned(), self.auth_token.clone()),
        ])
    }
}

/// 使用默认天眼查小程序接口地址和请求头创建客户端。
pub fn create_tianyancha_api(
    authorization: impl Into<String>,
    auth_token: impl Into<String>,
) -> anyhow::Result<TianyanchaApi> {
    let config = ApiConfig::builder("https://api9.tianyancha.com")
        .default_header(CONTENT_TYPE.as_str(), "application/json")
        .default_header(HOST.as_str(), "api9.tianyancha.com")
        .default_header(ACCEPT.as_str(), "*/*")
        .default_header("version", "TYC-XCX-WX")
        .default_header(
            "User-Agent",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 12_1_4 like Mac OS X)              AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/16D57              MicroMessenger/7.0.5(0x17000523) NetType/WIFI Language/zh_CN",
        )
        .default_header("Accept-Language", "zh-cn")
        .build()?;
    TianyanchaApi::new(authorization, auth_token, config)
}

/// 使用默认华为云 API Marketplace 地址创建天眼查签名版客户端。
pub fn create_tianyancha_huawei_api(
    access_key: impl Into<String>,
    secret_key: impl Into<String>,
) -> anyhow::Result<TianyanchaHuaweiApi> {
    let config = ApiConfig::builder("http://kzenterprisewmh.apistore.huaweicloud.com")
        .default_header(ACCEPT.as_str(), "application/json")
        .build()?;
    TianyanchaHuaweiApi::new(access_key, secret_key, config)
}

/// 天眼查公司搜索响应数据。
///
/// 未显式建模的上游字段会进入 `extra`，避免接口扩展时丢失排查信息。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TianyanchaCompanySearchData {
    #[serde(rename = "adviceQuery", default)]
    pub advice_query: Option<Value>,
    #[serde(rename = "companyCount", default)]
    pub company_count: Option<i64>,
    #[serde(rename = "companyHumanCount", default)]
    pub company_human_count: Option<i64>,
    #[serde(rename = "companyList", default)]
    pub company_list: Vec<TianyanchaCompany>,
    #[serde(rename = "companyTotal", default)]
    pub company_total: Option<i64>,
    #[serde(rename = "companyTotalPage", default)]
    pub company_total_page: Option<i64>,
    #[serde(rename = "companyTotalStr", default)]
    pub company_total_str: Option<String>,
    #[serde(rename = "humanCount", default)]
    pub human_count: Option<i64>,
    #[serde(rename = "modifiedQuery", default)]
    pub modified_query: Option<Value>,
    #[serde(rename = "searchContent", default)]
    pub search_content: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// 天眼查搜索列表中的公司摘要。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TianyanchaCompany {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(rename = "legalPersonName", default)]
    pub legal_person_name: Option<String>,
    #[serde(rename = "regStatus", default)]
    pub reg_status: Option<String>,
    #[serde(rename = "regCapital", default)]
    pub reg_capital: Option<String>,
    #[serde(rename = "creditCode", default)]
    pub credit_code: Option<String>,
    #[serde(rename = "phoneNum", default)]
    pub phone_num: Option<String>,
    #[serde(rename = "emailList", default)]
    pub email_list: Vec<String>,
    #[serde(rename = "companyOrgType", default)]
    pub company_org_type: Option<String>,
    #[serde(rename = "regLocation", default)]
    pub reg_location: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// 天眼查企业基础详情。
///
/// 字段命名跟随 Rust snake_case，serde rename 保持上游 JSON wire 兼容。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TianyanchaCompanyDetail {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(rename = "legalPersonName", default)]
    pub legal_person_name: Option<String>,
    #[serde(rename = "legalPersonId", default)]
    pub legal_person_id: Option<i64>,
    #[serde(rename = "regStatus", default)]
    pub reg_status: Option<String>,
    #[serde(rename = "creditCode", default)]
    pub credit_code: Option<String>,
    #[serde(rename = "companyCreditCode", default)]
    pub company_credit_code: Option<String>,
    #[serde(rename = "regCapital", default)]
    pub reg_capital: Option<String>,
    #[serde(rename = "regNumber", default)]
    pub reg_number: Option<String>,
    #[serde(rename = "companyOrgType", default)]
    pub company_org_type: Option<String>,
    #[serde(rename = "companyProfilePlainText", default)]
    pub company_profile_plain_text: Option<String>,
    #[serde(rename = "businessScope", default)]
    pub business_scope: Option<String>,
    #[serde(rename = "phoneNumber", default)]
    pub phone_number: Option<String>,
    #[serde(rename = "phoneList", default)]
    pub phone_list: Vec<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(rename = "emailList", default)]
    pub email_list: Vec<String>,
    #[serde(rename = "regLocation", default)]
    pub reg_location: Option<String>,
    #[serde(rename = "taxNumber", default)]
    pub tax_number: Option<String>,
    #[serde(rename = "estiblishTime", default)]
    pub estiblish_time: Option<i64>,
    #[serde(rename = "approvedTime", default)]
    pub approved_time: Option<i64>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// 天眼查华为云 API Marketplace 签名版客户端。
///
/// 客户端在每次请求时生成 `SDK-HMAC-SHA256` 签名头；调用方只传入 AK/SK，
/// 不需要手写 canonical request。
#[derive(Clone, Debug)]
pub struct TianyanchaHuaweiApi {
    access_key: String,
    secret_key: String,
    http: HttpApiClient,
}

impl TianyanchaHuaweiApi {
    pub fn new(
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        config: ApiConfig,
    ) -> anyhow::Result<Self> {
        let access_key = access_key.into();
        let secret_key = secret_key.into();
        if trim_non_blank(Some(access_key.as_str())).is_none() {
            bail!("invalid config: huawei access_key cannot be blank");
        }
        if trim_non_blank(Some(secret_key.as_str())).is_none() {
            bail!("invalid config: huawei secret_key cannot be blank");
        }
        Ok(Self {
            access_key,
            secret_key,
            http: HttpApiClient::new(config)?,
        })
    }

    /// 通过华为云接口按关键词搜索企业。
    ///
    /// `page_num` 和 `page_size` 会向上收敛到至少 `1`；空关键词会返回配置错误。
    pub fn search_companies(
        &self,
        keyword: impl AsRef<str>,
        page_num: usize,
        page_size: usize,
    ) -> anyhow::Result<TianyanchaHuaweiCompanySearchData> {
        let keyword = trim_non_blank(Some(keyword.as_ref()))
            .ok_or_else(|| anyhow!("invalid config: keyword cannot be blank"))?;
        let query = vec![
            ("keyword", keyword.to_owned()),
            ("pageNum", page_num.max(1).to_string()),
            ("pageSize", page_size.max(1).to_string()),
        ];
        let url = self
            .http
            .build_url("/api-mall/api/company_search/query", &query)?;
        let signed_headers = self.sign_headers(Method::GET.as_str(), &url, None, None)?;
        let response =
            HttpApiClient::with_headers(self.http.get_url(url), &signed_headers)?
                .send()
                .context("failed to search huawei tianyancha company")?;
        let response: TianyanchaHuaweiResponse = HttpApiClient::read_json(response)?;
        response.into_data("search huawei tianyancha company")
    }

    pub(crate) fn sign_headers(
        &self,
        method: &str,
        url: &Url,
        body: Option<&[u8]>,
        timestamp: Option<&str>,
    ) -> anyhow::Result<BTreeMap<String, String>> {
        let payload_hash = sha256_hex(body.unwrap_or_default());
        let host = url
            .host_str()
            .map(|host| match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_owned(),
            })
            .ok_or_else(|| anyhow!("invalid response: huawei url missing host"))?;
        let request_time = timestamp.map_or_else(
            || Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
            ToOwned::to_owned,
        );

        let canonical_uri = canonical_uri(url);
        let canonical_query = canonical_query_string(url);
        let canonical_headers = format!("host:{host}\nx-sdk-date:{request_time}\n");
        let signed_headers = "host;x-sdk-date";
        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let hashed_request = sha256_hex(canonical_request.as_bytes());
        let string_to_sign = format!("SDK-HMAC-SHA256\n{request_time}\n{hashed_request}");

        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .context("signature error")?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex_string(&mac.finalize().into_bytes());
        let authorization = format!(
            "SDK-HMAC-SHA256 Access={access_key}, SignedHeaders={signed_headers}, Signature={signature}",
            access_key = self.access_key
        );

        Ok(BTreeMap::from([
            (HOST.as_str().to_owned(), host),
            ("X-Sdk-Date".to_owned(), request_time),
            (AUTHORIZATION.as_str().to_owned(), authorization),
        ]))
    }
}

/// 华为云签名版企业搜索响应数据。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TianyanchaHuaweiCompanySearchData {
    #[serde(rename = "companyList", default)]
    pub company_list: Vec<TianyanchaHuaweiCompany>,
    #[serde(rename = "orderNo", default)]
    pub order_no: Option<String>,
    #[serde(rename = "pageInfo", default)]
    pub page_info: Option<TianyanchaHuaweiPageInfo>,
}

/// 华为云签名版企业摘要。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TianyanchaHuaweiCompany {
    #[serde(rename = "companyCode", default)]
    pub company_code: String,
    #[serde(rename = "companyName", default)]
    pub company_name: String,
    #[serde(rename = "companyStatus", default)]
    pub company_status: String,
    #[serde(rename = "creditNo", default)]
    pub credit_no: String,
    #[serde(rename = "establishDate", default)]
    pub establish_date: String,
    #[serde(rename = "legalPerson", default)]
    pub legal_person: String,
}

/// 华为云签名版分页信息。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TianyanchaHuaweiPageInfo {
    #[serde(rename = "pageIndex", default)]
    pub page_index: String,
    #[serde(rename = "pageSize", default)]
    pub page_size: String,
    #[serde(rename = "totalRecords", default)]
    pub total_records: String,
}

#[derive(Debug, serde::Deserialize)]
struct TianyanchaSearchResponse {
    #[serde(default)]
    data: Option<TianyanchaCompanySearchData>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    #[serde(rename = "vipMessage")]
    vip_message: Option<String>,
}

impl TianyanchaSearchResponse {
    fn into_data(self, action: &str) -> anyhow::Result<TianyanchaCompanySearchData> {
        if matches!(self.state.as_deref(), Some("ok")) {
            return self.data.ok_or_else(|| {
                anyhow!("invalid response: {action} returned ok without data")
            });
        }
        bail!(
            "invalid response: {action} failed: {}",
            self.message
                .or(self.vip_message)
                .unwrap_or_else(|| "unknown error".to_owned())
        )
    }
}

#[derive(Debug, serde::Deserialize)]
struct TianyanchaDetailResponse {
    #[serde(default)]
    data: Option<TianyanchaCompanyDetail>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(rename = "errorMessage", default)]
    error_message: Option<Value>,
}

impl TianyanchaDetailResponse {
    fn into_data(self, action: &str) -> anyhow::Result<TianyanchaCompanyDetail> {
        if matches!(self.state.as_deref(), Some("ok")) {
            return self.data.ok_or_else(|| {
                anyhow!("invalid response: {action} returned ok without data")
            });
        }
        let error_message = self.error_message.and_then(|value| match value {
            Value::Null => None,
            Value::String(value) => Some(value),
            other => Some(other.to_string()),
        });
        bail!(
            "invalid response: {action} failed: {}",
            self.message
                .or(error_message)
                .unwrap_or_else(|| "unknown error".to_owned())
        )
    }
}

#[derive(Debug, serde::Deserialize)]
struct TianyanchaHuaweiResponse {
    #[serde(default)]
    code: i32,
    #[serde(default)]
    data: Option<TianyanchaHuaweiCompanySearchData>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    success: Option<bool>,
}

impl TianyanchaHuaweiResponse {
    fn into_data(self, action: &str) -> anyhow::Result<TianyanchaHuaweiCompanySearchData> {
        if self.code == 200 || self.success == Some(true) {
            return self.data.ok_or_else(|| {
                anyhow!("invalid response: {action} returned success without data")
            });
        }
        bail!(
            "invalid response: {action} failed: {}",
            self.msg.unwrap_or_else(|| format!("code={}", self.code))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Url;

    #[test]
    fn company_search_data_round_trips_with_extra_fields() {
        let value = TianyanchaCompanySearchData {
            advice_query: Some(Value::String("rust".to_owned())),
            company_count: Some(2),
            company_human_count: None,
            company_list: vec![TianyanchaCompany {
                id: 7,
                name: "Demo".to_owned(),
                alias: Some("demo".to_owned()),
                legal_person_name: Some("Alice".to_owned()),
                reg_status: Some("active".to_owned()),
                reg_capital: Some("100".to_owned()),
                credit_code: Some("ABC".to_owned()),
                phone_num: None,
                email_list: vec!["demo@example.com".to_owned()],
                company_org_type: None,
                reg_location: Some("Shanghai".to_owned()),
                logo: None,
                extra: BTreeMap::from([("custom".to_owned(), Value::String("x".to_owned()))]),
            }],
            company_total: Some(2),
            company_total_page: Some(1),
            company_total_str: Some("2".to_owned()),
            human_count: Some(1),
            modified_query: None,
            search_content: Some("demo".to_owned()),
            extra: BTreeMap::from([("trace".to_owned(), Value::String("ok".to_owned()))]),
        };

        let json = serde_json::to_string(&value).expect("value should serialize");
        let decoded: TianyanchaCompanySearchData =
            serde_json::from_str(&json).expect("value should deserialize");

        assert_eq!(value, decoded);
    }

    #[test]
    fn huawei_signature_canonicalizes_query_string() -> Result<(), Box<dyn std::error::Error>> {
        let api = TianyanchaHuaweiApi::new(
            "ak-demo",
            "sk-demo",
            ApiConfig::builder("http://example.com").build()?,
        )?;
        let url = Url::parse(
            "http://example.com/api-mall/api/company_search/query?pageSize=20&keyword=%E6%B5%8B%E8%AF%95&pageNum=2",
        )?;
        let headers = api.sign_headers("GET", &url, None, Some("20260421T120000Z"))?;

        assert_eq!(
            crate::util::canonical_query_string(&url),
            "keyword=%E6%B5%8B%E8%AF%95&pageNum=2&pageSize=20"
        );
        assert_eq!(
            headers.get("X-Sdk-Date").map(String::as_str),
            Some("20260421T120000Z")
        );
        assert!(
            headers
                .get("authorization")
                .or_else(|| headers.get("Authorization"))
                .is_some()
        );
        Ok(())
    }
}
