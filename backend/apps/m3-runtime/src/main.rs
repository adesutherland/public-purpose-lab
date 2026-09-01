//! M3.4 executable adapter for the Director, Identity and Presentation workloads.

use std::{
    collections::BTreeMap, convert::Infallible, env, net::SocketAddr, path::PathBuf, sync::Arc,
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response, Sse, sse::Event},
    routing::{get, post},
};
use futures_util::StreamExt;
use ppl_contracts::{
    CommandOutcome, O001_VERSION, OperationalEvent, OutcomeStatus, PresentationCapabilityManifest,
    PresentationCue, PresentationCueOutcome, PresentationOutcomeResult, PresentationRegistration,
    ScenarioControlCommand, ScenarioLifecycleAction, ScenarioLifecycleCommand, ScenarioState,
    SyntheticSessionStatus,
};
use ppl_ctl_01::{DirectorError, DirectorRuntime};
use ppl_ctl_02::{PresentationError, PresentationRuntime};
use ppl_iam_01::{
    GrantRequest, IamError,
    application_grants::{ApplicationGrantStore, GrantStoreError},
    application_sessions::{ApplicationSessionStore, SessionError},
};
use ppl_int_01::nats::{
    Broker, BrokerConfig, BrokerError, CONTROL_OUTCOME_SUBJECT, CONTROL_SUBJECT, CUE_SUBJECT,
    DIRECTOR_EVENT_SUBJECT, GRANT_REQUEST_SUBJECT, IDENTITY_OUTCOME_SUBJECT, OUTCOME_SUBJECT,
    REGISTRATION_SUBJECT, SYNTHETIC_GRANT_SUBJECT, SYNTHETIC_TERMINATION_SUBJECT, WorkloadMode,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use uuid::Uuid;

const PACKAGE_ID: &str = "presentation-control-assurance";
const PACKAGE_VERSION: &str = "1.2.0";
const MANIFEST_JSON: &str =
    include_str!("../../../../contracts/presentation/examples/p-001-assurance-surface.json");

mod identity;
mod kms;
mod oidc;

use identity::{
    IdentityBroker, LOCAL_MAPPING_VERSION, LocalIdentityBroker, ManagedIdentityBroker,
    ManagedIdentityRuntimeConfig, SyntheticGrantDeliveryEvent, SyntheticGrantRequestEvent,
    SyntheticIdentityOutcomeEvent, SyntheticTerminationEvent, load_trust_bundle,
    local_external_identity,
};
use oidc::{OidcAuthenticator, OidcConfig, OidcError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeProfile {
    NativeDevelopment,
    LocalContainers,
    Minikube,
    PrivateHostedSmoke,
    ManagedHosted,
}

impl RuntimeProfile {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "native-development" => Ok(Self::NativeDevelopment),
            "local-containers" => Ok(Self::LocalContainers),
            "minikube" => Ok(Self::Minikube),
            "private-hosted-smoke" => Ok(Self::PrivateHostedSmoke),
            "managed-hosted" => Ok(Self::ManagedHosted),
            _ => Err(AppError::configuration("runtime-profile-invalid")),
        }
    }

    const fn interactive(self) -> bool {
        !matches!(self, Self::PrivateHostedSmoke)
    }

    const fn local_test_identity(self) -> bool {
        matches!(
            self,
            Self::NativeDevelopment | Self::LocalContainers | Self::Minikube
        )
    }

    const fn name(self) -> &'static str {
        match self {
            Self::NativeDevelopment => "native-development",
            Self::LocalContainers => "local-containers",
            Self::Minikube => "minikube",
            Self::PrivateHostedSmoke => "private-hosted-smoke",
            Self::ManagedHosted => "managed-hosted",
        }
    }
}

#[derive(Clone)]
struct AppConfig {
    mode: WorkloadMode,
    profile: RuntimeProfile,
    address: SocketAddr,
    state_path: PathBuf,
    static_directory: PathBuf,
    workbench_directory: Option<PathBuf>,
    package_directory: PathBuf,
    source_revision: String,
    image_digest: String,
    allowed_origin: String,
    broker: Option<BrokerConfig>,
    environment_id: String,
    security_state_path: PathBuf,
    identity_state_root: PathBuf,
    trust_bundle_path: PathBuf,
    oidc: Option<OidcConfig>,
    mapping_version: String,
    managed_identity: Option<ManagedIdentityRuntimeConfig>,
    gate_a_readiness: Option<GateAReadinessConfig>,
}

#[derive(Clone)]
struct GateAReadinessConfig {
    instance_id: String,
    workload_identity: String,
}

#[derive(Clone)]
struct DirectorState {
    config: AppConfig,
    runtime: DirectorRuntime,
    broker: Option<Broker>,
    sessions: ApplicationSessionStore,
    oidc: Option<Arc<OidcAuthenticator>>,
}

#[derive(Clone)]
struct GatewayState {
    config: AppConfig,
    runtime: PresentationRuntime,
    broker: Option<Broker>,
    manifest_digest: String,
    sessions: ApplicationSessionStore,
    oidc: Option<Arc<OidcAuthenticator>>,
    grants: Arc<std::sync::OnceLock<ApplicationGrantStore>>,
    cue_channel: broadcast::Sender<PresentationCue>,
}

#[derive(Clone)]
struct IdentityState {
    config: AppConfig,
    identity: Arc<IdentityBroker>,
    broker: Broker,
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    code: &'static str,
}

impl AppError {
    const fn configuration(code: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code,
        }
    }

    const fn refused(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }

    const fn unauthorised(code: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "status": "refused",
                "code": self.code,
                "maturity": "in-development",
                "informationProfile": "synthetic-only"
            })),
        )
            .into_response()
    }
}

impl From<DirectorError> for AppError {
    fn from(error: DirectorError) -> Self {
        warn!(reason = %error, "director operation refused");
        AppError::refused("director-operation-refused")
    }
}

impl From<PresentationError> for AppError {
    fn from(error: PresentationError) -> Self {
        warn!(reason = %error, "presentation operation refused");
        AppError::refused("presentation-operation-refused")
    }
}

impl From<BrokerError> for AppError {
    fn from(error: BrokerError) -> Self {
        error!(reason = %error, "broker operation failed");
        AppError::configuration("broker-operation-failed")
    }
}

impl From<SessionError> for AppError {
    fn from(error: SessionError) -> Self {
        warn!(reason = %error, "application session operation refused");
        AppError::unauthorised(error.reason_code())
    }
}

impl From<IamError> for AppError {
    fn from(error: IamError) -> Self {
        error!(reason = %error, "identity broker operation failed safely");
        AppError::configuration("identity-operation-failed")
    }
}

impl From<GrantStoreError> for AppError {
    fn from(error: GrantStoreError) -> Self {
        error!(reason = %error, "synthetic establishment state failed safely");
        AppError::configuration("synthetic-establishment-failed")
    }
}

impl From<OidcError> for AppError {
    fn from(error: OidcError) -> Self {
        warn!(reason = %error, "external identity operation refused");
        AppError::unauthorised(error.reason_code())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ppl_m3_runtime=info,tower_http=info".into()),
        )
        .with_target(false)
        .compact()
        .init();
    if let Err(error) = run().await {
        error!(code = error.code, "M3.4 runtime stopped safely");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| AppError::configuration("tls-crypto-provider-conflict"))?;
    let config = load_config()?;
    match config.mode {
        WorkloadMode::ScenarioDirector => run_director(config).await,
        WorkloadMode::PresentationGateway => run_gateway(config).await,
        WorkloadMode::IdentityBroker => run_identity(config).await,
    }
}

async fn connect_broker(config: &AppConfig) -> Result<Option<Broker>, AppError> {
    let Some(broker_config) = config.broker.clone() else {
        return Ok(None);
    };
    let broker = Broker::connect(broker_config).await?;
    broker.ensure_stream().await?;
    Ok(Some(broker))
}

async fn run_director(config: AppConfig) -> Result<(), AppError> {
    let runtime = DirectorRuntime::open(&config.state_path)?;
    runtime.admit_bundled_package(
        &config.package_directory,
        &config.source_revision,
        &config.image_digest,
        OffsetDateTime::now_utc(),
    )?;
    let broker = connect_broker(&config).await?;
    let oidc = discover_oidc(&config).await?;
    let state = DirectorState {
        config: config.clone(),
        runtime,
        broker,
        sessions: ApplicationSessionStore::open(
            &config.security_state_path,
            &config.environment_id,
            "scenario-director",
        )?,
        oidc,
    };
    if let Some(broker) = state.broker.clone() {
        spawn_gate_a_readiness(&state.config, broker.clone());
        tokio::spawn(director_consumer(state.clone(), broker.clone()));
        tokio::spawn(director_outbox(state.clone(), broker));
    }
    let router = common_layers(director_router(state), &config);
    serve(router, &config).await
}

async fn run_gateway(config: AppConfig) -> Result<(), AppError> {
    let runtime = PresentationRuntime::open(&config.state_path)?;
    let manifest: PresentationCapabilityManifest = serde_json::from_str(MANIFEST_JSON)
        .map_err(|_| AppError::configuration("manifest-invalid"))?;
    let manifest_digest = runtime.admit_manifest(&manifest, OffsetDateTime::now_utc())?;
    let broker = connect_broker(&config).await?;
    let (cue_channel, _) = broadcast::channel(32);
    let oidc = discover_oidc(&config).await?;
    let state = GatewayState {
        config: config.clone(),
        runtime,
        broker,
        manifest_digest,
        sessions: ApplicationSessionStore::open(
            &config.security_state_path,
            &config.environment_id,
            "presentation-gateway",
        )?,
        oidc,
        grants: Arc::new(std::sync::OnceLock::new()),
        cue_channel,
    };
    if config.profile.interactive() {
        let _ = gateway_grant_store(&state)?;
    }
    if let Some(broker) = state.broker.clone() {
        spawn_gate_a_readiness(&state.config, broker.clone());
        tokio::spawn(gateway_consumer(state.clone(), broker.clone()));
        tokio::spawn(gateway_outbox(state.clone(), broker));
    }
    let router = common_layers(gateway_router(state), &config);
    serve(router, &config).await
}

