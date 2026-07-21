use pay_api::{PayInterface, PayRequest};

pub(crate) struct WechatPay;

impl PayInterface for WechatPay {
    fn pay(&self, request: &PayRequest) {
        println!(
            "wechat pays order={}, amount_fen={}",
            request.order_no, request.amount_fen
        );
    }
}

#[rudi::Transient(name = "wechat")]
fn wechat_pay() -> Box<dyn PayInterface> {
    Box::new(WechatPay)
}
