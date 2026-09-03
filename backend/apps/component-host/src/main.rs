//! Gate A configurable component host and Operations Console adapter.

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use async_nats::{
    Client, ConnectOptions,
    jetstream::{
        self,
        consumer::{AckPolicy, pull},
    },
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, header},
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::StreamExt;
use ppl_aut_01::{PolicyAdapter, PolicyConfig};
use ppl_cnt_01::{SourceLifecycleEvent, SourceStore};
use ppl_contracts::{
    A001_VERSION, A002_VERSION, AZ001_VERSION, AssertionStatus, AssertionType,
    AuthorisationDecision, AuthorisationDecisionRequest, AuthorisationDecisionStatus,
    AuthorisationObligation, AuthoritativeAssertion, K001_VERSION, O001_VERSION,
    OperationalCommand, OperationalEvent, PrincipalReference, PrincipalType,
    ProcessingLifecycleQuery, ProcessingLifecycleState, SourceIntakeCommand, SourceIntakeQuery,
    SourceIntakeStatus, SourceLifecycleQuery, SourceStageCommand, SourceValidationStatus,
    StagedSourceContent, StagedSourceContentQuery,
};
use ppl_kno_01::{ProcessingLifecycleEvent, ProcessingStore, inspect_content};
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
const SOURCE_AUTHORISATION_SUBJECT: &str = "ppl.gate-c.decisions.AUT-01";
const SOURCE_LIFECYCLE_EVENT_SUBJECT: &str = "ppl.gate-c.events.CNT-01";
const PROCESSING_LIFECYCLE_EVENT_SUBJECT: &str = "ppl.gate-c.events.KNO-01";
const SOURCE_PROCESSING_INPUT_SUBJECT: &str = "ppl.gate-c.processing-input.CNT-01";
const PROCESSING_QUERY_SUBJECT: &str = "ppl.gate-c.queries.KNO-01";
const SOURCE_STREAM_NAME: &str = "PPL_GATE_C_SOURCE";

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
        capability: "bounded staged-source processing lifecycle",
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
    source_state_path: Option<PathBuf>,
    processing_state_path: Option<PathBuf>,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    client: Client,
    components: Arc<RwLock<BTreeMap<String, ObservedComponent>>>,
    events: Arc<RwLock<VecDeque<OperationalEvent>>>,
    idempotency: Arc<Mutex<HashMap<String, String>>>,
    source_store: Option<SourceStore>,
    processing_store: Option<ProcessingStore>,
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
    let source_store = match &config.source_state_path {
        Some(path) => {
            Some(SourceStore::open(path).map_err(|_| SafeError("source-store-unavailable"))?)
        }
        None => None,
    };
    let processing_store = match &config.processing_state_path {
        Some(path) => Some(
            ProcessingStore::open(path).map_err(|_| SafeError("processing-store-unavailable"))?,
        ),
        None => None,
    };
    let state = AppState {
        config: config.clone(),
        client,
        components: Arc::new(RwLock::new(BTreeMap::new())),
        events: Arc::new(RwLock::new(VecDeque::new())),
        idempotency: Arc::new(Mutex::new(HashMap::new())),
        source_store,
        processing_store,
    };

    if config.component.id == "OPS-01" {
        let observer = state.clone();
        tokio::spawn(async move {
            if let Err(error) = observe_events(observer).await {
                error!(reason = error.0, "operations event observer stopped");
            }
        });
    }
    if config.component.id == "CNT-01" {
        let source_intake = state.clone();
        tokio::spawn(async move {
            if let Err(error) = consume_source_intake(source_intake).await {
                error!(reason = error.0, "source-intake consumer stopped");
            }
        });
    }
    if config.component.id == "AUT-01" {
        let source_authorisation = state.clone();
        tokio::spawn(async move {
            if let Err(error) = consume_source_authorisation(source_authorisation).await {
                error!(reason = error.0, "source authorisation consumer stopped");
            }
        });
    }
    if config.component.id == "KNO-01" {
        let staged = state.clone();
        tokio::spawn(async move { consume_staged_sources(staged).await });
        let queries = state.clone();
        tokio::spawn(async move {
            if let Err(error) = consume_processing_queries(queries).await {
                error!(reason = error.0, "processing query consumer stopped");
            }
        });
        let processor = state.clone();
        tokio::spawn(async move { reconcile_processing(processor).await });
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
    let mut contracts = vec![json!({
        "contractId": CONTRACT_ID,
        "contractVersion": O001_VERSION
    })];
    if state.config.component.id == "CNT-01" {
        contracts.push(json!({
            "contractId": "A-001",
            "contractVersion": A001_VERSION,
            "status": "implemented"
        }));
        contracts.push(json!({
            "contractId": "A-002",
            "contractVersion": A002_VERSION,
            "status": "working-draft"
        }));
    }
    if state.config.component.id == "AUT-01" {
        contracts.push(json!({
            "contractId": "AZ-001",
            "contractVersion": AZ001_VERSION,
            "status": "agreed"
        }));
    }
    if state.config.component.id == "KNO-01" {
        contracts.push(json!({
            "contractId": "K-001",
            "contractVersion": K001_VERSION,
            "status": "implemented"
        }));
    }
    Json(json!({
        "componentId": state.config.component.id,
        "contracts": contracts,
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
        subject_reference: None,
    }
}

async fn consume_source_intake(state: AppState) -> Result<(), SafeError> {
    let store = state
        .source_store
        .clone()
        .ok_or(SafeError("source-store-unavailable"))?;
    let mut commands = state
        .client
        .subscribe("ppl.gate-c.commands.CNT-01")
        .await
        .map_err(|_| SafeError("source-command-subscription-unavailable"))?;
    let mut queries = state
        .client
        .subscribe("ppl.gate-c.queries.CNT-01")
        .await
        .map_err(|_| SafeError("source-query-subscription-unavailable"))?;
    let mut processing_inputs = state
        .client
        .subscribe(SOURCE_PROCESSING_INPUT_SUBJECT)
        .await
        .map_err(|_| SafeError("processing-input-subscription-unavailable"))?;
    let mut retry = tokio::time::interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            Some(message) = commands.next() => {
                let Some(reply) = message.reply else {
                    warn!("source-intake command without reply subject was refused");
                    continue;
                };
                let response = handle_source_command(&state, &store, &message.payload).await;
                publish_source_outbox(&state, &store).await;
                respond(&state, reply, response).await?;
            }
            Some(message) = queries.next() => {
                let Some(reply) = message.reply else {
                    warn!("source-intake query without reply subject was refused");
                    continue;
                };
                let response = handle_source_query(&state, &store, &message.payload);
                respond(&state, reply, response).await?;
            }
            Some(message) = processing_inputs.next() => {
                let Some(reply) = message.reply else {
                    warn!("processing input query without reply subject was refused");
                    continue;
                };
                let response = handle_processing_input_query(&state, &store, &message.payload);
                respond(&state, reply, response).await?;
            }
            _ = retry.tick() => publish_source_outbox(&state, &store).await,
            else => return Err(SafeError("source-intake-subscription-closed")),
        }
    }
}