async fn discover_oidc(config: &AppConfig) -> Result<Option<Arc<OidcAuthenticator>>, AppError> {
    match &config.oidc {
        Some(oidc) => {
            let discovered = OidcAuthenticator::discover(oidc.clone()).await?;
            if discovered.mapping_version() != config.mapping_version {
                return Err(AppError::configuration("role-mapping-version-mismatch"));
            }
            Ok(Some(Arc::new(discovered)))
        }
        None => Ok(None),
    }
}

async fn run_identity(config: AppConfig) -> Result<(), AppError> {
    let identity = if config.profile.local_test_identity() {
        IdentityBroker::Local(Box::new(LocalIdentityBroker::open(
            &config.identity_state_root,
            &config.trust_bundle_path,
            &config.environment_id,
            OffsetDateTime::now_utc(),
        )?))
    } else if config.profile == RuntimeProfile::ManagedHosted {
        let managed = config
            .managed_identity
            .as_ref()
            .ok_or(AppError::configuration("managed-identity-binding-required"))?;
        IdentityBroker::Managed(Box::new(
            ManagedIdentityBroker::open(
                &config.security_state_path,
                &config.trust_bundle_path,
                &managed.configuration_path,
                &config.environment_id,
                managed.kms_key_version.clone(),
                &managed.project_id,
            )
            .await?,
        ))
    } else {
        return Err(AppError::configuration("managed-identity-binding-required"));
    };
    let identity = Arc::new(identity);
    let broker = connect_broker(&config)
        .await?
        .ok_or(AppError::configuration("interactive-broker-unavailable"))?;
    let state = IdentityState {
        config: config.clone(),
        identity,
        broker: broker.clone(),
    };
    spawn_gate_a_readiness(&state.config, broker.clone());
    tokio::spawn(identity_consumer(state.clone(), broker));
    let router = Router::new()
        .route("/health/live", get(identity_liveness))
        .route("/health/ready", get(identity_readiness))
        .route("/health/contracts", get(identity_contracts))
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    serve(router, &config).await
}

async fn serve(router: Router, config: &AppConfig) -> Result<(), AppError> {
    let listener = tokio::net::TcpListener::bind(config.address)
        .await
        .map_err(|_| AppError::configuration("listener-unavailable"))?;
    info!(
        mode = ?config.mode,
        profile = config.profile.name(),
        address = %config.address,
        "M3.4 runtime listening"
    );
    axum::serve(listener, router)
        .await
        .map_err(|_| AppError::configuration("listener-failed"))
}

fn common_layers(router: Router, config: &AppConfig) -> Router {
    let index = config.static_directory.join("index.html");
    let service = ServeDir::new(&config.static_directory)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(index));
    let router = if let Some(workbench) = &config.workbench_directory {
        router.nest_service(
            "/workbench",
            ServeDir::new(workbench)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(workbench.join("index.html"))),
        )
    } else {
        router
    };
    router
        .fallback_service(service)
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'",
            ),
        ))
        .layer(TraceLayer::new_for_http())
}

#[allow(clippy::too_many_lines)]
fn load_config() -> Result<AppConfig, AppError> {
    let mode = match required_env("PPL_RUNTIME_MODE")?.as_str() {
        "scenario-director" => WorkloadMode::ScenarioDirector,
        "presentation-gateway" => WorkloadMode::PresentationGateway,
        "identity-broker" => WorkloadMode::IdentityBroker,
        _ => return Err(AppError::configuration("runtime-mode-invalid")),
    };
    let profile = RuntimeProfile::parse(&required_env("PPL_RUNTIME_PROFILE")?)?;
    let default_port = match mode {
        WorkloadMode::ScenarioDirector => 18_081,
        WorkloadMode::PresentationGateway => 18_082,
        WorkloadMode::IdentityBroker => 18_083,
    };
    let address = env::var("PPL_LISTEN_ADDRESS")
        .unwrap_or_else(|_| format!("127.0.0.1:{default_port}"))
        .parse::<SocketAddr>()
        .map_err(|_| AppError::configuration("listen-address-invalid"))?;
    if profile == RuntimeProfile::NativeDevelopment && !address.ip().is_loopback() {
        return Err(AppError::configuration(
            "interactive-listener-must-be-loopback",
        ));
    }
    let mode_name = match mode {
        WorkloadMode::ScenarioDirector => "scenario-director",
        WorkloadMode::PresentationGateway => "presentation-gateway",
        WorkloadMode::IdentityBroker => "identity-broker",
    };
    let static_name = match mode {
        WorkloadMode::ScenarioDirector | WorkloadMode::IdentityBroker => "director",
        WorkloadMode::PresentationGateway => "presentation",
    };
    let broker = if profile == RuntimeProfile::PrivateHostedSmoke {
        None
    } else {
        Some(load_broker_config(mode)?)
    };
    let environment_id = load_environment_id()?;
    let security_state_path = env::var("PPL_SECURITY_STATE_PATH").map_or_else(
        |_| PathBuf::from(format!("var/m3/{mode_name}-security.sqlite")),
        PathBuf::from,
    );
    let allowed_origin = env::var("PPL_ALLOWED_ORIGIN")
        .unwrap_or_else(|_| format!("http://127.0.0.1:{default_port}"));
    let (oidc, mapping_version) =
        if profile == RuntimeProfile::ManagedHosted && mode != WorkloadMode::IdentityBroker {
            let role_mapping_path = PathBuf::from(required_env("PPL_OIDC_ROLE_MAPPING_PATH")?);
            let mapping_version = load_mapping_version(&role_mapping_path)?;
            let required_role = required_role(mode).to_owned();
            let redirect_uri = required_env("PPL_GOOGLE_OIDC_REDIRECT_URI")?;
            validate_managed_web_binding(&allowed_origin, &redirect_uri)?;
            (
                Some(OidcConfig {
                    issuer: "https://accounts.google.com".to_owned(),
                    client_id: required_env("PPL_GOOGLE_OIDC_CLIENT_ID")?,
                    client_secret: read_protected_text("PPL_GOOGLE_OIDC_CLIENT_SECRET_FILE")?,
                    redirect_uri,
                    environment_id: environment_id.clone(),
                    audience: mode_name.to_owned(),
                    required_role,
                    role_mapping_path,
                    flow_state_path: security_state_path.with_extension("oidc.sqlite"),
                }),
                mapping_version,
            )
        } else {
            (None, LOCAL_MAPPING_VERSION.to_owned())
        };
    let managed_identity =
        if profile == RuntimeProfile::ManagedHosted && mode == WorkloadMode::IdentityBroker {
            Some(ManagedIdentityRuntimeConfig {
                configuration_path: PathBuf::from(required_env(
                    "PPL_MANAGED_IDENTITY_CONFIGURATION_PATH",
                )?),
                kms_key_version: required_env("PPL_KMS_ISSUER_KEY_VERSION")?,
                project_id: required_env("PPL_GCP_PROJECT_ID")?,
            })
        } else {
            None
        };
    Ok(AppConfig {
        mode,
        profile,
        address,
        state_path: env::var("PPL_STATE_PATH").map_or_else(
            |_| PathBuf::from(format!("var/m3/{mode_name}.sqlite")),
            PathBuf::from,
        ),
        static_directory: env::var("PPL_STATIC_DIRECTORY").map_or_else(
            |_| PathBuf::from(format!("frontend/apps/{static_name}/dist")),
            PathBuf::from,
        ),
        workbench_directory: if mode == WorkloadMode::PresentationGateway {
            Some(env::var("PPL_WORKBENCH_DIRECTORY").map_or_else(
                |_| PathBuf::from("frontend/apps/workbench/dist"),
                PathBuf::from,
            ))
        } else {
            None
        },
        package_directory: env::var("PPL_PACKAGE_DIRECTORY").map_or_else(
            |_| PathBuf::from("scenarios/presentation-control-assurance"),
            PathBuf::from,
        ),
        source_revision: env::var("PPL_SOURCE_REVISION")
            .unwrap_or_else(|_| "working-tree".to_owned()),
        image_digest: env::var("PPL_IMAGE_DIGEST")
            .unwrap_or_else(|_| "native-development".to_owned()),
        allowed_origin,
        broker,
        environment_id,
        security_state_path,
        identity_state_root: env::var("PPL_IDENTITY_STATE_ROOT")
            .map_or_else(|_| PathBuf::from("var/m3/identity"), PathBuf::from),
        trust_bundle_path: env::var("PPL_TRUST_BUNDLE_PATH").map_or_else(
            |_| PathBuf::from("var/m3/identity-public/trust-bundle.json"),
            PathBuf::from,
        ),
        oidc,
        mapping_version,
        managed_identity,
        gate_a_readiness: load_gate_a_readiness()?,
    })
}

fn load_gate_a_readiness() -> Result<Option<GateAReadinessConfig>, AppError> {
    if !env::var("PPL_GATE_A_READINESS").is_ok_and(|value| value == "1") {
        return Ok(None);
    }
    let instance_id = required_env("PPL_INSTANCE_ID")?;
    let workload_identity_path = required_env("PPL_WORKLOAD_IDENTITY_FILE")?;
    let workload_identity = std::fs::read_to_string(workload_identity_path)
        .map_err(|_| AppError::configuration("workload-identity-unavailable"))?
        .trim()
        .to_owned();
    if instance_id.is_empty() || workload_identity.is_empty() {
        return Err(AppError::configuration("gate-a-identity-invalid"));
    }
    Ok(Some(GateAReadinessConfig {
        instance_id,
        workload_identity,
    }))
}

