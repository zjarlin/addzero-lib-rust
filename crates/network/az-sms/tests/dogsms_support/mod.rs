#![allow(dead_code)]

use az_sms::dogsms::client::{DogSmsClient, DogSmsConfig};
use std::env;
use uuid::Uuid;

pub const DOGSMS_API_KEY_ENV: &str = "DOGSMS_API_KEY";
pub const DOGSMS_TEST_SERVICE_CODE: &str = "telegram";
pub const DOGSMS_TEST_COUNTRY_CODE: &str = "US";
pub const DOGSMS_TEST_RENTAL_COUNTRY_CODE: &str = "GB";
pub const DOGSMS_TEST_RENTAL_MONTHS: u8 = 1;
pub const DOGSMS_EXISTING_REQUEST_ID_ENV: &str = "DOGSMS_EXISTING_REQUEST_ID";
pub const DOGSMS_CANCEL_REQUEST_ID_ENV: &str = "DOGSMS_CANCEL_REQUEST_ID";

pub fn live_client() -> DogSmsClient {
    let api_key = env::var(DOGSMS_API_KEY_ENV)
        .unwrap_or_else(|_| panic!("{DOGSMS_API_KEY_ENV} is required for DogSMS live API tests"));

    DogSmsClient::new(DogSmsConfig::builder(api_key).build().unwrap()).unwrap()
}

pub fn existing_request_id() -> String {
    env::var(DOGSMS_EXISTING_REQUEST_ID_ENV).unwrap_or_else(|_| {
        panic!("{DOGSMS_EXISTING_REQUEST_ID_ENV} is required for this DogSMS live API test")
    })
}

pub fn cancel_request_id() -> String {
    env::var(DOGSMS_CANCEL_REQUEST_ID_ENV).unwrap_or_else(|_| {
        panic!("{DOGSMS_CANCEL_REQUEST_ID_ENV} is required for this DogSMS live API test")
    })
}

pub fn idempotency_key(label: &str) -> String {
    format!("az-sms-{label}-{}", Uuid::new_v4())
}