fn handle_processing_input_query(
    state: &AppState,
    store: &SourceStore,
    payload: &[u8],
) -> Result<Vec<u8>, ppl_cnt_01::SourceStoreError> {
    let query: StagedSourceContentQuery = serde_json::from_slice(payload)
        .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
    let response = store.processing_input(&query, &state.config.environment_id, &now())?;
    serde_json::to_vec(&response).map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)
}

async fn handle_source_command(
    state: &AppState,
    store: &SourceStore,
    payload: &[u8],
) -> Result<Vec<u8>, ppl_cnt_01::SourceStoreError> {
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
    match envelope.get("contractId").and_then(Value::as_str) {
        Some("A-001") => {
            let command: SourceIntakeCommand = serde_json::from_value(envelope)
                .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
            let outcome = store.apply(&command, &state.config.environment_id, &now())?;
            if outcome.status == SourceIntakeStatus::Quarantined {
                let source_version_id = outcome
                    .source_version
                    .as_ref()
                    .ok_or(ppl_cnt_01::SourceStoreError::OutcomeInvalid)?
                    .source_version_id
                    .clone();
                store.validate_source(&source_version_id, &state.config.environment_id, &now())?;
            }
            serde_json::to_vec(&outcome).map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)
        }
        Some("A-002") => {
            let command: SourceStageCommand = serde_json::from_value(envelope)
                .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
            let lifecycle =
                store.lifecycle(&command.source_version_id, &state.config.environment_id)?;
            let decision = if lifecycle.validation.status == SourceValidationStatus::Validated {
                request_source_authorisation(state, &command).await
            } else {
                let request = source_authorisation_request(&command, OffsetDateTime::now_utc());
                safe_authorisation_decision(
                    &request,
                    AuthorisationDecisionStatus::NotApplicable,
                    "source-validation-refused",
                    OffsetDateTime::now_utc(),
                )
            };
            let outcome = store.stage(&command, &decision, &state.config.environment_id, &now())?;
            serde_json::to_vec(&outcome).map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)
        }
        _ => Err(ppl_cnt_01::SourceStoreError::OutcomeInvalid),
    }
}

