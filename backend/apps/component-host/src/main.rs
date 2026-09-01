//! Gate A configurable component host and Operations Console adapter.

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use async_nats::{Client, ConnectOptions};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, header},
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::StreamExt;
use ppl_contracts::{O001_VERSION, OperationalCommand, OperationalEvent};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::{Mutex, RwLock};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use uuid::Uuid;

const CONTRACT_ID: &str = "O-001";
const INFORMATION_PROFILE: &str = "synthetic-only";
const COMMAND_PURPOSE: &str = "gate-a-component-mesh";
const READINESS_INTERVAL: Duration = Duration::from_secs(5);
const STALE_AFTER: Duration = Duration::from_secs(15);
const MAX_EVENTS: usize = 250;

#[derive(Clone, Copy)]
struct ComponentDefinition {
    id: &'static str,
    name: &'static str,
    capability: &'static str,
    command: &'static str,
}

const COMPONENTS: [ComponentDefinition; 12] = [
    ComponentDefinition {
        id: "CTL-01",
        name: "scenario-director",
        capability: "scenario lifecycle and presentation orchestration",
        command: "inspect-scenario-capability",
    },
    ComponentDefinition {
        id: "CTL-02",
        name: "presentation-gateway",
        capability: "target-owned semantic presentation",
        command: "inspect-presentation-capability",
    },
    ComponentDefinition {
        id: "IAM-01",
        name: "identity-broker",
        capability: "environment-scoped synthetic identity",
        command: "inspect-identity-capability",
    },
    ComponentDefinition {
        id: "AUT-01",
        name: "authorisation",
        capability: "protected-action decision boundary",
        command: "inspect-authorisation-capability",
    },
    ComponentDefinition {
        id: "DOM-01",
        name: "engagement",
        capability: "bounded synthetic engagement ownership",
        command: "inspect-engagement-capability",
    },
    ComponentDefinition {
        id: "CNT-01",
        name: "source-governance",
        capability: "governed source intake boundary",
        command: "inspect-source-intake-capability",
    },
    ComponentDefinition {
        id: "KNO-01",
        name: "knowledge-processing",
        capability: "bounded knowledge-processing boundary",
        command: "inspect-processing-capability",
    },
    ComponentDefinition {
        id: "WRK-01",
        name: "review-workflow",
        capability: "human review workflow boundary",
        command: "inspect-review-capability",
    },
    ComponentDefinition {
        id: "RPT-01",
        name: "reporting",
        capability: "versioned report boundary",
        command: "inspect-report-capability",
    },
    ComponentDefinition {
        id: "AUD-01",
        name: "audit-evidence",
        capability: "append-oriented evidence references",
        command: "inspect-evidence-capability",
    },
    ComponentDefinition {
        id: "OPS-01",
        name: "operations",
        capability: "component and event projection",
        command: "snapshot-components",
    },
    ComponentDefinition {
        id: "INT-01",
        name: "event-infrastructure",
        capability: "authenticated versioned event carriage",
        command: "inspect-event-infrastructure",
    },
];

const PROBE_COMPONENTS: [&str; 9] = [
    "AUT-01", "DOM-01", "CNT-01", "KNO-01", "WRK-01", "RPT-01", "AUD-01", "OPS-01", "INT-01",
];

#[derive(Clone)]
struct Config {
    component: ComponentDefinition,
    instance_id: String,
    workload_identity: String,
    environment_id: String,
    address: SocketAddr,
    source_revision: String,
    image_digest: String,
    static_directory: Option<PathBuf>,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    client: Client,
    components: Arc<RwLock<BTreeMap<String, ObservedComponent>>>,
    events: Arc<RwLock<VecDeque<OperationalEvent>>>,
    idempotency: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Clone)]
struct ObservedComponent {
    event: OperationalEvent,
    received_at: Instant,
}

