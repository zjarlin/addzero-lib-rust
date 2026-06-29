use anyhow::{bail, Context as AnyhowContext};
use az_aio_platform::core::db;
use az_str::api::normalized_id_or_else;
use rudi::{Context, DynProvider, Module, modules, providers, singleton};
use std::{collections::BTreeSet, sync::Arc};
use toasty::stmt::{List, Query};

use crate::backend::{
    auth::{
        now_epoch_secs, token_hash, EdgeApiToken, EdgeAuthorizedToken, EdgeUsageRecord,
        DEMO_WEATHER_TOKEN,
    },
    model::{
        EdgeApiTokenRecord, EdgeUsageRecordRow, GatewayFlow, GatewayFlowSummary, GatewayRouteDefinition, GatewayRouteSummary, TABLE_NAME_PREFIX,
    },
    weather_asset::WEATHER_CURRENT_ROUTE,
};

#[derive(Clone)]
pub struct EdgeGatewayStore {
    db: db::Db,
}

impl EdgeGatewayStore {
    pub fn from_shared(db: db::Db) -> Self {
        Self { db }
    }


    pub async fn ensure_builtin_weather_route(&self) -> anyhow::Result<()> {
        self.upsert_route_definition(GatewayRouteInput {
            id: Some("route_weather_current".to_string()),
            route: WEATHER_CURRENT_ROUTE.to_string(),
            method: "POST".to_string(),
            name: "Weather Current API".to_string(),
            status: Some("active".to_string()),
            auth_required: Some(true),
            script_language: Some("json-template".to_string()),
            script_code: "return weather.current(request);".to_string(),
            request_example: r#"{"latitude":31.2304,"longitude":121.4737,"timezone":"Asia/Shanghai"}"#.to_string(),
            response_template: r#"{"temperatureCelsius":"number","windSpeedKmh":"number"}"#.to_string(),
            notes: "Built-in callable weather asset backed by Open-Meteo.".to_string(),
        })
        .await
        .map(|_| ())
    }