fn spawn_gate_a_readiness(config: &AppConfig, broker: Broker) {
    let Some(gate_a) = config.gate_a_readiness.clone() else {
        return;
    };
    let (component_id, component_name, capability) = match config.mode {
        WorkloadMode::ScenarioDirector => (
            "CTL-01",
            "scenario-director",
            "scenario lifecycle and presentation orchestration",
        ),
        WorkloadMode::PresentationGateway => (
            "CTL-02",
            "presentation-gateway",
            "target-owned semantic presentation",
        ),
        WorkloadMode::IdentityBroker => (
            "IAM-01",
            "identity-broker",
            "environment-scoped synthetic identity",
        ),
    };
    let environment_id = config.environment_id.clone();
    let source_revision = config.source_revision.clone();
    let image_digest = config.image_digest.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let event = OperationalEvent {
                contract_id: "O-001".to_owned(),
                contract_version: O001_VERSION.to_owned(),
                event_id: format!("event:{}", Uuid::new_v4()),
                event_type: "component.ready".to_owned(),
                component_id: component_id.to_owned(),
                component_name: component_name.to_owned(),
                instance_id: gate_a.instance_id.clone(),
                workload_identity: gate_a.workload_identity.clone(),
                environment_id: environment_id.clone(),
                status: "ready".to_owned(),
                capability: capability.to_owned(),
                source_revision: source_revision.clone(),
                image_digest: image_digest.clone(),
                occurred_at: OffsetDateTime::now_utc()
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| "time-unavailable".to_owned()),
                information_profile: "synthetic-only".to_owned(),
                command_name: None,
                correlation_id: None,
                causation_id: None,
                idempotency_key: None,
                reason_code: None,
            };
            if let Err(error) = broker.publish_operational_event(&event).await {
                warn!(reason = %error, component_id, "Gate A readiness event was not published");
            }
        }
    });
}

struct GateBEvent<'a> {
    event_type: &'a str,
    status: &'a str,
    command_name: Option<&'a str>,
    correlation_id: Option<&'a str>,
    causation_id: Option<&'a str>,
    reason_code: Option<&'a str>,
}

async fn publish_gate_b_event(
    config: &AppConfig,
    broker: Option<&Broker>,
    details: GateBEvent<'_>,
) {
    let (Some(broker), Some(instance)) = (broker, config.gate_a_readiness.as_ref()) else {
        return;
    };
    let (component_id, component_name, capability) = match config.mode {
        WorkloadMode::ScenarioDirector => (
            "CTL-01",
            "scenario-director",
            "scenario lifecycle and presentation orchestration",
        ),
        WorkloadMode::PresentationGateway => (
            "CTL-02",
            "presentation-gateway",
            "target-owned semantic presentation",
        ),
        WorkloadMode::IdentityBroker => (
            "IAM-01",
            "identity-broker",
            "environment-scoped synthetic identity",
        ),
    };
    let event = OperationalEvent {
        contract_id: "O-001".to_owned(),
        contract_version: O001_VERSION.to_owned(),
        event_id: format!("event:{}", Uuid::new_v4()),
        event_type: details.event_type.to_owned(),
        component_id: component_id.to_owned(),
        component_name: component_name.to_owned(),
        instance_id: instance.instance_id.clone(),
        workload_identity: instance.workload_identity.clone(),
        environment_id: config.environment_id.clone(),
        status: details.status.to_owned(),
        capability: capability.to_owned(),
        source_revision: config.source_revision.clone(),
        image_digest: config.image_digest.clone(),
        occurred_at: now_string().unwrap_or_else(|_| "time-unavailable".to_owned()),
        information_profile: "synthetic-only".to_owned(),
        command_name: details.command_name.map(str::to_owned),
        correlation_id: details.correlation_id.map(str::to_owned),
        causation_id: details.causation_id.map(str::to_owned),
        idempotency_key: None,
        reason_code: details.reason_code.map(str::to_owned),
    };
    if let Err(error) = broker.publish_operational_event(&event).await {
        warn!(reason = %error, event_type = details.event_type, "Gate B operational event was not published");
    }
}

fn validate_managed_web_binding(allowed_origin: &str, redirect_uri: &str) -> Result<(), AppError> {
    let parsed = url::Url::parse(allowed_origin)
        .map_err(|_| AppError::configuration("managed-origin-invalid"))?;
    if parsed.scheme() != "https"
        || allowed_origin != parsed.origin().ascii_serialization()
        || redirect_uri != format!("{allowed_origin}/auth/google/callback")
    {
        return Err(AppError::configuration("managed-origin-invalid"));
    }
    Ok(())
}

#[cfg(test)]
mod configuration_tests {
    use super::validate_managed_web_binding;

    #[test]
    fn managed_web_binding_requires_exact_https_origin_and_callback() {
        assert!(
            validate_managed_web_binding(
                "https://demo.example.org",
                "https://demo.example.org/auth/google/callback"
            )
            .is_ok()
        );
        assert!(
            validate_managed_web_binding(
                "http://demo.example.org",
                "http://demo.example.org/auth/google/callback"
            )
            .is_err()
        );
        assert!(
            validate_managed_web_binding(
                "https://demo.example.org/path",
                "https://demo.example.org/path/auth/google/callback"
            )
            .is_err()
        );
        assert!(
            validate_managed_web_binding(
                "https://demo.example.org",
                "https://other.example.org/auth/google/callback"
            )
            .is_err()
        );
    }
}

fn read_protected_text(variable: &'static str) -> Result<String, AppError> {
    let path = required_env(variable)?;
    let value = std::fs::read_to_string(path)
        .map_err(|_| AppError::configuration("protected-configuration-unavailable"))?;
    let value = value.trim().to_owned();
    if value.is_empty() {
        Err(AppError::configuration("protected-configuration-invalid"))
    } else {
        Ok(value)
    }
}

fn load_mapping_version(path: &std::path::Path) -> Result<String, AppError> {
    let value: Value = serde_json::from_slice(
        &std::fs::read(path).map_err(|_| AppError::configuration("role-mapping-unavailable"))?,
    )
    .map_err(|_| AppError::configuration("role-mapping-invalid"))?;
    value
        .get("mappingVersion")
        .and_then(Value::as_str)
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
        .ok_or(AppError::configuration("role-mapping-invalid"))
}

fn load_environment_id() -> Result<String, AppError> {
    let value = if let Ok(value) = env::var("PPL_ENVIRONMENT_ID") {
        value
    } else {
        let path = required_env("PPL_ENVIRONMENT_ID_FILE")?;
        std::fs::read_to_string(path)
            .map_err(|_| AppError::configuration("environment-id-unavailable"))?
    };
    let value = value.trim().to_owned();
    if value.len() < 8
        || value.len() > 128
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | ':' | '_')
        })
    {
        return Err(AppError::configuration("environment-id-invalid"));
    }
    Ok(value)
}

fn load_broker_config(mode: WorkloadMode) -> Result<BrokerConfig, AppError> {
    let credentials_file = env::var("PPL_NATS_CREDENTIALS").ok().map(PathBuf::from);
    let nkey_seed_file = env::var("PPL_NATS_NKEY_SEED_FILE").ok().map(PathBuf::from);
    let explicitly_insecure =
        env::var("PPL_ALLOW_INSECURE_LOCAL_BROKER").is_ok_and(|value| value == "1");
    if credentials_file.is_none() && nkey_seed_file.is_none() && !explicitly_insecure {
        return Err(AppError::configuration("broker-credentials-required"));
    }
    Ok(BrokerConfig {
        url: env::var("PPL_NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned()),
        credentials_file,
        nkey_seed_file,
        root_certificate: env::var("PPL_NATS_ROOT_CERTIFICATE")
            .ok()
            .map(PathBuf::from),
        client_certificate: env::var("PPL_NATS_CLIENT_CERTIFICATE")
            .ok()
            .map(PathBuf::from),
        client_key: env::var("PPL_NATS_CLIENT_KEY").ok().map(PathBuf::from),
        workload_mode: mode,
    })
}

fn required_env(name: &'static str) -> Result<String, AppError> {
    env::var(name).map_err(|_| AppError::configuration("required-configuration-missing"))
}

fn director_router(state: DirectorState) -> Router {
    Router::new()
        .route("/health/live", get(director_liveness))
        .route("/health/ready", get(director_readiness))
        .route("/health/contracts", get(director_contracts))
        .route("/api/v1/development-session", post(director_login))
        .route("/api/v1/login-mode", get(director_login_mode))
        .route("/api/v1/environment", get(director_environment))
        .route("/auth/google/start", get(director_oidc_start))
        .route("/auth/google/callback", get(director_oidc_callback))
        .route("/api/v1/session-context", get(director_session_context))
        .route("/api/v1/logout", post(director_logout))
        .route("/api/v1/status/{session_id}", get(director_status))
        .route("/api/v1/sessions", post(create_session))
        .route(
            "/api/v1/sessions/{session_id}/lifecycle",
            post(apply_lifecycle),
        )
        .route("/api/v1/sessions/{session_id}/cue", post(issue_cue))
        .route(
            "/api/v1/sessions/{session_id}/logical-time",
            post(advance_time),
        )
        .route(
            "/api/v1/sessions/{session_id}/cue-delay",
            post(request_cue_delay),
        )
        .route(
            "/api/v1/sessions/{session_id}/synthetic-sign-in",
            post(request_synthetic_sign_in),
        )
        .with_state(state)
}

fn gateway_router(state: GatewayState) -> Router {
    Router::new()
        .route("/health/live", get(gateway_liveness))
        .route("/health/ready", get(gateway_readiness))
        .route("/health/contracts", get(gateway_contracts))
        .route("/api/v1/development-session", post(gateway_login))
        .route("/api/v1/login-mode", get(gateway_login_mode))
        .route("/auth/google/start", get(gateway_oidc_start))
        .route("/auth/google/callback", get(gateway_oidc_callback))
        .route("/api/v1/logout", post(gateway_logout))
        .route("/api/v1/registrations", post(register_surface))
        .route("/api/v1/session-context", get(gateway_session_context))
        .route("/api/v1/cues", get(cue_events))
        .route("/api/v1/outcomes", post(record_outcome))
        .with_state(state)
}

async fn director_liveness(State(state): State<DirectorState>) -> Json<Value> {
    Json(liveness_json(&state.config))
}

async fn gateway_liveness(State(state): State<GatewayState>) -> Json<Value> {
    Json(liveness_json(&state.config))
}

async fn identity_liveness(State(state): State<IdentityState>) -> Json<Value> {
    Json(liveness_json(&state.config))
}

async fn identity_readiness(State(state): State<IdentityState>) -> Response {
    let trust = state.identity.trust();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "softwareStatus": "healthy",
            "identityReady": true,
            "environmentId": trust.record.environment_id,
            "trustDomain": trust.record.trust_domain,
            "trustEpoch": trust.record.trust_epoch,
            "trustProfile": trust.record.trust_profile,
            "keyCustodyClass": trust.record.key_custody_class,
            "componentChannelAuthorised": state.broker.can_publish(SYNTHETIC_GRANT_SUBJECT),
            "brokerAvailabilityProfile": "single-server-development-assurance",
            "maturity": "in-development",
            "informationProfile": "synthetic-only"
        })),
    )
        .into_response()
}

