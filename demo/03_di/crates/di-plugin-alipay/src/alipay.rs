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

#[rudi::Transient(name = "alipay")]
fn alipay() -> Box<dyn PayInterface> {
    Box::new(Alipay)
}