#[derive(Debug)]
struct SafeError(&'static str);

impl IntoResponse for SafeError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "failed",
                "code": self.0,
                "informationProfile": INFORMATION_PROFILE
            })),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ppl_component_host=info,tower_http=info".into()),
        )
        .with_target(false)
        .compact()
        .init();
    if let Err(error) = run().await {
        error!(reason = error.0, "Gate A component host stopped safely");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), SafeError> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| SafeError("tls-crypto-provider-conflict"))?;
    let config = load_config()?;
    let client = connect(&config).await?;
    let state = AppState {
        config: config.clone(),
        client,
        components: Arc::new(RwLock::new(BTreeMap::new())),
        events: Arc::new(RwLock::new(VecDeque::new())),
        idempotency: Arc::new(Mutex::new(HashMap::new())),
    };

    if config.component.id == "OPS-01" {
        let observer = state.clone();
        tokio::spawn(async move {
            if let Err(error) = observe_events(observer).await {
                error!(reason = error.0, "operations event observer stopped");
            }
        });
    }
    let commands = state.clone();
    tokio::spawn(async move {
        if let Err(error) = consume_commands(commands).await {
            error!(reason = error.0, "component command consumer stopped");
        }
    });
    let heartbeat = state.clone();
    tokio::spawn(async move { publish_readiness(heartbeat).await });

    let router = router(state);
    let listener = tokio::net::TcpListener::bind(config.address)
        .await
        .map_err(|_| SafeError("listener-unavailable"))?;
    info!(
        component_id = config.component.id,
        instance_id = config.instance_id,
        address = %config.address,
        "Gate A component host listening"
    );
    axum::serve(listener, router)
        .await
        .map_err(|_| SafeError("listener-failed"))
}

async fn connect(config: &Config) -> Result<Client, SafeError> {
    let seed_path = required_env("PPL_NATS_NKEY_SEED_FILE")?;
    let seed = tokio::fs::read_to_string(seed_path)
        .await
        .map_err(|_| SafeError("workload-credential-unavailable"))?;
    let mut options = ConnectOptions::new()
        .name(format!("ppl-{}", config.component.name))
        .nkey(seed.trim().to_owned());
    if let Ok(root) = env::var("PPL_NATS_ROOT_CERTIFICATE") {
        options = options.add_root_certificates(root.into()).require_tls(true);
    }
    let url = required_env("PPL_NATS_URL")?;
    options
        .connect(url)
        .await
        .map_err(|_| SafeError("event-infrastructure-unavailable"))
}

fn router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/health/contracts", get(contracts))
        .route("/api/v1/component", get(component));
    if state.config.component.id == "OPS-01" {
        router = router
            .route("/api/v1/mesh", get(mesh))
            .route("/api/v1/events", get(events))
            .route("/api/v1/probe", post(probe));
        if let Some(directory) = &state.config.static_directory {
            router = router.fallback_service(
                ServeDir::new(directory)
                    .append_index_html_on_directories(true)
                    .fallback(ServeFile::new(directory.join("index.html"))),
            );
        }
    }
    router
        .with_state(state)
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'",
            ),
        ))
        .layer(TraceLayer::new_for_http())
}

async fn liveness(State(state): State<AppState>) -> Json<Value> {
    Json(component_json(&state.config, "healthy"))
}

async fn readiness(State(state): State<AppState>) -> Json<Value> {
    Json(component_json(&state.config, "ready"))
}

async fn contracts(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "componentId": state.config.component.id,
        "contracts": [{"contractId": CONTRACT_ID, "contractVersion": O001_VERSION}],
        "maturity": "in-development"
    }))
}

async fn component(State(state): State<AppState>) -> Json<Value> {
    Json(component_json(&state.config, "ready"))
}

fn component_json(config: &Config, status: &str) -> Value {
    json!({
        "componentId": config.component.id,
        "componentName": config.component.name,
        "instanceId": config.instance_id,
        "workloadIdentity": config.workload_identity,
        "status": status,
        "capability": config.component.capability,
        "acceptedCommand": config.component.command,
        "sourceRevision": config.source_revision,
        "imageDigest": config.image_digest,
        "environmentId": config.environment_id,
        "contractId": CONTRACT_ID,
        "contractVersion": O001_VERSION,
        "maturity": "in-development",
        "informationProfile": INFORMATION_PROFILE
    })
}

