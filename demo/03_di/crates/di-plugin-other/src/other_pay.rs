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

#[rudi::Transient(name = "other")]
fn other_pay() -> Box<dyn PayInterface> {
    Box::new(OtherPay)
}