fn handle_source_query(
    state: &AppState,
    store: &SourceStore,
    payload: &[u8],
) -> Result<Vec<u8>, ppl_cnt_01::SourceStoreError> {
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
    match envelope.get("contractId").and_then(Value::as_str) {
        Some("A-001") => {
            let query: SourceIntakeQuery = serde_json::from_value(envelope)
                .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
            if query.contract_version != A001_VERSION
                || query.message_type != "source-intake.query"
                || query.environment_id != state.config.environment_id
            {
                return Err(ppl_cnt_01::SourceStoreError::OutcomeInvalid);
            }
            let outcome = store.outcome(&query.command_id, &state.config.environment_id)?;
            serde_json::to_vec(&outcome).map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)
        }
        Some("A-002") => {
            let query: SourceLifecycleQuery = serde_json::from_value(envelope)
                .map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)?;
            if query.contract_version != A002_VERSION
                || query.message_type != "source-lifecycle.query"
                || query.environment_id != state.config.environment_id
            {
                return Err(ppl_cnt_01::SourceStoreError::OutcomeInvalid);
            }
            let status = store.lifecycle(&query.source_version_id, &state.config.environment_id)?;
            serde_json::to_vec(&status).map_err(|_| ppl_cnt_01::SourceStoreError::OutcomeInvalid)
        }
        _ => Err(ppl_cnt_01::SourceStoreError::OutcomeInvalid),
    }
}

async fn request_source_authorisation(
    state: &AppState,
    command: &SourceStageCommand,
) -> AuthorisationDecision {
    let evaluated_at = OffsetDateTime::now_utc();
    let request = source_authorisation_request(command, evaluated_at);
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        state.client.request(
            SOURCE_AUTHORISATION_SUBJECT,
            serde_json::to_vec(&request).unwrap_or_default().into(),
        ),
    )
    .await;
    if let Ok(Ok(message)) = response
        && let Ok(decision) = serde_json::from_slice::<AuthorisationDecision>(&message.payload)
    {
        return decision;
    }
    safe_authorisation_decision(
        &request,
        AuthorisationDecisionStatus::Indeterminate,
        "dependency-unavailable",
        evaluated_at,
    )
}