async fn publish_readiness(state: AppState) {
    let mut interval = tokio::time::interval(READINESS_INTERVAL);
    loop {
        interval.tick().await;
        let event = operational_event(&state.config, "component.ready", "ready", None, None);
        if let Err(error) = publish_event(&state, &event).await {
            warn!(reason = error.0, "readiness event was not published");
        }
    }
}

async fn consume_commands(state: AppState) -> Result<(), SafeError> {
    let subject = format!("ppl.gate-a.commands.{}", state.config.component.id);
    let mut subscription = state
        .client
        .subscribe(subject)
        .await
        .map_err(|_| SafeError("command-subscription-unavailable"))?;
    while let Some(message) = subscription.next().await {
        let Ok(command) = serde_json::from_slice::<OperationalCommand>(&message.payload) else {
            warn!("invalid operational command refused before processing");
            continue;
        };
        let event = evaluate_command(&state, &command, &message.payload).await;
        publish_event(&state, &event).await?;
    }
    Err(SafeError("command-subscription-closed"))
}

async fn evaluate_command(
    state: &AppState,
    command: &OperationalCommand,
    payload: &[u8],
) -> OperationalEvent {
    let reason = if command.contract_id != CONTRACT_ID || command.contract_version != O001_VERSION {
        Some("contract-version-unsupported")
    } else if command.environment_id != state.config.environment_id {
        Some("environment-mismatch")
    } else if command.target_component != state.config.component.id {
        Some("target-mismatch")
    } else if command.issuer_component != "OPS-01" || command.purpose != COMMAND_PURPOSE {
        Some("authority-or-purpose-refused")
    } else if command.command_name != state.config.component.command {
        Some("command-unsupported")
    } else if command.idempotency_key.is_empty() {
        Some("idempotency-key-required")
    } else {
        None
    };
    if let Some(reason) = reason {
        return operational_event(
            &state.config,
            "component.command-refused",
            "refused",
            Some(command),
            Some(reason),
        );
    }

    let payload_fingerprint = format!("{:x}", Sha256::digest(payload));
    let mut idempotency = state.idempotency.lock().await;
    if let Some(previous) = idempotency.get(&command.idempotency_key) {
        let (event_type, status, reason) = if previous == &payload_fingerprint {
            (
                "component.command-duplicate",
                "accepted",
                "exact-redelivery",
            )
        } else {
            (
                "component.command-refused",
                "refused",
                "idempotency-content-conflict",
            )
        };
        return operational_event(
            &state.config,
            event_type,
            status,
            Some(command),
            Some(reason),
        );
    }
    idempotency.insert(command.idempotency_key.clone(), payload_fingerprint);
    operational_event(
        &state.config,
        "component.command-accepted",
        "accepted",
        Some(command),
        None,
    )
}

fn operational_event(
    config: &Config,
    event_type: &str,
    status: &str,
    command: Option<&OperationalCommand>,
    reason_code: Option<&str>,
) -> OperationalEvent {
    OperationalEvent {
        contract_id: CONTRACT_ID.to_owned(),
        contract_version: O001_VERSION.to_owned(),
        event_id: format!("event:{}", Uuid::new_v4()),
        event_type: event_type.to_owned(),
        component_id: config.component.id.to_owned(),
        component_name: config.component.name.to_owned(),
        instance_id: config.instance_id.clone(),
        workload_identity: config.workload_identity.clone(),
        environment_id: config.environment_id.clone(),
        status: status.to_owned(),
        capability: config.component.capability.to_owned(),
        source_revision: config.source_revision.clone(),
        image_digest: config.image_digest.clone(),
        occurred_at: now(),
        information_profile: INFORMATION_PROFILE.to_owned(),
        command_name: command.map(|value| value.command_name.clone()),
        correlation_id: command.map(|value| value.correlation_id.clone()),
        causation_id: command.map(|value| value.command_id.clone()),
        idempotency_key: command.map(|value| value.idempotency_key.clone()),
        reason_code: reason_code.map(str::to_owned),
    }
}