async fn identity_contracts(State(state): State<IdentityState>) -> Json<Value> {
    Json(serde_json::json!({
        "selfTest": "passed",
        "contracts": ["I-001", "I-002", "I-003", "I-004", "I-005", "AZ-001"],
        "environmentId": state.config.environment_id,
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    }))
}

async fn director_contracts(State(state): State<DirectorState>) -> Result<Json<Value>, AppError> {
    let package = state
        .runtime
        .package_admission(PACKAGE_ID, PACKAGE_VERSION)?;
    Ok(Json(serde_json::json!({
        "selfTest": "passed",
        "contracts": ["D-001", "D-002", "D-003", "D-004", "P-001", "P-002", "P-003", "P-004"],
        "identityContracts": ["I-001", "I-004", "I-005"],
        "packageId": package.package_id,
        "packageVersion": package.package_version,
        "packageDigest": package.package_digest,
        "scenarioDigest": package.scenario_digest,
        "sourceRevision": package.source_revision,
        "imageDigest": package.image_digest,
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    })))
}

async fn gateway_contracts(State(state): State<GatewayState>) -> Json<Value> {
    Json(serde_json::json!({
        "selfTest": "passed",
        "contracts": ["P-001", "P-002", "P-003", "P-004"],
        "identityContracts": ["I-001", "I-004", "I-005"],
        "manifestId": "assurance-presentation-surface",
        "manifestVersion": "1.2.0",
        "manifestDigest": state.manifest_digest,
        "sourceRevision": state.config.source_revision,
        "imageDigest": state.config.image_digest,
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    }))
}

fn liveness_json(config: &AppConfig) -> Value {
    serde_json::json!({
        "softwareStatus": "healthy",
        "mode": format!("{:?}", config.mode),
        "profile": config.profile.name(),
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    })
}

async fn director_readiness(State(state): State<DirectorState>) -> Response {
    let identity_ready =
        state.config.profile != RuntimeProfile::ManagedHosted || state.oidc.is_some();
    readiness_response(&state.config, state.broker.is_some(), identity_ready)
}

async fn gateway_readiness(State(state): State<GatewayState>) -> Response {
    let identity_ready = (state.config.profile != RuntimeProfile::ManagedHosted
        || state.oidc.is_some())
        && gateway_grant_store(&state).is_ok();
    readiness_response(&state.config, state.broker.is_some(), identity_ready)
}

fn readiness_response(config: &AppConfig, broker_ready: bool, identity_ready: bool) -> Response {
    let interactive_ready = config.profile.interactive() && broker_ready && identity_ready;
    let status = if interactive_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let reason_code = if interactive_ready {
        Value::Null
    } else if !config.profile.interactive() {
        Value::String("interactive-profile-disabled".to_owned())
    } else if !broker_ready {
        Value::String("event-broker-unavailable".to_owned())
    } else {
        Value::String("identity-binding-unavailable".to_owned())
    };
    (
        status,
        Json(serde_json::json!({
            "softwareStatus": "healthy",
            "interactiveReady": interactive_ready,
            "trustProfile": if matches!(config.profile, RuntimeProfile::PrivateHostedSmoke | RuntimeProfile::ManagedHosted) { "managed" } else { "development-assurance" },
            "reasonCode": reason_code,
            "brokerAvailabilityProfile": "single-server-development-assurance",
            "maturity": "in-development",
            "informationProfile": "synthetic-only"
        })),
    )
        .into_response()
}

async fn director_login(
    State(state): State<DirectorState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    development_login(
        &state.config,
        &state.sessions,
        &headers,
        "local-presenter",
        "presenter",
        "scenario-director",
    )
}

async fn gateway_login(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    development_login(
        &state.config,
        &state.sessions,
        &headers,
        "local-surface-operator",
        "surface-operator",
        "presentation-gateway",
    )
}

async fn director_login_mode(State(state): State<DirectorState>) -> Json<Value> {
    Json(login_mode_json(&state.config))
}

async fn gateway_login_mode(State(state): State<GatewayState>) -> Json<Value> {
    Json(login_mode_json(&state.config))
}

fn login_mode_json(config: &AppConfig) -> Value {
    serde_json::json!({
        "mode": if config.profile.local_test_identity() { "local-test" } else { "google-oidc" },
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    })
}

async fn director_oidc_start(State(state): State<DirectorState>) -> Result<Response, AppError> {
    oidc_start(state.oidc.as_deref())
}

async fn gateway_oidc_start(State(state): State<GatewayState>) -> Result<Response, AppError> {
    oidc_start(state.oidc.as_deref())
}

fn oidc_start(oidc: Option<&OidcAuthenticator>) -> Result<Response, AppError> {
    let oidc = oidc.ok_or(AppError::unauthorised("google-oidc-unavailable"))?;
    let start = oidc.begin(OffsetDateTime::now_utc())?;
    let mut response = Redirect::temporary(&start.authorisation_url).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "PPL_OIDC_FLOW={}; HttpOnly; Secure; SameSite=Lax; Path=/auth/google/callback; Max-Age=600",
            start.flow_cookie
        ))
        .map_err(|_| AppError::configuration("session-cookie-invalid"))?,
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

#[derive(Deserialize)]
struct OidcCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn director_oidc_callback(
    State(state): State<DirectorState>,
    headers: HeaderMap,
    Query(query): Query<OidcCallbackQuery>,
) -> Result<Response, AppError> {
    oidc_callback(
        &state.config,
        &state.sessions,
        state.oidc.as_deref(),
        &headers,
        query,
    )
    .await
}

async fn gateway_oidc_callback(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Query(query): Query<OidcCallbackQuery>,
) -> Result<Response, AppError> {
    oidc_callback(
        &state.config,
        &state.sessions,
        state.oidc.as_deref(),
        &headers,
        query,
    )
    .await
}

async fn oidc_callback(
    config: &AppConfig,
    sessions: &ApplicationSessionStore,
    oidc: Option<&OidcAuthenticator>,
    headers: &HeaderMap,
    query: OidcCallbackQuery,
) -> Result<Response, AppError> {
    if config.profile != RuntimeProfile::ManagedHosted {
        return Err(AppError::configuration("oidc-profile-mismatch"));
    }
    let oidc = oidc.ok_or(AppError::unauthorised("google-oidc-unavailable"))?;
    if query.error.is_some() {
        return Err(AppError::unauthorised("oidc-provider-refused"));
    }
    let flow_cookie = cookie_value(headers, "PPL_OIDC_FLOW")
        .ok_or(AppError::unauthorised("oidc-flow-state-refused"))?;
    let identity = oidc
        .complete(
            flow_cookie,
            query
                .state
                .as_deref()
                .ok_or(AppError::unauthorised("oidc-flow-state-refused"))?,
            query
                .code
                .as_deref()
                .ok_or(AppError::unauthorised("oidc-provider-refused"))?,
            OffsetDateTime::now_utc(),
        )
        .await?;
    let credentials = sessions.establish(&identity, OffsetDateTime::now_utc())?;
    let mut response = Redirect::to("/").into_response();
    set_application_cookies(&mut response, &credentials, true)?;
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "PPL_OIDC_FLOW=; HttpOnly; Secure; SameSite=Lax; Path=/auth/google/callback; Max-Age=0",
        ),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

fn development_login(
    config: &AppConfig,
    sessions: &ApplicationSessionStore,
    headers: &HeaderMap,
    principal_id: &str,
    role: &str,
    audience: &str,
) -> Result<Response, AppError> {
    if !config.profile.local_test_identity() {
        return Err(AppError::unauthorised(
            "development-assurance-adapter-unavailable",
        ));
    }
    require_origin(config, headers)?;
    let now = OffsetDateTime::now_utc();
    let identity =
        local_external_identity(&config.environment_id, audience, principal_id, role, now)?;
    let credentials = sessions.establish(&identity, now)?;
    let mut response = Json(serde_json::json!({
        "status": "established",
        "principalId": principal_id,
        "roles": [role],
        "identityKind": "external-human-test-adapter",
        "expiresInSeconds": 1800,
        "maturity": "in-development",
        "informationProfile": "synthetic-only",
        "warning": "Synthetic development assurance only"
    }))
    .into_response();
    set_application_cookies(&mut response, &credentials, false)?;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

fn set_application_cookies(
    response: &mut Response,
    credentials: &ppl_iam_01::application_sessions::SessionCredentials,
    secure: bool,
) -> Result<(), AppError> {
    let secure_attribute = if secure { "; Secure" } else { "" };
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "PPL_APP_SESSION={}; HttpOnly{secure_attribute}; SameSite=Strict; Path=/; Max-Age=1800",
            credentials.token
        ))
        .map_err(|_| AppError::configuration("session-cookie-invalid"))?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "PPL_CSRF={}{secure_attribute}; SameSite=Strict; Path=/; Max-Age=1800",
            credentials.csrf_token
        ))
        .map_err(|_| AppError::configuration("session-cookie-invalid"))?,
    );
    Ok(())
}

fn authorise(
    config: &AppConfig,
    sessions: &ApplicationSessionStore,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    if !config.profile.interactive() {
        return Err(AppError::unauthorised(
            "development-assurance-adapter-unavailable",
        ));
    }
    require_origin(config, headers)?;
    let token = cookie_value(headers, "PPL_APP_SESSION")
        .ok_or(AppError::unauthorised("application-session-required"))?;
    let csrf = headers
        .get("x-ppl-csrf")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::unauthorised("csrf-required"))?;
    let role = required_role(config.mode);
    let mapping_version = current_mapping_version(config)?;
    sessions.authorise_write(
        token,
        csrf,
        role,
        &mapping_version,
        OffsetDateTime::now_utc(),
    )?;
    Ok(())
}

fn require_origin(config: &AppConfig, headers: &HeaderMap) -> Result<(), AppError> {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::unauthorised("origin-required"))?;
    if origin == config.allowed_origin {
        Ok(())
    } else {
        Err(AppError::unauthorised("origin-refused"))
    }
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|pair| pair.strip_prefix(&format!("{name}=")))
}

