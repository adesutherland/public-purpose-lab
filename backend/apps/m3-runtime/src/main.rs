//! M3.3 executable adapter for separate Director and Presentation workloads.

use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::{get, post},
};
use futures_util::StreamExt;
use ppl_contracts::{
    CommandOutcome, OutcomeStatus, PresentationCapabilityManifest, PresentationCue,
    PresentationCueOutcome, PresentationRegistration, ScenarioControlCommand,
    ScenarioLifecycleAction, ScenarioLifecycleCommand, ScenarioState,
};
use ppl_ctl_01::{DirectorError, DirectorRuntime};
use ppl_ctl_02::{PresentationError, PresentationRuntime};
use ppl_int_01::nats::{
    Broker, BrokerConfig, BrokerError, CONTROL_OUTCOME_SUBJECT, CONTROL_SUBJECT, CUE_SUBJECT,
    DIRECTOR_EVENT_SUBJECT, OUTCOME_SUBJECT, REGISTRATION_SUBJECT, WorkloadMode,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::{Mutex, broadcast};
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use uuid::Uuid;

const PACKAGE_ID: &str = "presentation-control-assurance";
const PACKAGE_VERSION: &str = "1.0.0";
const MANIFEST_JSON: &str =
    include_str!("../../../../contracts/presentation/examples/p-001-assurance-surface.json");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeProfile {
    NativeDevelopment,
    LocalContainers,
    Minikube,
    PrivateHostedSmoke,
}

impl RuntimeProfile {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "native-development" => Ok(Self::NativeDevelopment),
            "local-containers" => Ok(Self::LocalContainers),
            "minikube" => Ok(Self::Minikube),
            "private-hosted-smoke" => Ok(Self::PrivateHostedSmoke),
            _ => Err(AppError::configuration("runtime-profile-invalid")),
        }
    }

    const fn interactive(self) -> bool {
        !matches!(self, Self::PrivateHostedSmoke)
    }

    const fn name(self) -> &'static str {
        match self {
            Self::NativeDevelopment => "native-development",
            Self::LocalContainers => "local-containers",
            Self::Minikube => "minikube",
            Self::PrivateHostedSmoke => "private-hosted-smoke",
        }
    }
}

#[derive(Clone, Debug)]
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
}

#[derive(Clone)]
struct LocalSessions {
    sessions: Arc<Mutex<HashMap<String, Instant>>>,
}

impl LocalSessions {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn issue(&self) -> String {
        let token = Uuid::new_v4().to_string();
        self.sessions
            .lock()
            .await
            .insert(token.clone(), Instant::now() + Duration::from_mins(30));
        token
    }

    async fn valid(&self, token: &str) -> bool {
        let mut sessions = self.sessions.lock().await;
        let now = Instant::now();
        sessions.retain(|_, expiry| *expiry > now);
        sessions.get(token).is_some_and(|expiry| *expiry > now)
    }
}

#[derive(Clone)]
struct DirectorState {
    config: AppConfig,
    runtime: DirectorRuntime,
    broker: Option<Broker>,
    sessions: LocalSessions,
}