async fn publish_event(state: &AppState, event: &OperationalEvent) -> Result<(), SafeError> {
    let subject = format!("ppl.gate-a.events.{}", state.config.component.id);
    let bytes = serde_json::to_vec(event).map_err(|_| SafeError("event-payload-invalid"))?;
    state
        .client
        .publish(subject, bytes.into())
        .await
        .map_err(|_| SafeError("event-publish-failed"))?;
    state
        .client
        .flush()
        .await
        .map_err(|_| SafeError("event-publish-failed"))
}

async fn observe_events(state: AppState) -> Result<(), SafeError> {
    let mut subscription = state
        .client
        .subscribe("ppl.gate-a.events.*")
        .await
        .map_err(|_| SafeError("event-observer-unavailable"))?;
    while let Some(message) = subscription.next().await {
        let event: OperationalEvent =
            match serde_json::from_slice::<OperationalEvent>(&message.payload) {
                Ok(event) if event.environment_id == state.config.environment_id => event,
                _ => continue,
            };
        if event.event_type == "component.ready" {
            state.components.write().await.insert(
                event.component_id.clone(),
                ObservedComponent {
                    event: event.clone(),
                    received_at: Instant::now(),
                },
            );
        }
        let mut events = state.events.write().await;
        events.push_front(event);
        events.truncate(MAX_EVENTS);
    }
    Err(SafeError("event-observer-closed"))
}

async fn mesh(State(state): State<AppState>) -> Json<Value> {
    let observed = state.components.read().await;
    let components = COMPONENTS
        .iter()
        .map(|expected| {
            if let Some(actual) = observed.get(expected.id) {
                let age = actual.received_at.elapsed();
                json!({
                    "componentId": expected.id,
                    "componentName": expected.name,
                    "status": if age <= STALE_AFTER { "ready" } else { "stale" },
                    "instanceId": actual.event.instance_id,
                    "workloadIdentity": actual.event.workload_identity,
                    "capability": actual.event.capability,
                    "sourceRevision": actual.event.source_revision,
                    "imageDigest": actual.event.image_digest,
                    "lastActivity": actual.event.occurred_at,
                    "ageSeconds": age.as_secs(),
                })
            } else {
                json!({
                    "componentId": expected.id,
                    "componentName": expected.name,
                    "status": "missing",
                    "capability": expected.capability,
                })
            }
        })
        .collect::<Vec<_>>();
    let ready = components
        .iter()
        .filter(|value| value["status"] == "ready")
        .count();
    Json(json!({
        "environmentId": state.config.environment_id,
        "contractId": CONTRACT_ID,
        "contractVersion": O001_VERSION,
        "expected": COMPONENTS.len(),
        "ready": ready,
        "status": if ready == COMPONENTS.len() { "ready" } else { "degraded" },
        "components": components,
        "observedAt": now(),
        "maturity": "in-development",
        "informationProfile": INFORMATION_PROFILE
    }))
}

async fn events(State(state): State<AppState>) -> Json<Value> {
    let events = state.events.read().await;
    Json(json!({
        "environmentId": state.config.environment_id,
        "events": events.iter().cloned().collect::<Vec<_>>(),
        "observedAt": now(),
        "informationProfile": INFORMATION_PROFILE
    }))
}