fn authorise_read(
    config: &AppConfig,
    sessions: &ApplicationSessionStore,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    if !config.profile.interactive() {
        return Err(AppError::unauthorised(
            "development-assurance-adapter-unavailable",
        ));
    }
    let token = cookie_value(headers, "PPL_APP_SESSION")
        .ok_or(AppError::unauthorised("application-session-required"))?;
    let mapping_version = current_mapping_version(config)?;
    sessions.authorise_read(
        token,
        required_role(config.mode),
        &mapping_version,
        OffsetDateTime::now_utc(),
    )?;
    Ok(())
}

fn current_mapping_version(config: &AppConfig) -> Result<String, AppError> {
    match &config.oidc {
        Some(oidc) => load_mapping_version(&oidc.role_mapping_path),
        None => Ok(config.mapping_version.clone()),
    }
}

const fn required_role(mode: WorkloadMode) -> &'static str {
    match mode {
        WorkloadMode::ScenarioDirector => "presenter",
        WorkloadMode::PresentationGateway => "surface-operator",
        WorkloadMode::IdentityBroker => "identity-broker-workload",
    }
}

async fn director_logout(
    State(state): State<DirectorState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    logout(&state.config, &state.sessions, &headers)
}

async fn gateway_logout(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    logout(&state.config, &state.sessions, &headers)
}

fn logout(
    config: &AppConfig,
    sessions: &ApplicationSessionStore,
    headers: &HeaderMap,
) -> Result<Response, AppError> {
    require_origin(config, headers)?;
    if let Some(token) = cookie_value(headers, "PPL_APP_SESSION") {
        let csrf = headers
            .get("x-ppl-csrf")
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::unauthorised("csrf-required"))?;
        let mapping_version = current_mapping_version(config)?;
        sessions.authorise_write(
            token,
            csrf,
            required_role(config.mode),
            &mapping_version,
            OffsetDateTime::now_utc(),
        )?;
        sessions.revoke(token, "external-user-logout", OffsetDateTime::now_utc())?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    let secure_attribute = if config.profile == RuntimeProfile::ManagedHosted {
        "; Secure"
    } else {
        ""
    };
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "PPL_APP_SESSION=; HttpOnly{secure_attribute}; SameSite=Strict; Path=/; Max-Age=0"
        ))
        .map_err(|_| AppError::configuration("session-cookie-invalid"))?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "PPL_CSRF={secure_attribute}; SameSite=Strict; Path=/; Max-Age=0"
        ))
        .map_err(|_| AppError::configuration("session-cookie-invalid"))?,
    );
    Ok(response)
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DirectorStatusQuery {
    surface_slot: Option<String>,
}

async fn director_status(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    Query(query): Query<DirectorStatusQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorise_read(&state.config, &state.sessions, &headers)?;
    let session = state.runtime.session(&session_id)?;
    let surface_slot = query.surface_slot.as_deref().unwrap_or("audience-display");
    let registration = state
        .runtime
        .current_registration(&session_id, surface_slot)
        .ok();
    let checkpoint = state.runtime.presentation_checkpoint(&session_id)?;
    Ok(Json(serde_json::json!({
        "session": session,
        "registration": registration,
        "presentationCheckpoint": checkpoint,
        "maturity": "in-development",
        "informationProfile": "synthetic-only",
        "warning": "Synthetic development assurance only"
    })))
}

async fn director_session_context(
    State(state): State<DirectorState>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let token = cookie_value(&headers, "PPL_APP_SESSION")
        .ok_or(AppError::unauthorised("application-session-required"))?;
    let mapping_version = current_mapping_version(&state.config)?;
    let authorised = state.sessions.authorise_read(
        token,
        "presenter",
        &mapping_version,
        OffsetDateTime::now_utc(),
    )?;
    Ok(Json(serde_json::json!({
        "externalPrincipalId": authorised.external_identity.principal_id,
        "roles": authorised.external_identity.roles,
        "expiresAt": authorised.expires_at,
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    })))
}

