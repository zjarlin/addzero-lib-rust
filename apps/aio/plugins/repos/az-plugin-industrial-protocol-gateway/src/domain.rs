//! Domain model for gateway endpoint and route declarations.

use az_derive_aliases::{apply, plain_copy_eq, plain_copy_eq_display};

/// Industrial or integration protocol handled by the gateway.
#[apply(plain_copy_eq_display)]
pub enum Protocol {
    /// Modbus RTU over serial field buses.
    #[display("Modbus RTU")]
    ModbusRtu,
    /// Modbus TCP over industrial Ethernet.
    #[display("Modbus TCP")]
    ModbusTcp,
    /// OPC UA northbound or supervisory integration.
    #[display("OPC UA")]
    OpcUa,
    /// MQTT telemetry publishing and command subscriptions.
    #[display("MQTT")]
    Mqtt,
    /// HTTP JSON integration for business systems.
    #[display("HTTP JSON")]
    HttpJson,
}

/// Runtime role of a gateway endpoint.
#[apply(plain_copy_eq)]
pub enum EndpointRole {
    /// Data enters the gateway from this endpoint.
    Source,
    /// Data leaves the gateway through this endpoint.
    Sink,
    /// Endpoint can both receive and publish values.
    Bidirectional,
}

/// A declared protocol endpoint.
#[apply(plain_copy_eq)]
pub struct Endpoint {
    pub id: &'static str,
    pub label: &'static str,
    pub protocol: Protocol,
    pub address: &'static str,
    pub role: EndpointRole,
}

/// A conversion route between two declared endpoints.
#[apply(plain_copy_eq)]
pub struct ConversionRoute {
    pub id: &'static str,
    pub label: &'static str,
    pub source_endpoint: &'static str,
    pub target_endpoint: &'static str,
    pub mapping: &'static str,
    pub cadence_ms: u64,
}

/// Complete static gateway profile contributed by this plugin.
#[apply(plain_copy_eq)]
pub struct GatewayProfile<'a> {
    pub endpoints: &'a [Endpoint],
    pub routes: &'a [ConversionRoute],
}

impl GatewayProfile<'_> {
    /// Validates that every route references known endpoints and uses a sane
    /// polling cadence.
    pub fn validate(&self) -> Result<(), GatewayProfileError> {
        for endpoint in self.endpoints {
            if endpoint.id.trim().is_empty() {
                return Err(GatewayProfileError::EmptyEndpointId);
            }
        }

        for route in self.routes {
            if route.id.trim().is_empty() {
                return Err(GatewayProfileError::EmptyRouteId);
            }
            if route.cadence_ms < 50 {
                return Err(GatewayProfileError::CadenceTooFast {
                    route_id: route.id,
                    cadence_ms: route.cadence_ms,
                });
            }
            if !self.has_endpoint(route.source_endpoint) {
                return Err(GatewayProfileError::UnknownEndpoint {
                    route_id: route.id,
                    endpoint_id: route.source_endpoint,
                });
            }
            if !self.has_endpoint(route.target_endpoint) {
                return Err(GatewayProfileError::UnknownEndpoint {
                    route_id: route.id,
                    endpoint_id: route.target_endpoint,
                });
            }
        }
        Ok(())
    }

    fn has_endpoint(&self, endpoint_id: &str) -> bool {
        self.endpoints
            .iter()
            .any(|endpoint| endpoint.id == endpoint_id)
    }
}

/// Validation failure for a gateway profile.
#[apply(plain_copy_eq)]
pub enum GatewayProfileError {
    EmptyEndpointId,
    EmptyRouteId,
    CadenceTooFast {
        route_id: &'static str,
        cadence_ms: u64,
    },
    UnknownEndpoint {
        route_id: &'static str,
        endpoint_id: &'static str,
    },
}

static ENDPOINTS: &[Endpoint] = &[
    Endpoint {
        id: "rtu-line-a",
        label: "Line A PLC serial bus",
        protocol: Protocol::ModbusRtu,
        address: "COM3 9600-8-N-1 unit=1",
        role: EndpointRole::Source,
    },
    Endpoint {
        id: "tcp-energy",
        label: "Energy meter Modbus TCP",
        protocol: Protocol::ModbusTcp,
        address: "10.10.8.21:502 unit=2",
        role: EndpointRole::Source,
    },
    Endpoint {
        id: "opc-supervisor",
        label: "Plant supervisor OPC UA",
        protocol: Protocol::OpcUa,
        address: "opc.tcp://scada.local:4840",
        role: EndpointRole::Bidirectional,
    },
    Endpoint {
        id: "mqtt-edge",
        label: "Edge telemetry broker",
        protocol: Protocol::Mqtt,
        address: "mqtts://broker.local:8883/site/aio",
        role: EndpointRole::Sink,
    },
    Endpoint {
        id: "mes-http",
        label: "MES HTTP adapter",
        protocol: Protocol::HttpJson,
        address: "https://mes.local/api/ingest",
        role: EndpointRole::Sink,
    },
];

static ROUTES: &[ConversionRoute] = &[
    ConversionRoute {
        id: "rtu-to-mqtt-production",
        label: "PLC production counters to MQTT",
        source_endpoint: "rtu-line-a",
        target_endpoint: "mqtt-edge",
        mapping: "holding[40001..40012] -> site/aio/line-a/production",
        cadence_ms: 1_000,
    },
    ConversionRoute {
        id: "tcp-to-opc-energy",
        label: "Energy meter registers to OPC UA",
        source_endpoint: "tcp-energy",
        target_endpoint: "opc-supervisor",
        mapping: "input[30001..30016] -> ns=2;s=Energy.LineA",
        cadence_ms: 2_000,
    },
    ConversionRoute {
        id: "opc-to-mes-quality",
        label: "Quality events to MES",
        source_endpoint: "opc-supervisor",
        target_endpoint: "mes-http",
        mapping: "ns=2;s=Quality.Events -> /api/ingest/quality",
        cadence_ms: 5_000,
    },
];

/// Builds the default sample profile used by the plugin pages and tests.
pub fn default_profile() -> GatewayProfile<'static> {
    GatewayProfile {
        endpoints: ENDPOINTS,
        routes: ROUTES,
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversionRoute, GatewayProfile, GatewayProfileError, default_profile};

    #[test]
    fn default_profile_should_keep_all_routes_resolvable() {
        let profile = default_profile();

        // Every packaged route must point at endpoints present in the profile.
        assert_eq!(profile.validate(), Ok(()));
    }

    #[test]
    fn validate_should_reject_too_fast_polling_route() {
        let endpoint_profile = default_profile();
        let routes = [ConversionRoute {
            id: "unsafe-polling",
            label: "Unsafe polling",
            source_endpoint: "rtu-line-a",
            target_endpoint: "mqtt-edge",
            mapping: "holding[40001] -> site/aio/test",
            cadence_ms: 10,
        }];
        let profile = GatewayProfile {
            endpoints: endpoint_profile.endpoints,
            routes: &routes,
        };

        // Sub-50ms polling is rejected to avoid overloading serial field buses.
        assert_eq!(
            profile.validate(),
            Err(GatewayProfileError::CadenceTooFast {
                route_id: "unsafe-polling",
                cadence_ms: 10,
            }),
        );
    }
}