async fn probe(State(state): State<AppState>) -> Result<Json<Value>, SafeError> {
    let correlation_id = format!("correlation:{}", Uuid::new_v4());
    let issued_at = now();
    for component_id in PROBE_COMPONENTS {
        let definition =
            definition(component_id).ok_or(SafeError("component-definition-missing"))?;
        let command = OperationalCommand {
            contract_id: CONTRACT_ID.to_owned(),
            contract_version: O001_VERSION.to_owned(),
            command_id: format!("command:{}", Uuid::new_v4()),
            command_name: definition.command.to_owned(),
            target_component: definition.id.to_owned(),
            issuer_component: "OPS-01".to_owned(),
            environment_id: state.config.environment_id.clone(),
            purpose: COMMAND_PURPOSE.to_owned(),
            correlation_id: correlation_id.clone(),
            causation_id: format!("operation:{}", Uuid::new_v4()),
            idempotency_key: format!("probe:{correlation_id}:{}", definition.id),
            issued_at: issued_at.clone(),
        };
        let bytes =
            serde_json::to_vec(&command).map_err(|_| SafeError("command-payload-invalid"))?;
        state
            .client
            .publish(
                format!("ppl.gate-a.commands.{}", definition.id),
                bytes.into(),
            )
            .await
            .map_err(|_| SafeError("command-publish-failed"))?;
    }
    state
        .client
        .flush()
        .await
        .map_err(|_| SafeError("command-publish-failed"))?;
    Ok(Json(json!({
        "status": "issued",
        "correlationId": correlation_id,
        "targetCount": PROBE_COMPONENTS.len(),
        "issuedAt": issued_at,
        "purpose": COMMAND_PURPOSE
    })))
}

fn load_config() -> Result<Config, SafeError> {
    let component_id = required_env("PPL_COMPONENT_ID")?;
    let component = definition(&component_id).ok_or(SafeError("component-id-invalid"))?;
    if matches!(component.id, "CTL-01" | "CTL-02" | "IAM-01") {
        return Err(SafeError("component-owned-by-m3-runtime"));
    }
    let instance_id = required_env("PPL_INSTANCE_ID")?;
    let workload_identity = load_text("PPL_WORKLOAD_IDENTITY_FILE")?;
    let environment_id = if let Ok(value) = env::var("PPL_ENVIRONMENT_ID") {
        value
    } else {
        load_text("PPL_ENVIRONMENT_ID_FILE")?
    };
    if instance_id.is_empty() || workload_identity.is_empty() || environment_id.len() < 8 {
        return Err(SafeError("component-identity-invalid"));
    }
    Ok(Config {
        component,
        instance_id,
        workload_identity,
        environment_id,
        address: env::var("PPL_LISTEN_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
            .parse()
            .map_err(|_| SafeError("listen-address-invalid"))?,
        source_revision: env::var("PPL_SOURCE_REVISION")
            .unwrap_or_else(|_| "working-tree".to_owned()),
        image_digest: env::var("PPL_IMAGE_DIGEST")
            .unwrap_or_else(|_| "unresolved-build-digest".to_owned()),
        static_directory: env::var("PPL_STATIC_DIRECTORY").ok().map(PathBuf::from),
    })
}

fn definition(id: &str) -> Option<ComponentDefinition> {
    COMPONENTS.iter().copied().find(|value| value.id == id)
}

fn required_env(name: &'static str) -> Result<String, SafeError> {
    env::var(name).map_err(|_| SafeError("required-configuration-missing"))
}

fn load_text(variable: &'static str) -> Result<String, SafeError> {
    let path = required_env(variable)?;
    let value = std::fs::read_to_string(path)
        .map_err(|_| SafeError("protected-configuration-unavailable"))?;
    let value = value.trim().to_owned();
    if value.is_empty() {
        Err(SafeError("protected-configuration-invalid"))
    } else {
        Ok(value)
    }
}

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "time-unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{COMPONENTS, PROBE_COMPONENTS, definition};

    #[test]
    fn component_ids_and_names_are_unique() {
        for (index, component) in COMPONENTS.iter().enumerate() {
            assert!(
                COMPONENTS[index + 1..]
                    .iter()
                    .all(|other| other.id != component.id && other.name != component.name)
            );
            assert!(!component.command.is_empty());
        }
    }

    #[test]
    fn probe_targets_only_configurable_component_hosts() {
        for target in PROBE_COMPONENTS {
            let component = definition(target).expect("probe target must be defined");
            assert!(!matches!(component.id, "CTL-01" | "CTL-02" | "IAM-01"));
        }
    }
}
