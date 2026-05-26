use pay_api::{PayInterface, PayRequest};

pub(crate) struct Alipay;

impl PayInterface for Alipay {
    fn pay(&self, request: &PayRequest) {
        println!(
            "alipay pays order={}, amount_fen={}",
            request.order_no, request.amount_fen
        );
    }
}

inventory::submit! {
    pay_api::PayFactoryRegistration {
        factory: || Box::new(Alipay),
    }
}
