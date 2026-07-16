use az_aio_platform::core::db::ToastyModelContribution;
use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_edge_gateway_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_edge_gateway_gateway_flows"]
pub struct GatewayFlow {
    #[key]
    pub id: String,
    #[index]
    pub route: String,
    pub name: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_edge_gateway_gateway_route_definitions"]
pub struct GatewayRouteDefinition {
    #[key]
    pub id: String,
    #[index]
    pub route: String,
    #[index]
    pub method: String,
    pub name: String,
    pub status: String,
    pub auth_required: String,
    pub script_language: String,
    pub script_code: String,
    pub request_example: String,
    pub response_template: String,
    pub notes: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_edge_gateway_edge_api_token_records"]
pub struct EdgeApiTokenRecord {
    #[key]
    pub id: String,
    #[index]
    pub token_hash: String,
    pub name: String,
    pub allowed_routes_json: String,
    pub status: String,
    pub expires_at_epoch_secs: String,
    pub last_used_at_epoch_secs: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_edge_gateway_edge_usage_record_rows"]
pub struct EdgeUsageRecordRow {
    #[key]
    pub id: String,
    #[index]
    pub token_id: String,
    #[index]
    pub route: String,
    pub asset_id: String,
    pub status_code: String,
    pub request_units: String,
    pub duration_ms: String,
    pub created_at_epoch_secs: String,
}

#[rudi::Singleton(name = "edge-gateway-toasty-models")]
pub fn edge_gateway_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(toasty::models!(
        GatewayFlow,
        GatewayRouteDefinition,
        EdgeApiTokenRecord,
        EdgeUsageRecordRow
    ))
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRouteSummary {
    pub id: String,
    pub route: String,
    pub method: String,
    pub name: String,
    pub status: String,
    pub auth_required: bool,
    pub script_language: String,
    pub script_code: String,
    pub request_example: String,
    pub response_template: String,
    pub notes: String,
    pub updated_at: String,
}

impl From<GatewayRouteDefinition> for GatewayRouteSummary {
    fn from(route: GatewayRouteDefinition) -> Self {
        Self {
            id: route.id,
            route: route.route,
            method: route.method,
            name: route.name,
            status: route.status,
            auth_required: route.auth_required == "true",
            script_language: route.script_language,
            script_code: route.script_code,
            request_example: route.request_example,
            response_template: route.response_template,
            notes: route.notes,
            updated_at: route.updated_at,
        }
    }
}