fn source_authorisation_request(
    command: &SourceStageCommand,
    evaluated_at: OffsetDateTime,
) -> AuthorisationDecisionRequest {
    let principal = |principal_type, principal_id: &str, issuer: &str| PrincipalReference {
        principal_type,
        principal_id: principal_id.to_owned(),
        environment_id: command.environment_id.clone(),
        issuer: issuer.to_owned(),
        trust_domain: Some(format!("trust-domain:{}", command.environment_id)),
    };
    let assertion = |assertion_type, source_id: &str| AuthoritativeAssertion {
        source_id: source_id.to_owned(),
        assertion_type,
        subject_id: command.actor_id.clone(),
        resource_id: command.source_version_id.clone(),
        purpose_codes: vec![command.purpose.clone()],
        status: AssertionStatus::Active,
        effective_at: format_time(evaluated_at - time::Duration::minutes(1)),
        expires_at: format_time(evaluated_at + time::Duration::minutes(5)),
        version: "1.0.0".to_owned(),
    };
    AuthorisationDecisionRequest {
        contract_id: "AZ-001".to_owned(),
        contract_version: AZ001_VERSION.to_owned(),
        kind: "decision-request".to_owned(),
        request_id: format!("authorisation-request:{}", command.command_id),
        environment_id: command.environment_id.clone(),
        requester: principal(
            PrincipalType::Workload,
            "workload-source-governance",
            "environment-workload-issuer",
        ),
        actor: principal(
            PrincipalType::SyntheticHuman,
            &command.actor_id,
            &command.authority_reference,
        ),
        action: command.action.clone(),
        resource: command.source_version_id.clone(),
        purpose: command.purpose.clone(),
        requested_roles: vec![command.actor_role.clone()],
        assertions: vec![
            assertion(AssertionType::Relationship, "scenario-session-binding"),
            assertion(AssertionType::Consent, "source-owner-staging-authority"),
        ],
        policy_version: "1.0.0".to_owned(),
        requested_at: command.requested_at.clone(),
    }
}

async fn consume_source_authorisation(state: AppState) -> Result<(), SafeError> {
    let mut requests = state
        .client
        .subscribe(SOURCE_AUTHORISATION_SUBJECT)
        .await
        .map_err(|_| SafeError("source-authorisation-subscription-unavailable"))?;
    while let Some(message) = requests.next().await {
        let Some(reply) = message.reply else {
            warn!("source authorisation request without reply subject was refused");
            continue;
        };
        let evaluated_at = OffsetDateTime::now_utc();
        let decision =
            match serde_json::from_slice::<AuthorisationDecisionRequest>(&message.payload) {
                Ok(request) => evaluate_source_authorisation(
                    &state.config.environment_id,
                    &request,
                    evaluated_at,
                ),
                Err(_) => continue,
            };
        state
            .client
            .publish(
                reply,
                serde_json::to_vec(&decision)
                    .map_err(|_| SafeError("source-authorisation-response-invalid"))?
                    .into(),
            )
            .await
            .map_err(|_| SafeError("source-authorisation-response-failed"))?;
    }
    Err(SafeError("source-authorisation-subscription-closed"))
}

fn evaluate_source_authorisation(
    environment_id: &str,
    request: &AuthorisationDecisionRequest,
    evaluated_at: OffsetDateTime,
) -> AuthorisationDecision {
    if request.requester.principal_id != "workload-source-governance"
        || request.actor.principal_id != "synthetic-reviewer"
        || request.action != "release-to-staging"
        || request.purpose != "governed-source-staging"
        || request.requested_roles != ["workbench-reviewer"]
        || !request.resource.starts_with("source-version:")
    {
        return safe_authorisation_decision(
            request,
            AuthorisationDecisionStatus::Deny,
            "source-staging-policy-refused",
            evaluated_at,
        );
    }
    PolicyAdapter::new(PolicyConfig {
        environment_id: environment_id.to_owned(),
        policy_version: "1.0.0".to_owned(),
        allowed_action: "release-to-staging".to_owned(),
        allowed_resources: vec![request.resource.clone()],
        relationship_source: "scenario-session-binding".to_owned(),
        consent_source: "source-owner-staging-authority".to_owned(),
        obligations: vec![AuthorisationObligation {
            code: "retain-staging-evidence".to_owned(),
            value: None,
        }],
        dependency_available: true,
    })
    .evaluate(request, evaluated_at)
}

fn safe_authorisation_decision(
    request: &AuthorisationDecisionRequest,
    status: AuthorisationDecisionStatus,
    reason_code: &str,
    decided_at: OffsetDateTime,
) -> AuthorisationDecision {
    AuthorisationDecision {
        contract_id: "AZ-001".to_owned(),
        contract_version: AZ001_VERSION.to_owned(),
        kind: "decision".to_owned(),
        decision_id: format!("decision:refused:{}", Uuid::new_v4()),
        request_id: request.request_id.clone(),
        status,
        reason_code: reason_code.to_owned(),
        obligations: Vec::new(),
        policy_version: "1.0.0".to_owned(),
        decided_at: format_time(decided_at),
        valid_until: None,
        evidence_references: Vec::new(),
    }
}