    pub async fn list_route_definitions(&self) -> anyhow::Result<Vec<GatewayRouteSummary>> {
        let mut db = self.db.lock().await;
        let routes = Query::<List<GatewayRouteDefinition>>::all()
            .exec(&mut *db)
            .await?;
        Ok(routes.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_route_definition(
        &self,
        input: GatewayRouteInput,
    ) -> anyhow::Result<GatewayRouteSummary> {
        validate_gateway_route_input(&input)?;
        let id = normalized_id_or_else(input.id, db::new_uuid_id);
        let now = db::timestamp_secs();
        let status = input.status.unwrap_or_else(|| "draft".to_string());
        let auth_required = input.auth_required.unwrap_or(true).to_string();
        let script_language = input
            .script_language
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "javascript".to_string());
        let mut db = self.db.lock().await;
        let existing = Query::<List<GatewayRouteDefinition>>::filter(
            GatewayRouteDefinition::fields().id().eq(&id),
        )
        .first()
        .exec(&mut *db)
        .await?;
        let route = match existing {
            Some(_) => {
                GatewayRouteDefinition::filter(GatewayRouteDefinition::fields().id().eq(&id))
                    .update()
                    .route(input.route)
                    .method(input.method)
                    .name(input.name)
                    .status(status)
                    .auth_required(auth_required)
                    .script_language(script_language)
                    .script_code(input.script_code)
                    .request_example(input.request_example)
                    .response_template(input.response_template)
                    .notes(input.notes)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<GatewayRouteDefinition>>::filter(
                    GatewayRouteDefinition::fields().id().eq(&id),
                )
                .one()
                .exec(&mut *db)
                .await?
            }
            None => {
                GatewayRouteDefinition::create()
                    .id(id)
                    .route(input.route)
                    .method(input.method)
                    .name(input.name)
                    .status(status)
                    .auth_required(auth_required)
                    .script_language(script_language)
                    .script_code(input.script_code)
                    .request_example(input.request_example)
                    .response_template(input.response_template)
                    .notes(input.notes)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(route.into())
    }

    pub async fn ensure_demo_weather_token(&self) -> anyhow::Result<()> {
        self.upsert_api_token(EdgeApiToken::active(
            "tok_weather_demo",
            "demo weather",
            DEMO_WEATHER_TOKEN,
            [WEATHER_CURRENT_ROUTE],
        ))
        .await
    }

    pub async fn upsert_api_token(&self, token: EdgeApiToken) -> anyhow::Result<()> {
        let allowed_routes_json = serde_json::to_string(&token.allowed_routes)
            .context("serialize edge token route scopes")?;
        let now = db::timestamp_secs();
        let expires_at = token
            .expires_at_epoch_secs
            .map(|value| value.to_string())
            .unwrap_or_default();
        let last_used_at = token
            .last_used_at_epoch_secs
            .map(|value| value.to_string())
            .unwrap_or_default();
        let mut db = self.db.lock().await;
        let existing = Query::<List<EdgeApiTokenRecord>>::filter(
            EdgeApiTokenRecord::fields().id().eq(&token.id),
        )
        .first()
        .exec(&mut *db)
        .await?;
        if existing.is_some() {
            EdgeApiTokenRecord::filter(EdgeApiTokenRecord::fields().id().eq(&token.id))
                .update()
                .token_hash(token.token_hash)
                .name(token.name)
                .allowed_routes_json(allowed_routes_json)
                .status(token.status)
                .expires_at_epoch_secs(expires_at)
                .last_used_at_epoch_secs(last_used_at)
                .updated_at(now)
                .exec(&mut *db)
                .await?;
        } else {
            EdgeApiTokenRecord::create()
                .id(token.id)
                .token_hash(token.token_hash)
                .name(token.name)
                .allowed_routes_json(allowed_routes_json)
                .status(token.status)
                .expires_at_epoch_secs(expires_at)
                .last_used_at_epoch_secs(last_used_at)
                .updated_at(now)
                .exec(&mut *db)
                .await?;
        }
        Ok(())
    }

    pub async fn authorize_token(
        &self,
        route: &str,
        cleartext_token: &str,
    ) -> anyhow::Result<EdgeAuthorizedToken> {
        let now = now_epoch_secs();
        let hash = token_hash(cleartext_token);
        let mut db = self.db.lock().await;
        let Some(row) = Query::<List<EdgeApiTokenRecord>>::filter(
            EdgeApiTokenRecord::fields().token_hash().eq(&hash),
        )
        .first()
        .exec(&mut *db)
        .await? else {
            bail!("unauthorized edge asset call: invalid_token");
        };

        let token = token_from_record(row).context("decode edge token record")?;
        if !token.allows_route(route, now) {
            bail!("forbidden edge asset call: forbidden_route");
        }
        EdgeApiTokenRecord::filter(EdgeApiTokenRecord::fields().id().eq(&token.id))
            .update()
            .last_used_at_epoch_secs(now.to_string())
            .updated_at(db::timestamp_secs())
            .exec(&mut *db)
            .await?;
        Ok(EdgeAuthorizedToken {
            token_id: token.id,
            token_name: token.name,
        })
    }

    pub async fn record_usage(&self, record: EdgeUsageRecord) -> anyhow::Result<()> {
        let mut db = self.db.lock().await;
        EdgeUsageRecordRow::create()
            .id(db::new_uuid_id())
            .token_id(record.token_id)
            .route(record.route)
            .asset_id(record.asset_id)
            .status_code(record.status_code.to_string())
            .request_units(record.request_units.to_string())
            .duration_ms(record.duration_ms.to_string())
            .created_at_epoch_secs(record.created_at_epoch_secs.to_string())
            .exec(&mut *db)
            .await?;
        Ok(())
    }

    pub async fn list_usage_records(&self) -> anyhow::Result<Vec<EdgeUsageRecord>> {
        let mut db = self.db.lock().await;
        let rows = Query::<List<EdgeUsageRecordRow>>::all().exec(&mut *db).await?;
        Ok(rows.into_iter().map(usage_record_from_row).collect())
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
        let id = normalized_id_or_else(input.id, db::new_uuid_id);
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


pub fn edge_gateway_models() -> toasty::ModelSet {
    toasty::models!(
        GatewayFlow,
        GatewayRouteDefinition,
        EdgeApiTokenRecord,
        EdgeUsageRecordRow
    )
}

pub const EDGE_GATEWAY_BOOTSTRAP_SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS biz_edge_gateway_gateway_flows (id TEXT PRIMARY KEY, route TEXT NOT NULL, name TEXT NOT NULL, status TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_edge_gateway_gateway_flows_route_idx ON biz_edge_gateway_gateway_flows (route)",
    "CREATE TABLE IF NOT EXISTS biz_edge_gateway_gateway_route_definitions (id TEXT PRIMARY KEY, route TEXT NOT NULL, method TEXT NOT NULL, name TEXT NOT NULL, status TEXT NOT NULL, auth_required TEXT NOT NULL, script_language TEXT NOT NULL, script_code TEXT NOT NULL, request_example TEXT NOT NULL, response_template TEXT NOT NULL, notes TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_edge_gateway_gateway_route_definitions_route_idx ON biz_edge_gateway_gateway_route_definitions (route)",
    "CREATE INDEX IF NOT EXISTS biz_edge_gateway_gateway_route_definitions_method_idx ON biz_edge_gateway_gateway_route_definitions (method)",
    "CREATE TABLE IF NOT EXISTS biz_edge_gateway_edge_api_token_records (id TEXT PRIMARY KEY, token_hash TEXT NOT NULL, name TEXT NOT NULL, allowed_routes_json TEXT NOT NULL, status TEXT NOT NULL, expires_at_epoch_secs TEXT NOT NULL, last_used_at_epoch_secs TEXT NOT NULL, updated_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_edge_gateway_edge_api_token_records_token_hash_idx ON biz_edge_gateway_edge_api_token_records (token_hash)",
    "CREATE TABLE IF NOT EXISTS biz_edge_gateway_edge_usage_record_rows (id TEXT PRIMARY KEY, token_id TEXT NOT NULL, route TEXT NOT NULL, asset_id TEXT NOT NULL, status_code TEXT NOT NULL, request_units TEXT NOT NULL, duration_ms TEXT NOT NULL, created_at_epoch_secs TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS biz_edge_gateway_edge_usage_record_rows_token_id_idx ON biz_edge_gateway_edge_usage_record_rows (token_id)",
    "CREATE INDEX IF NOT EXISTS biz_edge_gateway_edge_usage_record_rows_route_idx ON biz_edge_gateway_edge_usage_record_rows (route)",
];


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayRouteInput {
    pub id: Option<String>,
    pub route: String,
    pub method: String,
    pub name: String,
    pub status: Option<String>,
    pub auth_required: Option<bool>,
    pub script_language: Option<String>,
    pub script_code: String,
    pub request_example: String,
    pub response_template: String,
    pub notes: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayFlowInput {
    pub id: Option<String>,
    pub route: String,
    pub name: String,
    pub status: Option<String>,
}

fn token_from_record(row: EdgeApiTokenRecord) -> anyhow::Result<EdgeApiToken> {
    let allowed_routes: BTreeSet<String> = serde_json::from_str(&row.allowed_routes_json)
        .context("parse edge token route scopes")?;
    Ok(EdgeApiToken {
        id: row.id,
        name: row.name,
        token_hash: row.token_hash,
        allowed_routes,
        status: row.status,
        expires_at_epoch_secs: parse_optional_u64(&row.expires_at_epoch_secs),
        last_used_at_epoch_secs: parse_optional_u64(&row.last_used_at_epoch_secs),
    })
}

fn usage_record_from_row(row: EdgeUsageRecordRow) -> EdgeUsageRecord {
    EdgeUsageRecord {
        token_id: row.token_id,
        route: row.route,
        asset_id: row.asset_id,
        status_code: row.status_code.parse().unwrap_or_default(),
        request_units: row.request_units.parse().unwrap_or_default(),
        duration_ms: row.duration_ms.parse().unwrap_or_default(),
        created_at_epoch_secs: row.created_at_epoch_secs.parse().unwrap_or_default(),
    }
}

fn parse_optional_u64(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        value.parse().ok()
    }
}

pub trait EdgeGatewayService: Send + Sync {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Clone)]
pub struct EdgeGatewayServiceImpl;

impl EdgeGatewayService for EdgeGatewayServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "edge-gateway"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

pub struct EdgeGatewayModule;

impl Module for EdgeGatewayModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| Arc::new(EdgeGatewayServiceImpl) as Arc<dyn EdgeGatewayService>),
            singleton(|cx| EdgeGatewayStore::from_shared(cx.resolve::<db::Db>())),
        ]
    }
}

pub fn build_edge_gateway_context() -> Context {
    Context::create(modules![EdgeGatewayModule])
}

pub fn build_edge_gateway_context_with_db(shared_db: db::Db) -> Context {
    Context::options()
        .singleton(shared_db)
        .create(modules![EdgeGatewayModule])
}


pub fn validate_gateway_route_input(input: &GatewayRouteInput) -> anyhow::Result<()> {
    if input.name.trim().is_empty() {
        bail!("gateway route name must not be blank");
    }
    if input.route.trim().is_empty() {
        bail!("gateway route path must not be blank");
    }
    if !input.route.starts_with('/') {
        bail!("gateway route path must start with /");
    }
    if !matches!(input.method.as_str(), "GET" | "POST") {
        bail!("gateway route method must be GET or POST");
    }
    Ok(())
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

#[cfg(test)]
mod tests {
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
    fn rudi_context_resolves_service() {
        let mut context = build_edge_gateway_context();
        let service = context.resolve::<Arc<dyn EdgeGatewayService>>();
        assert_eq!(service.plugin_id(), "edge-gateway");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
