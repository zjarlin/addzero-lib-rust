#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayRequest {
    pub order_no: String,
    pub amount_fen: u64,
}

impl PayRequest {
    pub fn new(order_no: impl Into<String>, amount_fen: u64) -> Self {
        Self {
            order_no: order_no.into(),
            amount_fen,
        }
    }
}

pub trait PayInterface: Send + Sync {
    fn code(&self) -> &'static str {
        let type_name = std::any::type_name::<Self>();
        type_name
            .rsplit_once("::")
            .map_or(type_name, |(module_path, _)| module_path)
    }
    fn pay(&self, request: &PayRequest);
}

/// 从 Rudi 上下文解析全部支付实现。
pub fn pay_interfaces(context: &mut rudi::Context) -> Vec<Box<dyn PayInterface>> {
    context
        .resolve_by_type::<Box<dyn PayInterface>>()
        .into_iter()
        .map(|pay_interface| {
            println!("registered pay interface: {}", pay_interface.code());
            pay_interface
        })
        .collect()
}
