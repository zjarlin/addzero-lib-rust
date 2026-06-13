# Edge Gateway

Native AZ AIO plugin for gateway flow execution.

## Runtime

- Dioxus renderer: `edge-gateway.page`
- Route: `/gateway`
- Axum APIs: `/api/edge-gateway/status`, `/api/edge-gateway/example`, `/api/edge-gateway/run`, `/api/edge-gateway/flows`, `/api/edge-gateway/flow`
- Toasty table prefix: `biz_edge_gateway_`
- shaku module: `store::EdgeGatewayModule`

## Domain

Gateway runtime request rendering, response capture, and execution stay in `gateway_runtime*.rs` domain files.
