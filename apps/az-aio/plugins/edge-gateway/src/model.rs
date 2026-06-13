use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_edge_gateway_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct GatewayFlow {
    #[key]
    pub id: String,
    #[index]
    pub route: String,
    pub name: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayFlowSummary {
    pub id: String,
    pub route: String,
    pub name: String,
    pub status: String,
}

impl From<GatewayFlow> for GatewayFlowSummary {
    fn from(flow: GatewayFlow) -> Self {
        Self {
            id: flow.id,
            route: flow.route,
            name: flow.name,
            status: flow.status,
        }
    }
}
