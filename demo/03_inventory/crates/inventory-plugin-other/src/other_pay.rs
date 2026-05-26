use pay_api::{PayInterface, PayRequest};

pub(crate) struct OtherPay;

impl PayInterface for OtherPay {
    fn pay(&self, request: &PayRequest) {
        println!(
            "other pays order={}, amount_fen={}",
            request.order_no, request.amount_fen
        );
    }
}

inventory::submit! {
    pay_api::PayFactoryRegistration {
        factory: || Box::new(OtherPay),
    }
}