async fn respond(
    state: &AppState,
    reply: async_nats::Subject,
    response: Result<Vec<u8>, ppl_cnt_01::SourceStoreError>,
) -> Result<(), SafeError> {
    let bytes = response.unwrap_or_else(|error| {
        serde_json::to_vec(&json!({
            "status": "refused",
            "code": error.to_string(),
            "informationProfile": INFORMATION_PROFILE,
        }))
        .unwrap_or_default()
    });
    state
        .client
        .publish(reply, bytes.into())
        .await
        .map_err(|_| SafeError("source-response-publish-failed"))
}

async fn publish_source_outbox(state: &AppState, store: &SourceStore) {
    let Ok(events) = store.pending_events() else {
        warn!("source event outbox could not be read");
        return;
    };
    for event in events {
        let operational = source_operational_event(&state.config, &event);
        let durable = async_nats::jetstream::new(state.client.clone())
            .publish(
                SOURCE_LIFECYCLE_EVENT_SUBJECT,
                match serde_json::to_vec(&operational) {
                    Ok(bytes) => bytes.into(),
                    Err(_) => break,
                },
            )
            .await;
        let durable = match durable {
            Ok(acknowledgement) => acknowledgement.await.is_ok(),
            Err(_) => false,
        };
        if durable && publish_event(state, &operational).await.is_ok() {
            if store.mark_published(&event.event_id, &now()).is_err() {
                warn!(
                    event_id = event.event_id,
                    "source event publication could not be concluded"
                );
            }
        } else {
            warn!(
                event_id = event.event_id,
                "source event remains pending durable publication"
            );
            break;
        }
    }
}