async fn director_environment(
    State(state): State<DirectorState>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorise_read(&state.config, &state.sessions, &headers)?;
    let control_path = if state.broker.is_some() {
        "ready"
    } else {
        "unavailable"
    };
    let (trust_profile, trust_description) = match state.config.profile {
        RuntimeProfile::NativeDevelopment
        | RuntimeProfile::LocalContainers
        | RuntimeProfile::Minikube => (
            "environment-local-synthetic-root",
            "Synthetic trust is generated for and valid only inside this environment.",
        ),
        RuntimeProfile::PrivateHostedSmoke | RuntimeProfile::ManagedHosted => (
            "managed-real-root",
            "Hosted trust uses the environment's managed non-synthetic root binding.",
        ),
    };
    let surface_url = |variable: &str, default: &str| {
        env::var(variable).ok().or_else(|| {
            state
                .config
                .profile
                .local_test_identity()
                .then(|| default.to_owned())
        })
    };
    Ok(Json(serde_json::json!({
        "environmentId": state.config.environment_id,
        "runtimeProfile": state.config.profile.name(),
        "trustProfile": trust_profile,
        "trustDescription": trust_description,
        "eventInfrastructure": control_path,
        "presentationSurfaceUrl": surface_url(
            "PPL_PRESENTATION_SURFACE_URL",
            "http://127.0.0.1:18082/"
        ),
        "workbenchSurfaceUrl": surface_url(
            "PPL_WORKBENCH_SURFACE_URL",
            "http://127.0.0.1:18082/workbench/"
        ),
        "componentReadinessUrl": surface_url(
            "PPL_OPERATIONS_SURFACE_URL",
            "http://127.0.0.1:18084/"
        ),
        "catalogue": [{
            "scenarioId": "governed-source-assurance",
            "title": "Governed source assurance",
            "purpose": "Show environment-scoped identity and robust semantic portal orchestration before governed source intake.",
            "maturity": "Gate B implementation",
            "estimatedDuration": "8 minutes",
            "actors": ["external presenter", "synthetic-reviewer"],
            "requiredComponents": ["CTL-01", "CTL-02", "IAM-01", "OPS-01", "INT-01"],
            "status": control_path,
            "reasons": if state.broker.is_some() {
                Vec::<String>::new()
            } else {
                vec!["Authenticated event infrastructure is unavailable.".to_owned()]
            },
            "limitations": [
                "Synthetic information only.",
                "Gate B changes views but performs no engagement, source, workflow or reporting business operation."
            ]
        }],
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CreateSessionRequest {
    session_id: Option<String>,
}

async fn create_session(
    State(state): State<DirectorState>,
    headers: HeaderMap,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<Value>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("session:{}", Uuid::new_v4()));
    let command = ScenarioLifecycleCommand {
        contract_id: "D-002".to_owned(),
        contract_version: "1.0.0".to_owned(),
        operation_id: format!("operation:{}", Uuid::new_v4()),
        session_id: session_id.clone(),
        package_id: PACKAGE_ID.to_owned(),
        package_version: PACKAGE_VERSION.to_owned(),
        action: ScenarioLifecycleAction::Create,
        expected_state: None,
        expected_revision: 0,
        requested_at: now_string()?,
        reason: Some("Create the synthetic assurance demonstration session".to_owned()),
    };
    let outcome = state
        .runtime
        .apply_lifecycle(&command, OffsetDateTime::now_utc())?;
    publish_gate_b_event(
        &state.config,
        state.broker.as_ref(),
        GateBEvent {
            event_type: "scenario.started",
            status: "accepted",
            command_name: Some("create-demonstration-session"),
            correlation_id: Some(&session_id),
            causation_id: Some(&command.operation_id),
            reason_code: None,
        },
    )
    .await;
    Ok(Json(serde_json::to_value(outcome).map_err(|_| {
        AppError::configuration("response-invalid")
    })?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct LifecycleRequest {
    action: ScenarioLifecycleAction,
    expected_state: Option<ScenarioState>,
    expected_revision: u64,
    reason: Option<String>,
}

async fn apply_lifecycle(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<LifecycleRequest>,
) -> Result<Json<Value>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    if request.action == ScenarioLifecycleAction::Reset {
        let current = state.runtime.session(&session_id)?;
        if current.revision != request.expected_revision
            || request
                .expected_state
                .is_some_and(|value| value != current.state)
            || !matches!(
                current.state,
                ScenarioState::Completed | ScenarioState::Stopped | ScenarioState::Failed
            )
        {
            return Err(AppError::refused("reset-precondition-refused"));
        }
        let control = ScenarioControlCommand {
            contract_id: "D-003".to_owned(),
            contract_version: "1.0.0".to_owned(),
            operation_id: format!("operation:{}", Uuid::new_v4()),
            session_id: session_id.clone(),
            kind: "reset".to_owned(),
            operation: "execute".to_owned(),
            target: Some("presentation-registry-baseline".to_owned()),
            logical_instant: None,
            advance_seconds: None,
            delay_milliseconds: None,
            expected_revision: request.expected_revision,
            requested_at: now_string()?,
        };
        coordinate_control(&state, &control).await?;
    }
    let command = ScenarioLifecycleCommand {
        contract_id: "D-002".to_owned(),
        contract_version: "1.0.0".to_owned(),
        operation_id: format!("operation:{}", Uuid::new_v4()),
        session_id: session_id.clone(),
        package_id: PACKAGE_ID.to_owned(),
        package_version: PACKAGE_VERSION.to_owned(),
        action: request.action,
        expected_state: request.expected_state,
        expected_revision: request.expected_revision,
        requested_at: now_string()?,
        reason: request.reason,
    };
    let outcome = state
        .runtime
        .apply_lifecycle(&command, OffsetDateTime::now_utc())?;
    if matches!(
        request.action,
        ScenarioLifecycleAction::Stop | ScenarioLifecycleAction::Reset
    ) {
        publish_synthetic_termination(&state, &session_id, request.action).await?;
    }
    let event_type = match request.action {
        ScenarioLifecycleAction::Pause => "scenario.paused",
        ScenarioLifecycleAction::Stop => "scenario.stopped",
        ScenarioLifecycleAction::Reset => "scenario.reset",
        _ => "scenario.step.requested",
    };
    publish_gate_b_event(
        &state.config,
        state.broker.as_ref(),
        GateBEvent {
            event_type,
            status: "accepted",
            command_name: Some(lifecycle_action_name(request.action)),
            correlation_id: Some(&session_id),
            causation_id: Some(&command.operation_id),
            reason_code: None,
        },
    )
    .await;
    Ok(Json(serde_json::to_value(outcome).map_err(|_| {
        AppError::configuration("response-invalid")
    })?))
}

async fn publish_synthetic_termination(
    state: &DirectorState,
    demonstration_session_id: &str,
    action: ScenarioLifecycleAction,
) -> Result<(), AppError> {
    let broker = state
        .broker
        .as_ref()
        .ok_or(AppError::refused("interactive-broker-unavailable"))?;
    broker
        .publish(
            SYNTHETIC_TERMINATION_SUBJECT,
            &SyntheticTerminationEvent {
                contract_id: "I-005".to_owned(),
                contract_version: "1.0.0".to_owned(),
                operation_id: format!("terminate:{}", Uuid::new_v4()),
                demonstration_session_id: demonstration_session_id.to_owned(),
                reason: format!("scenario-{}", lifecycle_action_name(action)),
                requested_at: now_string()?,
            },
        )
        .await?;
    Ok(())
}

const fn lifecycle_action_name(action: ScenarioLifecycleAction) -> &'static str {
    match action {
        ScenarioLifecycleAction::Create => "create",
        ScenarioLifecycleAction::Prepare => "prepare",
        ScenarioLifecycleAction::Start => "start",
        ScenarioLifecycleAction::Pause => "pause",
        ScenarioLifecycleAction::Resume => "resume",
        ScenarioLifecycleAction::Complete => "complete",
        ScenarioLifecycleAction::Stop => "stop",
        ScenarioLifecycleAction::Reset => "reset",
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SyntheticSignInRequest {
    actor_id: String,
    surface_slot: String,
}

async fn request_synthetic_sign_in(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<SyntheticSignInRequest>,
) -> Result<Json<Value>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    let synthetic_role = match (request.actor_id.as_str(), request.surface_slot.as_str()) {
        ("synthetic-audience-user", "audience-display") => "portal-viewer",
        ("synthetic-reviewer", "reviewer-workbench") => "workbench-reviewer",
        _ => return Err(AppError::refused("synthetic-actor-or-surface-unsupported")),
    };
    let session = state.runtime.session(&session_id)?;
    if !matches!(
        session.state,
        ScenarioState::Preparing
            | ScenarioState::Ready
            | ScenarioState::Running
            | ScenarioState::Paused
    ) {
        return Err(AppError::refused("synthetic-sign-in-session-not-active"));
    }
    state
        .runtime
        .current_registration(&session_id, &request.surface_slot)?;
    let request_id = format!("grant-request:{}", Uuid::new_v4());
    let broker = state
        .broker
        .as_ref()
        .ok_or(AppError::refused("interactive-broker-unavailable"))?;
    broker
        .publish(
            GRANT_REQUEST_SUBJECT,
            &SyntheticGrantRequestEvent {
                contract_id: "I-004".to_owned(),
                contract_version: "1.0.0".to_owned(),
                request_id: request_id.clone(),
                requested_at: now_string()?,
                request: GrantRequest {
                    workload_id: "scenario-director".to_owned(),
                    actor_id: request.actor_id.clone(),
                    application_id: "presentation-gateway".to_owned(),
                    audience: "presentation-gateway".to_owned(),
                    surface_id: request.surface_slot,
                    demonstration_session_id: session_id,
                    roles: vec![synthetic_role.to_owned()],
                    purpose: "demonstrate-presentation".to_owned(),
                    synthetic_realm: format!("synthetic-realm-{}", state.config.environment_id),
                },
            },
        )
        .await?;
    Ok(Json(serde_json::json!({
        "status": "requested",
        "requestId": request_id,
        "actorId": request.actor_id,
        "applicationId": "presentation-gateway",
        "maturity": "in-development",
        "informationProfile": "synthetic-only"
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CueRequest {
    surface_slot: String,
    semantic_view: String,
    heading: String,
    message: String,
    expires_in_seconds: Option<u64>,
}

async fn issue_cue(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CueRequest>,
) -> Result<Json<PresentationCue>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    let session = state.runtime.session(&session_id)?;
    let registration = state
        .runtime
        .current_registration(&session_id, &request.surface_slot)?;
    let now = OffsetDateTime::now_utc();
    let expires_in = request.expires_in_seconds.unwrap_or(60);
    if !(5..=300).contains(&expires_in) {
        return Err(AppError::refused("cue-expiry-out-of-range"));
    }
    let cue_id = format!("cue:{}", Uuid::new_v4());
    let digest_seed = format!(
        "{}|{}|{}|{}|{}",
        session_id, session.revision, request.surface_slot, request.semantic_view, cue_id
    );
    let mut context = BTreeMap::new();
    context.insert("heading".to_owned(), request.heading);
    context.insert("message".to_owned(), request.message);
    context.insert(
        "syntheticReference".to_owned(),
        if request.surface_slot == "reviewer-workbench" {
            "engagement-harbour-support-review"
        } else {
            "welcome-record"
        }
        .to_owned(),
    );
    let stage_id = if request.semantic_view == "pres-intro" {
        "scenario-introduction"
    } else {
        "portal-orchestration"
    };
    let step_id = match request.semantic_view.as_str() {
        "pres-intro" => "show-introduction",
        "wb-engagement" => "open-engagement-context",
        "wb-source-intake" => "open-source-intake",
        _ => "unsupported-view-test",
    };
    let cue = PresentationCue {
        contract_id: "P-003".to_owned(),
        contract_version: "1.0.0".to_owned(),
        cue_id,
        cue_digest: format!("{:x}", Sha256::digest(digest_seed.as_bytes())),
        idempotency_key: format!("idempotency:{}", Uuid::new_v4()),
        session_id,
        session_revision: session.revision,
        surface_slot: request.surface_slot,
        registration_id: registration.registration_id,
        registration_revision: registration.registration_revision,
        connection_generation: registration.connection_generation,
        semantic_view: request.semantic_view,
        view_version: "1.0.0".to_owned(),
        context,
        issued_at: format_time(now)?,
        expires_at: format_time(
            now.checked_add(time::Duration::seconds(
                i64::try_from(expires_in)
                    .map_err(|_| AppError::refused("cue-expiry-out-of-range"))?,
            ))
            .ok_or(AppError::refused("cue-expiry-out-of-range"))?,
        )?,
        stage_id: stage_id.to_owned(),
        step_id: step_id.to_owned(),
    };
    match state.runtime.issue_cue(&cue, now) {
        Ok(issued) => {
            publish_gate_b_event(
                &state.config,
                state.broker.as_ref(),
                GateBEvent {
                    event_type: "view.requested",
                    status: "accepted",
                    command_name: Some(&cue.semantic_view),
                    correlation_id: Some(&cue.session_id),
                    causation_id: Some(&cue.cue_id),
                    reason_code: None,
                },
            )
            .await;
            Ok(Json(issued))
        }
        Err(error) => {
            publish_gate_b_event(
                &state.config,
                state.broker.as_ref(),
                GateBEvent {
                    event_type: "view.refused",
                    status: "refused",
                    command_name: Some(&cue.semantic_view),
                    correlation_id: Some(&cue.session_id),
                    causation_id: Some(&cue.cue_id),
                    reason_code: Some("semantic-view-unsupported-or-invalid"),
                },
            )
            .await;
            Err(error.into())
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AdvanceTimeRequest {
    operation: String,
    expected_revision: u64,
    logical_instant: Option<String>,
    advance_seconds: Option<u64>,
}

async fn advance_time(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AdvanceTimeRequest>,
) -> Result<Json<Value>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    let operation_id = format!("operation:{}", Uuid::new_v4());
    let now = OffsetDateTime::now_utc();
    let outcome = match request.operation.as_str() {
        "set" => state.runtime.set_initial_logical_time(
            &operation_id,
            &session_id,
            request.expected_revision,
            request
                .logical_instant
                .as_deref()
                .ok_or(AppError::refused("logical-instant-required"))?,
            now,
        )?,
        "advance" => state.runtime.advance_logical_time(
            &operation_id,
            &session_id,
            request.expected_revision,
            request
                .advance_seconds
                .ok_or(AppError::refused("advance-seconds-required"))?,
            now,
        )?,
        _ => return Err(AppError::refused("logical-time-operation-unsupported")),
    };
    Ok(Json(serde_json::to_value(outcome).map_err(|_| {
        AppError::configuration("response-invalid")
    })?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CueDelayRequest {
    expected_revision: u64,
    delay_milliseconds: u64,
}

async fn request_cue_delay(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CueDelayRequest>,
) -> Result<Json<CommandOutcome>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    let session = state.runtime.session(&session_id)?;
    if session.revision != request.expected_revision || session.state != ScenarioState::Running {
        return Err(AppError::refused("session-revision-stale"));
    }
    let control = ScenarioControlCommand {
        contract_id: "D-003".to_owned(),
        contract_version: "1.0.0".to_owned(),
        operation_id: format!("operation:{}", Uuid::new_v4()),
        session_id,
        kind: "fault".to_owned(),
        operation: "activate".to_owned(),
        target: Some("presentation-cue-delay".to_owned()),
        logical_instant: None,
        advance_seconds: None,
        delay_milliseconds: Some(request.delay_milliseconds),
        expected_revision: request.expected_revision,
        requested_at: now_string()?,
    };
    Ok(Json(coordinate_control(&state, &control).await?))
}

async fn coordinate_control(
    state: &DirectorState,
    command: &ScenarioControlCommand,
) -> Result<CommandOutcome, AppError> {
    if state.broker.is_none() {
        return Err(AppError::refused("interactive-broker-unavailable"));
    }
    state
        .runtime
        .request_control(command, OffsetDateTime::now_utc())?;
    for _ in 0..100 {
        if let Some(outcome) = state.runtime.control_outcome(&command.operation_id)? {
            return if outcome.status == OutcomeStatus::Accepted {
                Ok(outcome)
            } else {
                Err(AppError::refused("component-control-refused"))
            };
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(AppError::refused("component-control-uncertain"))
}

fn now_string() -> Result<String, AppError> {
    format_time(OffsetDateTime::now_utc())
}

fn format_time(value: OffsetDateTime) -> Result<String, AppError> {
    value
        .format(&Rfc3339)
        .map_err(|_| AppError::configuration("clock-unavailable"))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RegistrationRequest {
    session_id: String,
    surface_slot: String,
    surface_role: String,
}

async fn register_surface(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<RegistrationRequest>,
) -> Result<Json<PresentationRegistration>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    if !matches!(
        request.surface_role.as_str(),
        "audience-display" | "reviewer-workbench"
    ) {
        return Err(AppError::refused("surface-role-unsupported"));
    }
    let now = OffsetDateTime::now_utc();
    let supported_views = match request.surface_role.as_str() {
        "audience-display" => vec!["pres-intro".to_owned(), "pres-progress".to_owned()],
        "reviewer-workbench" => {
            vec!["wb-engagement".to_owned(), "wb-source-intake".to_owned()]
        }
        _ => unreachable!("surface role was validated above"),
    };
    let registration = PresentationRegistration {
        contract_id: "P-002".to_owned(),
        contract_version: "1.0.0".to_owned(),
        registration_id: format!("registration:{}", Uuid::new_v4()),
        session_id: request.session_id,
        surface_slot: request.surface_slot,
        surface_role: request.surface_role,
        manifest_id: "assurance-presentation-surface".to_owned(),
        manifest_version: "1.2.0".to_owned(),
        supported_views,
        binding_mode: "development-assurance".to_owned(),
        registration_revision: 1,
        connection_generation: 1,
        lease_expires_at: format_time(
            now.checked_add(time::Duration::minutes(20))
                .ok_or(AppError::configuration("clock-unavailable"))?,
        )?,
    };
    let outcome = state.runtime.register(&registration, now)?;
    let token = cookie_value(&headers, "PPL_APP_SESSION")
        .ok_or(AppError::unauthorised("application-session-required"))?;
    state.sessions.bind_surface(
        token,
        &outcome.registration.surface_slot,
        &outcome.registration.session_id,
        now,
    )?;
    publish_gate_b_event(
        &state.config,
        state.broker.as_ref(),
        GateBEvent {
            event_type: "surface.registered",
            status: "accepted",
            command_name: Some(&outcome.registration.surface_slot),
            correlation_id: Some(&outcome.registration.session_id),
            causation_id: Some(&outcome.registration.registration_id),
            reason_code: None,
        },
    )
    .await;
    Ok(Json(outcome.registration))
}

async fn cue_events(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    authorise_read(&state.config, &state.sessions, &headers)?;
    let receiver = state.cue_channel.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|item| async move {
        match item {
            Ok(cue) => serde_json::to_string(&cue)
                .ok()
                .map(|json| Ok(Event::default().event("presentation-cue").data(json))),
            Err(_) => None,
        }
    });
    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

async fn gateway_session_context(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let token = cookie_value(&headers, "PPL_APP_SESSION")
        .ok_or(AppError::unauthorised("application-session-required"))?;
    let mapping_version = current_mapping_version(&state.config)?;
    let authorised = state.sessions.authorise_read(
        token,
        "surface-operator",
        &mapping_version,
        OffsetDateTime::now_utc(),
    )?;
    let registration = match (
        authorised.bound_demonstration_session_id.as_deref(),
        authorised.bound_surface_id.as_deref(),
    ) {
        (Some(session_id), Some(surface_id)) => state
            .runtime
            .current_registration(session_id, surface_id)
            .ok(),
        _ => None,
    };
    Ok(Json(
        if let Some(synthetic) = authorised.synthetic_identity {
            serde_json::json!({
                "externalPrincipalId": authorised.external_identity.principal_id,
                "syntheticStatus": "established",
                "syntheticActorId": synthetic.actor_id,
                "syntheticRoles": synthetic.roles,
                "applicationId": synthetic.application_id,
                "surfaceId": synthetic.surface_id,
                "demonstrationSessionId": synthetic.demonstration_session_id,
                "maximumValidUntil": synthetic.maximum_valid_until,
                "registration": registration,
                "environmentId": state.config.environment_id,
                "trustProfile": trust_profile_name(state.config.profile),
                "maturity": "in-development",
                "informationProfile": "synthetic-only"
            })
        } else {
            serde_json::json!({
                "externalPrincipalId": authorised.external_identity.principal_id,
                "syntheticStatus": "not-established",
                "registration": registration,
                "environmentId": state.config.environment_id,
                "trustProfile": trust_profile_name(state.config.profile),
                "maturity": "in-development",
                "informationProfile": "synthetic-only"
            })
        },
    ))
}

async fn record_outcome(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(outcome): Json<PresentationCueOutcome>,
) -> Result<Json<PresentationCueOutcome>, AppError> {
    authorise(&state.config, &state.sessions, &headers)?;
    if outcome.surface_slot == "reviewer-workbench" {
        require_synthetic_surface(
            &state,
            &headers,
            Some(&outcome.session_id),
            Some(&outcome.surface_slot),
        )?;
    } else if outcome.surface_slot != "audience-display" {
        return Err(AppError::unauthorised("surface-outcome-refused"));
    }
    let recorded = state
        .runtime
        .record_outcome(&outcome, OffsetDateTime::now_utc())?;
    let (event_type, status) = if recorded.result == PresentationOutcomeResult::Applied {
        ("view.applied", "accepted")
    } else {
        ("view.refused", "refused")
    };
    publish_gate_b_event(
        &state.config,
        state.broker.as_ref(),
        GateBEvent {
            event_type,
            status,
            command_name: Some(&recorded.semantic_view),
            correlation_id: Some(&recorded.session_id),
            causation_id: Some(&recorded.cue_id),
            reason_code: recorded.reason.as_deref(),
        },
    )
    .await;
    Ok(Json(recorded))
}

fn require_synthetic_surface(
    state: &GatewayState,
    headers: &HeaderMap,
    demonstration_session_id: Option<&str>,
    surface_id: Option<&str>,
) -> Result<(), AppError> {
    let token = cookie_value(headers, "PPL_APP_SESSION")
        .ok_or(AppError::unauthorised("application-session-required"))?;
    let mapping_version = current_mapping_version(&state.config)?;
    let authorised = state.sessions.authorise_read(
        token,
        "surface-operator",
        &mapping_version,
        OffsetDateTime::now_utc(),
    )?;
    let synthetic = authorised.synthetic_identity.ok_or(AppError::unauthorised(
        "synthetic-application-session-required",
    ))?;
    if synthetic.status != SyntheticSessionStatus::Established {
        return Err(AppError::unauthorised(
            "synthetic-application-session-required",
        ));
    }
    if demonstration_session_id.is_some_and(|value| value != synthetic.demonstration_session_id)
        || surface_id.is_some_and(|value| value != synthetic.surface_id)
    {
        return Err(AppError::unauthorised("synthetic-surface-binding-refused"));
    }
    Ok(())
}

const fn trust_profile_name(profile: RuntimeProfile) -> &'static str {
    match profile {
        RuntimeProfile::NativeDevelopment
        | RuntimeProfile::LocalContainers
        | RuntimeProfile::Minikube => "environment-local-synthetic-root",
        RuntimeProfile::PrivateHostedSmoke | RuntimeProfile::ManagedHosted => "managed-real-root",
    }
}

async fn director_consumer(state: DirectorState, broker: Broker) {
    let consumer = match broker.consumer().await {
        Ok(consumer) => consumer,
        Err(error) => {
            error!(reason = %error, "director consumer unavailable");
            return;
        }
    };
    let mut messages = match consumer.messages().await {
        Ok(messages) => messages,
        Err(error) => {
            error!(reason = %error, "director message stream unavailable");
            return;
        }
    };
    while let Some(message) = messages.next().await {
        let Ok(message) = message else {
            warn!("director message delivery failed");
            continue;
        };
        let result = if message.subject.as_str() == REGISTRATION_SUBJECT {
            serde_json::from_slice::<PresentationRegistration>(&message.payload)
                .map_err(|_| ())
                .and_then(|registration| {
                    state
                        .runtime
                        .observe_registration(&registration, OffsetDateTime::now_utc())
                        .map_err(|_| ())
                })
        } else if message.subject.as_str() == OUTCOME_SUBJECT {
            serde_json::from_slice::<PresentationCueOutcome>(&message.payload)
                .map_err(|_| ())
                .and_then(|outcome| {
                    state
                        .runtime
                        .observe_presentation_outcome(&outcome)
                        .map(|_| ())
                        .map_err(|_| ())
                })
        } else if message.subject.as_str() == CONTROL_OUTCOME_SUBJECT {
            serde_json::from_slice::<CommandOutcome>(&message.payload)
                .map_err(|_| ())
                .and_then(|outcome| {
                    state
                        .runtime
                        .observe_control_outcome(&outcome)
                        .map_err(|_| ())
                })
        } else if message.subject.as_str() == IDENTITY_OUTCOME_SUBJECT {
            serde_json::from_slice::<SyntheticIdentityOutcomeEvent>(&message.payload)
                .map(|outcome| {
                    info!(
                        request_id = %outcome.request_id,
                        status = %outcome.status,
                        code = %outcome.code,
                        "synthetic identity operation concluded"
                    );
                })
                .map_err(|_| ())
        } else {
            Err(())
        };
        if result.is_ok() {
            if let Err(error) = message.ack().await {
                warn!(reason = %error, "director message acknowledgement failed");
            }
        } else {
            warn!(subject = %message.subject, "director refused broker message");
        }
    }
}

async fn gateway_consumer(state: GatewayState, broker: Broker) {
    let consumer = match broker.consumer().await {
        Ok(consumer) => consumer,
        Err(error) => {
            error!(reason = %error, "presentation consumer unavailable");
            return;
        }
    };
    let mut messages = match consumer.messages().await {
        Ok(messages) => messages,
        Err(error) => {
            error!(reason = %error, "presentation message stream unavailable");
            return;
        }
    };
    while let Some(message) = messages.next().await {
        let Ok(message) = message else {
            warn!("presentation message delivery failed");
            continue;
        };
        let result = if message.subject.as_str() == CUE_SUBJECT {
            match serde_json::from_slice::<PresentationCue>(&message.payload) {
                Ok(cue) => match state.runtime.accept_cue(&cue, OffsetDateTime::now_utc()) {
                    Ok(delivery) => {
                        if delivery.delay_milliseconds > 0 {
                            tokio::time::sleep(Duration::from_millis(delivery.delay_milliseconds))
                                .await;
                        }
                        if delivery.status == "accepted" {
                            let _ = state.cue_channel.send(delivery.cue);
                        }
                        Ok(())
                    }
                    Err(error) => {
                        warn!(reason = %error, "presentation cue refused");
                        Err(())
                    }
                },
                Err(_) => Err(()),
            }
        } else if message.subject.as_str() == CONTROL_SUBJECT {
            match serde_json::from_slice::<ScenarioControlCommand>(&message.payload) {
                Ok(control) => state
                    .runtime
                    .apply_control(&control, OffsetDateTime::now_utc())
                    .map(|_| ())
                    .map_err(|error| {
                        warn!(reason = %error, "presentation control failed");
                    }),
                Err(_) => Err(()),
            }
        } else if message.subject.as_str() == SYNTHETIC_GRANT_SUBJECT {
            process_synthetic_grant(&state, &broker, &message.payload).await
        } else if message.subject.as_str() == SYNTHETIC_TERMINATION_SUBJECT {
            process_synthetic_termination(&state, &message.payload).await
        } else {
            Err(())
        };
        if result.is_ok()
            && let Err(error) = message.ack().await
        {
            warn!(reason = %error, "presentation message acknowledgement failed");
        }
    }
}

async fn process_synthetic_grant(
    state: &GatewayState,
    broker: &Broker,
    payload: &[u8],
) -> Result<(), ()> {
    let delivery =
        serde_json::from_slice::<SyntheticGrantDeliveryEvent>(payload).map_err(|_| ())?;
    let store = gateway_grant_store(state).map_err(|error| {
        warn!(reason = error.code, "presentation trust bundle unavailable");
    })?;
    let now = OffsetDateTime::now_utc();
    let outcome = store
        .establish(
            &delivery.grant,
            &delivery.grant.claims.surface_id,
            &delivery.grant.claims.demonstration_session_id,
            now,
        )
        .map_err(|error| {
            warn!(reason = %error, "synthetic grant establishment failed safely");
        })?;
    if outcome.status == SyntheticSessionStatus::Established {
        let mut binding = outcome.clone();
        binding.original_outcome_id = None;
        state
            .sessions
            .bind_synthetic_to_surface(&binding, now)
            .map_err(|error| {
                warn!(reason = %error, "synthetic session had no authorised surface binding");
            })?;
    }
    broker
        .publish(
            IDENTITY_OUTCOME_SUBJECT,
            &SyntheticIdentityOutcomeEvent {
                contract_id: "I-005".to_owned(),
                contract_version: "1.0.0".to_owned(),
                request_id: delivery.request_id.clone(),
                status: match outcome.status {
                    SyntheticSessionStatus::Established => "established",
                    SyntheticSessionStatus::Expired => "expired",
                    _ => "refused",
                }
                .to_owned(),
                code: outcome
                    .reason_code
                    .clone()
                    .unwrap_or_else(|| "synthetic-session-established".to_owned()),
                occurred_at: now_string().map_err(|_| ())?,
                synthetic_session: Some(outcome.clone()),
            },
        )
        .await
        .map_err(|error| {
            warn!(reason = %error, "synthetic outcome publication failed");
        })?;
    publish_gate_b_event(
        &state.config,
        Some(broker),
        GateBEvent {
            event_type: if outcome.status == SyntheticSessionStatus::Established {
                "synthetic-session.established"
            } else {
                "synthetic-session.refused"
            },
            status: if outcome.status == SyntheticSessionStatus::Established {
                "accepted"
            } else {
                "refused"
            },
            command_name: Some("establish-synthetic-application-session"),
            correlation_id: Some(&delivery.grant.claims.demonstration_session_id),
            causation_id: Some(&delivery.request_id),
            reason_code: outcome.reason_code.as_deref(),
        },
    )
    .await;
    Ok(())
}

async fn process_synthetic_termination(state: &GatewayState, payload: &[u8]) -> Result<(), ()> {
    let termination =
        serde_json::from_slice::<SyntheticTerminationEvent>(payload).map_err(|_| ())?;
    let now = OffsetDateTime::now_utc();
    state
        .sessions
        .clear_synthetic_bindings(&termination.demonstration_session_id, now)
        .map_err(|error| {
            warn!(reason = %error, "application synthetic binding termination failed");
        })?;
    if let Ok(store) = gateway_grant_store(state) {
        store
            .terminate_demonstration_session(
                &termination.demonstration_session_id,
                &termination.reason,
                now,
            )
            .map_err(|error| {
                warn!(reason = %error, "synthetic establishment termination failed");
            })?;
    }
    publish_gate_b_event(
        &state.config,
        state.broker.as_ref(),
        GateBEvent {
            event_type: "synthetic-session.terminated",
            status: "accepted",
            command_name: Some("terminate-synthetic-application-session"),
            correlation_id: Some(&termination.demonstration_session_id),
            causation_id: Some(&termination.operation_id),
            reason_code: Some(&termination.reason),
        },
    )
    .await;
    Ok(())
}

fn gateway_grant_store(state: &GatewayState) -> Result<&ApplicationGrantStore, AppError> {
    if let Some(store) = state.grants.get() {
        return Ok(store);
    }
    let trust = load_trust_bundle(&state.config.trust_bundle_path)?;
    let store = ApplicationGrantStore::open(
        state
            .config
            .security_state_path
            .with_extension("grants.sqlite"),
        trust,
        "presentation-gateway",
        "presentation-gateway",
    )?;
    let _ = state.grants.set(store);
    state
        .grants
        .get()
        .ok_or(AppError::configuration("synthetic-establishment-failed"))
}

async fn identity_consumer(state: IdentityState, broker: Broker) {
    let consumer = match broker.consumer().await {
        Ok(consumer) => consumer,
        Err(error) => {
            error!(reason = %error, "identity consumer unavailable");
            return;
        }
    };
    let mut messages = match consumer.messages().await {
        Ok(messages) => messages,
        Err(error) => {
            error!(reason = %error, "identity message stream unavailable");
            return;
        }
    };
    while let Some(message) = messages.next().await {
        let Ok(message) = message else {
            warn!("identity message delivery failed");
            continue;
        };
        if message.subject.as_str() != GRANT_REQUEST_SUBJECT {
            continue;
        }
        let Ok(event) = serde_json::from_slice::<SyntheticGrantRequestEvent>(&message.payload)
        else {
            warn!("identity broker refused malformed request");
            continue;
        };
        let now = OffsetDateTime::now_utc();
        let issued = state
            .identity
            .issue_grant(&event.request_id, &event.request, now)
            .await;
        let published = match issued {
            Ok(grant) => {
                let delivery = SyntheticGrantDeliveryEvent {
                    contract_id: "I-004".to_owned(),
                    contract_version: "1.0.0".to_owned(),
                    request_id: event.request_id.clone(),
                    delivered_at: match now_string() {
                        Ok(value) => value,
                        Err(_) => continue,
                    },
                    grant,
                };
                broker
                    .publish(SYNTHETIC_GRANT_SUBJECT, &delivery)
                    .await
                    .is_ok()
            }
            Err(error) => {
                warn!(
                    reason_code = error.code,
                    request_id = %event.request_id,
                    "synthetic grant refused"
                );
                let outcome = SyntheticIdentityOutcomeEvent {
                    contract_id: "I-004".to_owned(),
                    contract_version: "1.0.0".to_owned(),
                    request_id: event.request_id,
                    status: "refused".to_owned(),
                    code: "synthetic-grant-refused".to_owned(),
                    occurred_at: match now_string() {
                        Ok(value) => value,
                        Err(_) => continue,
                    },
                    synthetic_session: None,
                };
                broker
                    .publish(IDENTITY_OUTCOME_SUBJECT, &outcome)
                    .await
                    .is_ok()
            }
        };
        if published && let Err(error) = message.ack().await {
            warn!(reason = %error, "identity message acknowledgement failed");
        }
    }
}

async fn director_outbox(state: DirectorState, broker: Broker) {
    loop {
        match state.runtime.pending_outbox(32) {
            Ok(records) => {
                for record in records {
                    let subject = if record.event_type == "ppl.presentation.cue.requested" {
                        CUE_SUBJECT
                    } else if record.event_type == "ppl.presentation.control.requested" {
                        CONTROL_SUBJECT
                    } else {
                        DIRECTOR_EVENT_SUBJECT
                    };
                    if broker.publish(subject, &record.payload).await.is_ok()
                        && let Err(error) = state
                            .runtime
                            .mark_outbox_published(record.sequence, OffsetDateTime::now_utc())
                    {
                        warn!(reason = %error, "director outbox marker failed");
                    }
                }
            }
            Err(error) => warn!(reason = %error, "director outbox unavailable"),
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

async fn gateway_outbox(state: GatewayState, broker: Broker) {
    loop {
        match state.runtime.pending_outbox(32) {
            Ok(records) => {
                for record in records {
                    let published = if record.event_type == "ppl.presentation.surface.registered" {
                        if let Ok(outcome) = serde_json::from_value::<ppl_ctl_02::RegistrationOutcome>(
                            record.payload.clone(),
                        ) {
                            broker
                                .publish(REGISTRATION_SUBJECT, &outcome.registration)
                                .await
                                .is_ok()
                        } else {
                            false
                        }
                    } else if record.event_type == "ppl.presentation.cue.concluded" {
                        broker
                            .publish(OUTCOME_SUBJECT, &record.payload)
                            .await
                            .is_ok()
                    } else if record.event_type == "ppl.presentation.control.concluded" {
                        broker
                            .publish(CONTROL_OUTCOME_SUBJECT, &record.payload)
                            .await
                            .is_ok()
                    } else {
                        false
                    };
                    if published
                        && let Err(error) = state
                            .runtime
                            .mark_outbox_published(record.sequence, OffsetDateTime::now_utc())
                    {
                        warn!(reason = %error, "presentation outbox marker failed");
                    }
                }
            }
            Err(error) => warn!(reason = %error, "presentation outbox unavailable"),
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
