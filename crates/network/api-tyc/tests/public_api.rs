use az_api_tyc::client::{SearchCompanyOptions, TycApi};
use az_api_tyc::detail::CompanyDetailData;
use az_api_tyc::headers::TycCredentials;
use az_api_tyc::search::SearchRes;

#[test]
fn credentials_should_reject_blank_values() {
    assert!(TycCredentials::new("", "token").is_err());
}

#[test]
fn search_options_should_reject_zero_page_size() {
    let options = SearchCompanyOptions::new(1, 0);
    assert!(options.validate().is_err());
}

#[test]
fn detail_helpers_should_map_known_company_type_codes() {
    assert_eq!(CompanyDetailData::parse_company_type(1), Some("公司"));
}

#[test]
fn search_response_should_deserialize_minimal_payload() -> anyhow::Result<()> {
    let response: SearchRes = serde_json::from_str(
        r#"{
          "data": { "companyList": [{ "id": 42, "name": "示例公司" }], "companyTotal": 1 },
          "isLogin": 0,
          "message": "ok",
          "special": "111",
          "state": "ok",
          "vipMessage": ""
        }"#,
    )?;
    assert_eq!(response.data.company_list[0].id, 42);
    Ok(())
}

#[test]
fn client_should_build_with_explicit_credentials() -> anyhow::Result<()> {
    let credentials = TycCredentials::new("auth", "token")?;
    let _api = TycApi::new(credentials)?;
    Ok(())
}