async fn consume_staged_sources(state: AppState) {
    loop {
        if let Err(error) = consume_staged_stream_once(&state).await {
            warn!(reason = error.0, "staged-source consumer will retry");
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn consume_staged_stream_once(state: &AppState) -> Result<(), SafeError> {
    let store = state
        .processing_store
        .clone()
        .ok_or(SafeError("processing-store-unavailable"))?;
    let context = jetstream::new(state.client.clone());
    let stream = context
        .get_stream(SOURCE_STREAM_NAME)
        .await
        .map_err(|_| SafeError("source-stream-unavailable"))?;
    let consumer = stream
        .get_or_create_consumer(
            "knowledge-processing",
            pull::Config {
                durable_name: Some("knowledge-processing".to_owned()),
                description: Some("KNO-01 durable staged-source consumer for Gate C".to_owned()),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(15),
                max_deliver: 10,
                filter_subject: SOURCE_LIFECYCLE_EVENT_SUBJECT.to_owned(),
                max_ack_pending: 32,
                ..Default::default()
            },
        )
        .await
        .map_err(|_| SafeError("processing-consumer-unavailable"))?;
    let mut messages = consumer
        .messages()
        .await
        .map_err(|_| SafeError("processing-message-stream-unavailable"))?;
    while let Some(message) = messages.next().await {
        let message = message.map_err(|_| SafeError("processing-message-delivery-failed"))?;
        let Ok(event) = serde_json::from_slice::<OperationalEvent>(&message.payload) else {
            warn!("malformed source lifecycle fact was refused");
            if let Err(error) = message.ack().await {
                warn!(reason = %error, "malformed source fact acknowledgement failed");
            }
            continue;
        };
        if event.event_type != "source.staged" {
            if let Err(error) = message.ack().await {
                warn!(reason = %error, "non-staged source fact acknowledgement failed");
            }
            continue;
        }
        match store.accept(&event, &state.config.environment_id, &now()) {
            Ok(_) => {
                publish_processing_outbox(state, &store).await;
                if let Err(error) = message.ack().await {
                    warn!(reason = %error, "staged-source acknowledgement failed");
                }
            }
            Err(ppl_kno_01::ProcessingStoreError::EventRefused) => {
                warn!("staged-source fact failed the KNO-01 boundary checks");
                if let Err(error) = message.ack().await {
                    warn!(reason = %error, "refused staged-source acknowledgement failed");
                }
            }
            Err(_) => warn!("staged-source fact remains unacknowledged for reconciliation"),
        }
    }
    Err(SafeError("processing-message-stream-closed"))
}

async fn consume_processing_queries(state: AppState) -> Result<(), SafeError> {
    let store = state
        .processing_store
        .clone()
        .ok_or(SafeError("processing-store-unavailable"))?;
    let mut queries = state
        .client
        .subscribe(PROCESSING_QUERY_SUBJECT)
        .await
        .map_err(|_| SafeError("processing-query-subscription-unavailable"))?;
    while let Some(message) = queries.next().await {
        let Some(reply) = message.reply else {
            warn!("processing query without reply subject was refused");
            continue;
        };
        let response = handle_processing_query(&state, &store, &message.payload);
        respond_processing(&state, reply, response).await?;
    }
    Err(SafeError("processing-query-subscription-closed"))
}

fn handle_processing_query(
    state: &AppState,
    store: &ProcessingStore,
    payload: &[u8],
) -> Result<Vec<u8>, ppl_kno_01::ProcessingStoreError> {
    let query: ProcessingLifecycleQuery =
        serde_json::from_slice(payload).map_err(|_| ppl_kno_01::ProcessingStoreError::Invalid)?;
    if query.contract_id != "K-001"
        || query.contract_version != K001_VERSION
        || query.message_type != "processing-lifecycle.query"
        || query.environment_id != state.config.environment_id
        || query.source_version_id.is_some() == query.demonstration_session_id.is_some()
    {
        return Err(ppl_kno_01::ProcessingStoreError::Invalid);
    }
    let status = if let Some(source_version_id) = query.source_version_id {
        store.status(&source_version_id, &state.config.environment_id)?
    } else if let Some(session_id) = query.demonstration_session_id {
        store.latest_for_session(&session_id, &state.config.environment_id)?
    } else {
        return Err(ppl_kno_01::ProcessingStoreError::Invalid);
    };
    serde_json::to_vec(&status).map_err(|_| ppl_kno_01::ProcessingStoreError::Invalid)
}

async fn respond_processing(
    state: &AppState,
    reply: async_nats::Subject,
    response: Result<Vec<u8>, ppl_kno_01::ProcessingStoreError>,
) -> Result<(), SafeError> {
    let bytes = response.unwrap_or_else(|error| {
        serde_json::to_vec(&json!({
            "status": "refused",
            "code": error.to_string(),
            "informationProfile": INFORMATION_PROFILE,
        }))
        .unwrap_or_default()
    });
    state
        .client
        .publish(reply, bytes.into())
        .await
        .map_err(|_| SafeError("processing-response-publish-failed"))
}

async fn reconcile_processing(state: AppState) {
    let Some(store) = state.processing_store.clone() else {
        return;
    };
    let mut retry = tokio::time::interval(Duration::from_millis(250));
    loop {
        retry.tick().await;
        publish_processing_outbox(&state, &store).await;
        let Ok(source_versions) = store.reconcilable() else {
            warn!("processing reconciliation state could not be read");
            continue;
        };
        for source_version_id in source_versions {
            process_source_version(&state, &store, &source_version_id).await;
        }
    }
}

async fn process_source_version(
    state: &AppState,
    store: &ProcessingStore,
    source_version_id: &str,
) {
    let Ok(previous) = store.status(source_version_id, &state.config.environment_id) else {
        return;
    };
    let Ok(started) = store.start(source_version_id, &state.config.environment_id, &now()) else {
        return;
    };
    publish_processing_outbox(state, store).await;
    if previous.lifecycle_status == ProcessingLifecycleState::Accepted {
        tokio::time::sleep(Duration::from_millis(350)).await;
    }
    let query = StagedSourceContentQuery {
        contract_id: "K-001".to_owned(),
        contract_version: K001_VERSION.to_owned(),
        message_type: "staged-source-content.query".to_owned(),
        query_id: format!("processing-input-query:{}", Uuid::new_v4()),
        environment_id: state.config.environment_id.clone(),
        demonstration_session_id: started.demonstration_session_id.clone(),
        source_version_id: source_version_id.to_owned(),
        requester_component: "KNO-01".to_owned(),
        purpose: "bounded-source-processing".to_owned(),
        correlation_id: started.correlation_id.clone(),
        causation_id: started.processing_id.clone(),
        requested_at: now(),
    };
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        state.client.request(
            SOURCE_PROCESSING_INPUT_SUBJECT,
            match serde_json::to_vec(&query) {
                Ok(bytes) => bytes.into(),
                Err(_) => return,
            },
        ),
    )
    .await;
    let Ok(Ok(message)) = response else {
        return;
    };
    let input = match serde_json::from_slice::<StagedSourceContent>(&message.payload) {
        Ok(input)
            if input.contract_id == "K-001"
                && input.contract_version == K001_VERSION
                && input.environment_id == state.config.environment_id
                && input.demonstration_session_id == started.demonstration_session_id
                && input.source_version_id == source_version_id
                && input.released_to_component == "KNO-01"
                && input.purpose == "bounded-source-processing" =>
        {
            input
        }
        _ => {
            let _ = store.fail(
                source_version_id,
                &state.config.environment_id,
                "source-input-refused",
                &now(),
            );
            publish_processing_outbox(state, store).await;
            return;
        }
    };
    match inspect_content(&input) {
        Ok(result) => {
            let _ = store.complete(
                source_version_id,
                &state.config.environment_id,
                &result,
                &now(),
            );
        }
        Err(reason) => {
            let _ = store.fail(
                source_version_id,
                &state.config.environment_id,
                reason,
                &now(),
            );
        }
    }
    publish_processing_outbox(state, store).await;
}

async fn publish_processing_outbox(state: &AppState, store: &ProcessingStore) {
    let Ok(events) = store.pending_events() else {
        warn!("processing event outbox could not be read");
        return;
    };
    for event in events {
        let operational = processing_operational_event(&state.config, &event);
        let durable = jetstream::new(state.client.clone())
            .publish(
                PROCESSING_LIFECYCLE_EVENT_SUBJECT,
                match serde_json::to_vec(&operational) {
                    Ok(bytes) => bytes.into(),
                    Err(_) => break,
                },
            )
            .await;
        let durable = match durable {
            Ok(acknowledgement) => acknowledgement.await.is_ok(),
            Err(_) => false,
        };
        if durable && publish_event(state, &operational).await.is_ok() {
            if store.mark_published(&event.event_id, &now()).is_err() {
                warn!(
                    event_id = event.event_id,
                    "processing event publication could not be concluded"
                );
            }
        } else {
            warn!(
                event_id = event.event_id,
                "processing event remains pending durable publication"
            );
            break;
        }
    }
}

fn processing_operational_event(
    config: &Config,
    processing: &ProcessingLifecycleEvent,
) -> OperationalEvent {
    let status = match processing.event_type.as_str() {
        "processing.completed" => "completed",
        "processing.failed" => "failed",
        "processing.started" => "processing",
        _ => "accepted",
    };
    OperationalEvent {
        contract_id: "O-001".to_owned(),
        contract_version: O001_VERSION.to_owned(),
        event_id: processing.event_id.clone(),
        event_type: processing.event_type.clone(),
        component_id: config.component.id.to_owned(),
        component_name: config.component.name.to_owned(),
        instance_id: config.instance_id.clone(),
        workload_identity: config.workload_identity.clone(),
        environment_id: config.environment_id.clone(),
        status: status.to_owned(),
        capability: config.component.capability.to_owned(),
        source_revision: config.source_revision.clone(),
        image_digest: config.image_digest.clone(),
        occurred_at: processing.occurred_at.clone(),
        information_profile: INFORMATION_PROFILE.to_owned(),
        command_name: Some("process-staged-source".to_owned()),
        correlation_id: Some(processing.correlation_id.clone()),
        causation_id: Some(processing.causation_id.clone()),
        idempotency_key: Some(format!(
            "processing:{}:{}",
            processing.source_version_id, processing.event_type
        )),
        reason_code: processing.reason_code.clone(),
        subject_reference: Some(processing.source_version_id.clone()),
    }
}

fn source_operational_event(config: &Config, source: &SourceLifecycleEvent) -> OperationalEvent {
    OperationalEvent {
        contract_id: "O-001".to_owned(),
        contract_version: O001_VERSION.to_owned(),
        event_id: source.event_id.clone(),
        event_type: source.event_type.clone(),
        component_id: config.component.id.to_owned(),
        component_name: config.component.name.to_owned(),
        instance_id: config.instance_id.clone(),
        workload_identity: config.workload_identity.clone(),
        environment_id: config.environment_id.clone(),
        status: if source.event_type.ends_with("refused") {
            "refused"
        } else {
            "accepted"
        }
        .to_owned(),
        capability: config.component.capability.to_owned(),
        source_revision: config.source_revision.clone(),
        image_digest: config.image_digest.clone(),
        occurred_at: source.occurred_at.clone(),
        information_profile: INFORMATION_PROFILE.to_owned(),
        command_name: Some(
            match source.event_type.as_str() {
                "source.validated" | "source.validation-refused" => "validate-source",
                "source.staged" | "source.staging-refused" | "source.stage-duplicate" => {
                    "release-to-staging"
                }
                _ => "submit-to-quarantine",
            }
            .to_owned(),
        ),
        correlation_id: Some(source.correlation_id.clone()),
        causation_id: Some(source.causation_id.clone()),
        idempotency_key: None,
        reason_code: source.reason_code.clone(),
        subject_reference: Some(source.source_version_id.clone()),
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
        source_state_path: if component.id == "CNT-01" {
            Some(PathBuf::from(required_env("PPL_SOURCE_STATE_PATH")?))
        } else {
            None
        },
        processing_state_path: if component.id == "KNO-01" {
            Some(PathBuf::from(required_env("PPL_PROCESSING_STATE_PATH")?))
        } else {
            None
        },
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
    format_time(OffsetDateTime::now_utc())
}

fn format_time(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .unwrap_or_else(|_| "time-unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        COMPONENTS, PROBE_COMPONENTS, definition, evaluate_source_authorisation,
        source_authorisation_request,
    };
    use ppl_contracts::{AuthorisationDecisionStatus, SourceStageCommand};
    use time::OffsetDateTime;

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

    #[test]
    fn source_staging_policy_permits_only_the_exact_reviewer_context() {
        let command = SourceStageCommand {
            contract_id: "A-002".to_owned(),
            contract_version: "0.1.0".to_owned(),
            message_type: "staged-source-release.command".to_owned(),
            command_id: "stage-command:test-policy-0001".to_owned(),
            action: "release-to-staging".to_owned(),
            environment_id: "environment-test-0001".to_owned(),
            demonstration_session_id: "session:test-0001".to_owned(),
            engagement_id: "engagement:harbour-support-review".to_owned(),
            source_version_id: "source-version:test-policy-0001".to_owned(),
            actor_id: "synthetic-reviewer".to_owned(),
            actor_role: "workbench-reviewer".to_owned(),
            authority_reference: "application-session:test-policy-0001".to_owned(),
            purpose: "governed-source-staging".to_owned(),
            correlation_id: "session:test-0001".to_owned(),
            causation_id: "user-action:test-policy-0001".to_owned(),
            idempotency_key: "source-stage:test-policy-0001".to_owned(),
            requested_at: "2026-09-03T09:00:00Z".to_owned(),
        };
        let evaluated_at =
            OffsetDateTime::from_unix_timestamp(1_788_427_200).expect("fixed evaluation time");
        let request = source_authorisation_request(&command, evaluated_at);
        let permitted =
            evaluate_source_authorisation("environment-test-0001", &request, evaluated_at);
        assert_eq!(permitted.status, AuthorisationDecisionStatus::Permit);
        assert!(
            permitted
                .obligations
                .iter()
                .any(|obligation| obligation.code == "retain-staging-evidence")
        );

        let mut substituted = request;
        substituted.requested_roles = vec!["portal-viewer".to_owned()];
        let refused =
            evaluate_source_authorisation("environment-test-0001", &substituted, evaluated_at);
        assert_eq!(refused.status, AuthorisationDecisionStatus::Deny);
        assert_eq!(refused.reason_code, "source-staging-policy-refused");
    }
}
