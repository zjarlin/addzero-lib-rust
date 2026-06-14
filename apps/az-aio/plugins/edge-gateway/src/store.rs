use anyhow::bail;
use az_aio_platform::db;
use shaku::{Component, Interface, module};
use toasty::stmt::{List, Query};
use uuid::Uuid;

use crate::{
    model::{GatewayFlow, GatewayFlowSummary, TABLE_NAME_PREFIX},
};

#[derive(Clone)]
pub struct EdgeGatewayStore {
    db: db::SharedDb,
}

impl EdgeGatewayStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let database_url = db::verify_database_url(database_url)?;
        let toasty = toasty::Db::builder()
            .models(toasty::models!(GatewayFlow))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        toasty.push_schema().await?;
        Ok(Self {
            db: db::SharedDb::new(toasty),
        })
    }

    pub async fn list_flows(&self) -> anyhow::Result<Vec<GatewayFlowSummary>> {
        let mut db = self.db.lock().await;
        let flows = Query::<List<GatewayFlow>>::all().exec(&mut *db).await?;
        Ok(flows.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_flow(
        &self,
        input: GatewayFlowInput,
    ) -> anyhow::Result<GatewayFlowSummary> {
        validate_gateway_flow_input(&input)?;
        let id = normalized_id(input.id);
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let existing = Query::<List<GatewayFlow>>::filter(GatewayFlow::fields().id().eq(&id))
            .first()
            .exec(&mut *db)
            .await?;
        let flow = match existing {
            Some(_) => {
                GatewayFlow::filter(GatewayFlow::fields().id().eq(&id))
                    .update()
                    .route(input.route)
                    .name(input.name)
                    .status(input.status.unwrap_or_else(|| "active".to_string()))
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<GatewayFlow>>::filter(GatewayFlow::fields().id().eq(&id))
                    .one()
                    .exec(&mut *db)
                    .await?
            }
            None => {
                GatewayFlow::create()
                    .id(id)
                    .route(input.route)
                    .name(input.name)
                    .status(input.status.unwrap_or_else(|| "active".to_string()))
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(flow.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayFlowInput {
    pub id: Option<String>,
    pub route: String,
    pub name: String,
    pub status: Option<String>,
}

pub trait EdgeGatewayService: Interface {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Component)]
#[shaku(interface = EdgeGatewayService)]
pub struct EdgeGatewayServiceImpl;

impl EdgeGatewayService for EdgeGatewayServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "edge-gateway"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

module! {
    pub EdgeGatewayModule {
        components = [EdgeGatewayServiceImpl],
        providers = []
    }
}

pub fn build_edge_gateway_module() -> EdgeGatewayModule {
    EdgeGatewayModule::builder().build()
}

pub fn validate_gateway_flow_input(input: &GatewayFlowInput) -> anyhow::Result<()> {
    if input.name.trim().is_empty() {
        bail!("gateway flow name must not be blank");
    }
    if input.route.trim().is_empty() {
        bail!("gateway flow route must not be blank");
    }
    Ok(())
}

fn normalized_id(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use shaku::HasComponent;

    use super::*;

    #[test]
    fn rejects_blank_gateway_flow_input() {
        let input = GatewayFlowInput {
            id: None,
            route: "".to_string(),
            name: "Proxy".to_string(),
            status: None,
        };
        let error = validate_gateway_flow_input(&input).unwrap_err();
        assert_eq!(error.to_string(), "gateway flow route must not be blank");
    }

    #[test]
    fn shaku_module_resolves_service() {
        let module = build_edge_gateway_module();
        let service: &dyn EdgeGatewayService = module.resolve_ref();
        assert_eq!(service.plugin_id(), "edge-gateway");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
