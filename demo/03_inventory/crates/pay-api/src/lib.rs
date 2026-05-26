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

pub struct PayFactoryRegistration {
    pub factory: fn() -> Box<dyn PayInterface>,
}

inventory::collect!(PayFactoryRegistration);

pub fn pay_interfaces() -> Vec<Box<dyn PayInterface>> {
    inventory::iter::<PayFactoryRegistration>
        .into_iter()
        .map(|registration| (registration.factory)())
        .map(|pay_interface| {
            println!("registered pay interface: {}", pay_interface.code());
            pay_interface
        })
        .collect()
}
