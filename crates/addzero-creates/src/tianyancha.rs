use crate::http::HttpApiClient;
use crate::util::{
    canonical_query_string, canonical_uri, encode_url_component, hex_string, sha256_hex,
    trim_non_blank,
};
use crate::{ApiConfig, CreatesError, CreatesResult};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header::{AUTHORIZATION, HOST};
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct TianyanchaApi {
    authorization: String,
    auth_token: String,
    http: HttpApiClient,
}

impl TianyanchaApi {
    pub fn new(
        authorization: impl Into<String>,
        auth_token: impl Into<String>,
        config: ApiConfig,
    ) -> CreatesResult<Self> {
        let authorization = authorization.into();
        let auth_token = auth_token.into();
        if trim_non_blank(Some(authorization.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "tianyancha authorization cannot be blank".to_owned(),
            ));
        }
        if trim_non_blank(Some(auth_token.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "tianyancha auth_token cannot be blank".to_owned(),
            ));
        }
        Ok(Self {
            authorization,
            auth_token,
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search_company(
        &self,
        company_name: impl AsRef<str>,
        page_num: usize,
        page_size: usize,
        sort_type: impl AsRef<str>,
    ) -> CreatesResult<TianyanchaCompanySearchData> {
        let company_name = trim_non_blank(Some(company_name.as_ref())).ok_or_else(|| {
            CreatesError::InvalidConfig("company_name cannot be blank".to_owned())
        })?;
        let path = format!(
            "/services/v3/search/sNorV4/{}",
            encode_url_component(company_name)
        );
        let response: TianyanchaSearchResponse = self.http.get_json_with_headers(
            path.as_str(),
            &[
                ("pageNum", page_num.max(1).to_string()),
                ("pageSize", page_size.max(1).to_string()),
                ("sortType", sort_type.as_ref().trim().to_owned()),
            ],
            &self.request_headers(),
        )?;
        response.into_data("search tianyancha company")
    }

    pub fn get_base_info(&self, company_id: i64) -> CreatesResult<TianyanchaCompanyDetail> {
        let path = format!("/services/v3/t/common/baseinfoV5/{company_id}");
        let response: TianyanchaDetailResponse =
            self.http
                .get_json_with_headers(path.as_str(), &[], &self.request_headers())?;
        response.into_data("get tianyancha base info")
    }

    fn request_headers(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("Authorization".to_owned(), self.authorization.clone()),
            ("X-AUTH-TOKEN".to_owned(), self.auth_token.clone()),
        ])
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
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
    ) -> CreatesResult<Self> {
        let access_key = access_key.into();
        let secret_key = secret_key.into();
        if trim_non_blank(Some(access_key.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "huawei access_key cannot be blank".to_owned(),
            ));
        }
        if trim_non_blank(Some(secret_key.as_str())).is_none() {
            return Err(CreatesError::InvalidConfig(
                "huawei secret_key cannot be blank".to_owned(),
            ));
        }
        Ok(Self {
            access_key,
            secret_key,
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search_companies(
        &self,
        keyword: impl AsRef<str>,
        page_num: usize,
        page_size: usize,
    ) -> CreatesResult<TianyanchaHuaweiCompanySearchData> {
        let keyword = trim_non_blank(Some(keyword.as_ref()))
            .ok_or_else(|| CreatesError::InvalidConfig("keyword cannot be blank".to_owned()))?;
        let query = vec![
            ("keyword", keyword.to_owned()),
            ("pageNum", page_num.max(1).to_string()),
            ("pageSize", page_size.max(1).to_string()),
        ];
        let url = self
            .http
            .build_url("/api-mall/api/company_search/query", &query)?;
        let signed_headers = self.sign_headers(Method::GET.as_str(), &url, None, None)?;
        let response: TianyanchaHuaweiResponse =
            self.http.get_json_url_with_headers(url, &signed_headers)?;
        response.into_data("search huawei tianyancha company")
    }

    pub(crate) fn sign_headers(
        &self,
        method: &str,
        url: &Url,
        body: Option<&[u8]>,
        timestamp: Option<&str>,
    ) -> CreatesResult<BTreeMap<String, String>> {
        let payload_hash = sha256_hex(body.unwrap_or_default());
        let host = url
            .host_str()
            .map(|host| match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_owned(),
            })
            .ok_or_else(|| CreatesError::InvalidResponse("huawei url missing host".to_owned()))?;
        let request_time = timestamp
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| Utc::now().format("%Y%m%dT%H%M%SZ").to_string());

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
            .map_err(|error| CreatesError::Signature(error.to_string()))?;
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TianyanchaHuaweiCompanySearchData {
    #[serde(rename = "companyList", default)]
    pub company_list: Vec<TianyanchaHuaweiCompany>,
    #[serde(rename = "orderNo", default)]
    pub order_no: Option<String>,
    #[serde(rename = "pageInfo", default)]
    pub page_info: Option<TianyanchaHuaweiPageInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TianyanchaHuaweiPageInfo {
    #[serde(rename = "pageIndex", default)]
    pub page_index: String,
    #[serde(rename = "pageSize", default)]
    pub page_size: String,
    #[serde(rename = "totalRecords", default)]
    pub total_records: String,
}

#[derive(Debug, Deserialize)]
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
    fn into_data(self, action: &str) -> CreatesResult<TianyanchaCompanySearchData> {
        if matches!(self.state.as_deref(), Some("ok")) {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned ok without data"))
            });
        }
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message
                .or(self.vip_message)
                .unwrap_or_else(|| "unknown error".to_owned())
        )))
    }
}

#[derive(Debug, Deserialize)]
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
    fn into_data(self, action: &str) -> CreatesResult<TianyanchaCompanyDetail> {
        if matches!(self.state.as_deref(), Some("ok")) {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned ok without data"))
            });
        }
        let error_message = self.error_message.and_then(|value| match value {
            Value::Null => None,
            Value::String(value) => Some(value),
            other => Some(other.to_string()),
        });
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.message
                .or(error_message)
                .unwrap_or_else(|| "unknown error".to_owned())
        )))
    }
}

#[derive(Debug, Deserialize)]
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
    fn into_data(self, action: &str) -> CreatesResult<TianyanchaHuaweiCompanySearchData> {
        if self.code == 200 || self.success == Some(true) {
            return self.data.ok_or_else(|| {
                CreatesError::InvalidResponse(format!("{action} returned success without data"))
            });
        }
        Err(CreatesError::InvalidResponse(format!(
            "{action} failed: {}",
            self.msg.unwrap_or_else(|| format!("code={}", self.code))
        )))
    }
}