#[derive(Clone)]
struct GatewayState {
    config: AppConfig,
    runtime: PresentationRuntime,
    broker: Option<Broker>,
    manifest_digest: String,
    sessions: LocalSessions,
    cue_channel: broadcast::Sender<PresentationCue>,
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
        error!(code = error.code, "M3.3 runtime stopped safely");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let config = load_config()?;
    match config.mode {
        WorkloadMode::ScenarioDirector => run_director(config).await,
        WorkloadMode::PresentationGateway => run_gateway(config).await,
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
    let state = DirectorState {
        config: config.clone(),
        runtime,
        broker,
        sessions: LocalSessions::new(),
    };
    if let Some(broker) = state.broker.clone() {
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
    let state = GatewayState {
        config: config.clone(),
        runtime,
        broker,
        manifest_digest,
        sessions: LocalSessions::new(),
        cue_channel,
    };
    if let Some(broker) = state.broker.clone() {
        tokio::spawn(gateway_consumer(state.clone(), broker.clone()));
        tokio::spawn(gateway_outbox(state.clone(), broker));
    }
    let router = common_layers(gateway_router(state), &config);
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
        "M3.3 runtime listening"
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

fn load_config() -> Result<AppConfig, AppError> {
    let mode = match required_env("PPL_RUNTIME_MODE")?.as_str() {
        "scenario-director" => WorkloadMode::ScenarioDirector,
        "presentation-gateway" => WorkloadMode::PresentationGateway,
        _ => return Err(AppError::configuration("runtime-mode-invalid")),
    };
    let profile = RuntimeProfile::parse(&required_env("PPL_RUNTIME_PROFILE")?)?;
    let default_port = match mode {
        WorkloadMode::ScenarioDirector => 18_081,
        WorkloadMode::PresentationGateway => 18_082,
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
    };
    let static_name = match mode {
        WorkloadMode::ScenarioDirector => "director",
        WorkloadMode::PresentationGateway => "presentation",
    };
    let broker = if profile == RuntimeProfile::PrivateHostedSmoke {
        None
    } else {
        Some(load_broker_config(mode)?)
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
        allowed_origin: env::var("PPL_ALLOWED_ORIGIN")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{default_port}")),
        broker,
    })
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
        .with_state(state)
}

fn gateway_router(state: GatewayState) -> Router {
    Router::new()
        .route("/health/live", get(gateway_liveness))
        .route("/health/ready", get(gateway_readiness))
        .route("/health/contracts", get(gateway_contracts))
        .route("/api/v1/development-session", post(gateway_login))
        .route("/api/v1/registrations", post(register_surface))
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

async fn director_contracts(State(state): State<DirectorState>) -> Result<Json<Value>, AppError> {
    let package = state
        .runtime
        .package_admission(PACKAGE_ID, PACKAGE_VERSION)?;
    Ok(Json(serde_json::json!({
        "selfTest": "passed",
        "contracts": ["D-001", "D-002", "D-003", "D-004", "P-001", "P-002", "P-003", "P-004"],
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
        "manifestId": "assurance-presentation-surface",
        "manifestVersion": "1.0.0",
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
    readiness_response(&state.config, state.broker.is_some())
}

async fn gateway_readiness(State(state): State<GatewayState>) -> Response {
    readiness_response(&state.config, state.broker.is_some())
}

fn readiness_response(config: &AppConfig, broker_ready: bool) -> Response {
    let interactive_ready = config.profile.interactive() && broker_ready;
    let status = if interactive_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(serde_json::json!({
            "softwareStatus": "healthy",
            "interactiveReady": interactive_ready,
            "trustProfile": if config.profile == RuntimeProfile::PrivateHostedSmoke { "managed" } else { "development-assurance" },
            "reasonCode": if interactive_ready { Value::Null } else { Value::String("managed-trust-binding-absent".to_owned()) },
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
        "synthetic-presenter",
    )
    .await
}

async fn gateway_login(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    development_login(
        &state.config,
        &state.sessions,
        &headers,
        "synthetic-surface-operator",
    )
    .await
}

async fn development_login(
    config: &AppConfig,
    sessions: &LocalSessions,
    headers: &HeaderMap,
    actor: &str,
) -> Result<Response, AppError> {
    if !config.profile.interactive() {
        return Err(AppError::unauthorised(
            "development-assurance-adapter-unavailable",
        ));
    }
    require_origin(config, headers)?;
    let token = sessions.issue().await;
    let mut response = Json(serde_json::json!({
        "status": "established",
        "actor": actor,
        "expiresInSeconds": 1800,
        "maturity": "in-development",
        "informationProfile": "synthetic-only",
        "warning": "Synthetic development assurance only"
    }))
    .into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "PPL_DEV_SESSION={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=1800"
        ))
        .map_err(|_| AppError::configuration("session-cookie-invalid"))?,
    );
    Ok(response)
}

async fn authorise(
    config: &AppConfig,
    sessions: &LocalSessions,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    if !config.profile.interactive() {
        return Err(AppError::unauthorised(
            "development-assurance-adapter-unavailable",
        ));
    }
    require_origin(config, headers)?;
    let token = cookie_value(headers, "PPL_DEV_SESSION")
        .ok_or(AppError::unauthorised("development-session-required"))?;
    if sessions.valid(token).await {
        Ok(())
    } else {
        Err(AppError::unauthorised("development-session-expired"))
    }
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

async fn authorise_read(
    config: &AppConfig,
    sessions: &LocalSessions,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    if !config.profile.interactive() {
        return Err(AppError::unauthorised(
            "development-assurance-adapter-unavailable",
        ));
    }
    let token = cookie_value(headers, "PPL_DEV_SESSION")
        .ok_or(AppError::unauthorised("development-session-required"))?;
    if sessions.valid(token).await {
        Ok(())
    } else {
        Err(AppError::unauthorised("development-session-expired"))
    }
}

async fn director_status(
    State(state): State<DirectorState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    authorise_read(&state.config, &state.sessions, &headers).await?;
    let session = state.runtime.session(&session_id)?;
    let registration = state
        .runtime
        .current_registration(&session_id, "audience-display")
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
    authorise(&state.config, &state.sessions, &headers).await?;
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("session:{}", Uuid::new_v4()));
    let command = ScenarioLifecycleCommand {
        contract_id: "D-002".to_owned(),
        contract_version: "1.0.0".to_owned(),
        operation_id: format!("operation:{}", Uuid::new_v4()),
        session_id,
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
    authorise(&state.config, &state.sessions, &headers).await?;
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
        session_id,
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
    Ok(Json(serde_json::to_value(outcome).map_err(|_| {
        AppError::configuration("response-invalid")
    })?))
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
    authorise(&state.config, &state.sessions, &headers).await?;
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
    context.insert("syntheticReference".to_owned(), "welcome-record".to_owned());
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
        stage_id: "assurance-path".to_owned(),
        step_id: "show-welcome".to_owned(),
    };
    Ok(Json(state.runtime.issue_cue(&cue, now)?))
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
    authorise(&state.config, &state.sessions, &headers).await?;
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
    authorise(&state.config, &state.sessions, &headers).await?;
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
    authorise(&state.config, &state.sessions, &headers).await?;
    if !matches!(
        request.surface_role.as_str(),
        "audience-display" | "reviewer-workbench"
    ) {
        return Err(AppError::refused("surface-role-unsupported"));
    }
    let now = OffsetDateTime::now_utc();
    let registration = PresentationRegistration {
        contract_id: "P-002".to_owned(),
        contract_version: "1.0.0".to_owned(),
        registration_id: format!("registration:{}", Uuid::new_v4()),
        session_id: request.session_id,
        surface_slot: request.surface_slot,
        surface_role: request.surface_role,
        manifest_id: "assurance-presentation-surface".to_owned(),
        manifest_version: "1.0.0".to_owned(),
        supported_views: vec!["assurance-welcome".to_owned()],
        binding_mode: "development-assurance".to_owned(),
        registration_revision: 1,
        connection_generation: 1,
        lease_expires_at: format_time(
            now.checked_add(time::Duration::minutes(20))
                .ok_or(AppError::configuration("clock-unavailable"))?,
        )?,
    };
    let outcome = state.runtime.register(&registration, now)?;
    Ok(Json(outcome.registration))
}

async fn cue_events(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    authorise_read(&state.config, &state.sessions, &headers).await?;
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

async fn record_outcome(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(outcome): Json<PresentationCueOutcome>,
) -> Result<Json<PresentationCueOutcome>, AppError> {
    authorise(&state.config, &state.sessions, &headers).await?;
    Ok(Json(
        state
            .runtime
            .record_outcome(&outcome, OffsetDateTime::now_utc())?,
    ))
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
        if message.subject.as_str() != CUE_SUBJECT && message.subject.as_str() != CONTROL_SUBJECT {
            continue;
        }
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
        } else {
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
        };
        if result.is_ok()
            && let Err(error) = message.ack().await
        {
            warn!(reason = %error, "presentation message acknowledgement failed");
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
