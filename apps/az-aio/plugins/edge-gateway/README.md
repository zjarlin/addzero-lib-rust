# Edge Gateway

Native plugin for gateway flow execution.

## Runtime

- Dioxus UI contract page: `edge-gateway.page`
- Route: `/gateway`
- Axum APIs: `/api/edge-gateway/status`, `/api/edge-gateway/example`, `/api/edge-gateway/run`, `/api/edge-gateway/assets`, `/api/edge-gateway/assets/weather/current`, `/api/edge-gateway/assets/usage`, `/api/edge-gateway/flows`, `/api/edge-gateway/flow`
- Toasty table prefix: `biz_edge_gateway_`
- Rudi context: `store::build_edge_gateway_context`

## Callable Assets

The first callable asset is a token-gated current-weather API backed by Open-Meteo.

```bash
curl -X POST "$AZ_AIO_BASE_URL/api/edge-gateway/assets/weather/current" \
  -H 'Authorization: Bearer edge-demo-weather-token' \
  -H 'Content-Type: application/json' \
  -d '{"location":"Shanghai","timezone":"Asia/Shanghai"}'
```

For coordinate-based calls, send `latitude` and `longitude` together:

```json
{"latitude":31.2304,"longitude":121.4737,"timezone":"Asia/Shanghai"}
```

Asset metadata is available from `GET /api/edge-gateway/assets`; usage records are persisted in Toasty PostgreSQL when `DATABASE_URL` is configured and fall back to the degraded in-memory store when PG is unavailable.

## Domain

Gateway runtime request rendering, response capture, and execution stay in `gateway_runtime*.rs` domain files.
