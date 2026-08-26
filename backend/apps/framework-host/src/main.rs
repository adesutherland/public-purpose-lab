//! Local M1/M2 framework-host adapters and process boundary.

use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use ppl_contracts::{
    AssertionStatus, AssertionType, AuthoritativeAssertion, DemonstrationSignInGrant,
    EnvironmentClass, InformationProfile, InteractionEnvelope, OutcomeStatus, TrustProfile,
};
use ppl_iam_01::{
    DeclaredEnvironment, DemonstrationConfiguration, GrantRequest, IamRuntime, IdentityState,
    SyntheticActorRegistration, WorkloadRegistration,
};
use ppl_int_01::{InteractionRuntime, InteractionState, RuntimeConfig, parse_time};
use serde::de::DeserializeOwned;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

const USAGE: &str = "usage: ppl-framework-host <describe|manifest|healthcheck|process|serve|iam-bootstrap|iam-configure-demo|iam-health|iam-workload|iam-issue-grant|iam-establish|iam-terminate|iam-revoke> [--state-dir PATH] [--environment-id ID] [--now RFC3339] [--output PATH] [--workload-id ID] [--session-reference REF] [--reason CODE] [INPUT.json]";
const MAX_INPUT_BYTES: u64 = 1_048_576;

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    state_dir: Option<PathBuf>,
    environment_id: Option<String>,
    now: Option<String>,
    output: Option<PathBuf>,
    workload_id: Option<String>,
    session_reference: Option<String>,
    reason: Option<String>,
    input: Option<PathBuf>,
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage_error();
    };
    let options = match parse_options(args) {
        Ok(options) => options,
        Err(code) => return safe_error(code, ExitCode::from(2)),
    };

    match command.as_str() {
        "describe" if options == Options::default() => describe(),
        "manifest" if only_base_options(&options) => manifest(&options),
        "healthcheck" if only_base_options(&options) && options.now.is_none() => {
            healthcheck(&options)
        }
        "process" => process(&options),
        "serve" if only_base_options(&options) && options.now.is_none() => serve(&options),
        "iam-bootstrap" if no_action_options(&options) => iam_bootstrap(&options),
        "iam-configure-demo" if no_action_options(&options) => iam_configure_demo(&options),
        "iam-health" if no_action_options(&options) && options.now.is_none() => {
            iam_health(&options)
        }
        "iam-workload"
            if options.workload_id.is_some()
                && options.input.is_none()
                && options.output.is_none()
                && options.session_reference.is_none()
                && options.reason.is_none() =>
        {
            iam_workload(&options)
        }
        "iam-issue-grant"
            if options.input.is_some()
                && options.output.is_some()
                && options.workload_id.is_none()
                && options.session_reference.is_none()
                && options.reason.is_none() =>
        {
            iam_issue_grant(&options)
        }
        "iam-establish"
            if options.input.is_some()
                && options.output.is_none()
                && options.workload_id.is_none()
                && options.session_reference.is_none()
                && options.reason.is_none() =>
        {
            iam_establish(&options)
        }
        "iam-terminate"
            if options.session_reference.is_some()
                && options.reason.is_some()
                && options.input.is_none()
                && options.output.is_none()
                && options.workload_id.is_none() =>
        {
            iam_terminate(&options)
        }
        "iam-revoke"
            if options.reason.is_some()
                && options.input.is_none()
                && options.output.is_none()
                && options.workload_id.is_none()
                && options.session_reference.is_none() =>
        {
            iam_revoke(&options)
        }
        _ => usage_error(),
    }
}

fn describe() -> ExitCode {
    println!("framework-host|in-development|local-assurance-and-local-synthetic-only");
    println!("{}", ppl_aut_01::descriptor());
    println!("{}", ppl_iam_01::descriptor());
    println!("{}", ppl_int_01::descriptor());
    ExitCode::SUCCESS
}

fn manifest(options: &Options) -> ExitCode {
    let Some(runtime) = interaction_runtime(options, false) else {
        return safe_error("state_configuration_invalid", ExitCode::from(2));
    };
    let Ok(generated_at) = ppl_int_01::now_utc() else {
        return safe_error("clock_unavailable", ExitCode::from(5));
    };
    write_json(&runtime.capability_manifest(&generated_at))
}

fn healthcheck(options: &Options) -> ExitCode {
    if resolve_state_dir(options).is_none() {
        println!(
            "{{\"softwareStatus\":\"healthy\",\"interactionState\":\"not-configured\",\"identityState\":\"not-configured\",\"maturity\":\"in-development\"}}"
        );
        return ExitCode::SUCCESS;
    }
    let Some(interaction) = interaction_runtime(options, true) else {
        return safe_error("state_configuration_invalid", ExitCode::from(2));
    };
    let Some(identity) = iam_runtime(options) else {
        return safe_error("state_configuration_invalid", ExitCode::from(2));
    };
    let interaction_health = interaction.health();
    let identity_health = identity.health(declared_environment());
    let identity_required = env_flag("PPL_REQUIRE_IDENTITY");
    let ready = interaction_health.interaction_state == InteractionState::Ready
        && (!identity_required || identity_health.identity_state == IdentityState::Ready);
    let report = serde_json::json!({
        "softwareStatus": "healthy",
        "interactionState": interaction_health.interaction_state,
        "journalRecords": interaction_health.journal_records,
        "identityState": identity_health.identity_state,
        "identityRequired": identity_required,
        "maturity": "in-development",
        "environmentId": identity_health.environment_id,
        "environmentClass": identity_health.environment_class,
        "informationProfile": identity_health.information_profile,
        "activeTrustProfile": identity_health.active_trust_profile,
        "requiredTrustProfile": identity_health.required_trust_profile,
        "trustDomain": identity_health.trust_domain,
        "trustEpoch": identity_health.trust_epoch,
        "signerFingerprint": identity_health.signer_fingerprint,
        "keyCustodyClass": identity_health.key_custody_class,
        "recoveryProfile": identity_health.recovery_profile,
        "prominentWarning": identity_health.prominent_warning,
        "identityReasonCode": identity_health.reason_code,
        "interactionReasonCode": interaction_health.reason_code,
    });
    if write_json(&report) != ExitCode::SUCCESS {
        return ExitCode::from(5);
    }
    if ready {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

fn process(options: &Options) -> ExitCode {
    let Some(input) = options.input.as_deref() else {
        return safe_error("input_required", ExitCode::from(2));
    };
    let Some(runtime) = interaction_runtime(options, true) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Ok(envelope) = read_json_input::<InteractionEnvelope>(input) else {
        return safe_error("interaction_invalid", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    let Ok(outcome) = runtime.process(&envelope, now) else {
        return safe_error("interaction_state_failure", ExitCode::from(5));
    };
    let exit = match outcome.status {
        OutcomeStatus::Accepted | OutcomeStatus::Duplicate => ExitCode::SUCCESS,
        OutcomeStatus::Refused | OutcomeStatus::Expired => ExitCode::from(4),
        OutcomeStatus::Failed => ExitCode::from(5),
    };
    if write_json(&outcome) == ExitCode::SUCCESS {
        exit
    } else {
        ExitCode::from(5)
    }
}

fn serve(options: &Options) -> ExitCode {
    let Some(interaction) = interaction_runtime(options, true) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    if interaction.health().interaction_state != InteractionState::Ready {
        return safe_error("interaction_not_ready", ExitCode::from(3));
    }
    if env_flag("PPL_REQUIRE_IDENTITY") {
        let Some(identity) = iam_runtime(options) else {
            return safe_error("identity_state_required", ExitCode::from(2));
        };
        if identity.health(declared_environment()).identity_state != IdentityState::Ready {
            return safe_error("identity_not_ready", ExitCode::from(3));
        }
    }
    println!(
        "{{\"softwareStatus\":\"healthy\",\"interactionState\":\"ready\",\"identityRequired\":{},\"maturity\":\"in-development\",\"listener\":\"none\"}}",
        env_flag("PPL_REQUIRE_IDENTITY")
    );
    std::thread::park();
    ExitCode::SUCCESS
}

fn iam_bootstrap(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    match runtime.bootstrap_local(now) {
        Ok(record) => write_json(&record),
        Err(_) => safe_error("identity_bootstrap_failed", ExitCode::from(5)),
    }
}

fn iam_configure_demo(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    let Ok(record) = runtime.bootstrap_local(now) else {
        return safe_error("identity_bootstrap_failed", ExitCode::from(5));
    };
    let configuration = demonstration_configuration(&record.environment_id, now);
    match runtime.configure(&configuration, now) {
        Ok(()) => println!("{{\"status\":\"demonstration-configuration-ready\"}}"),
        Err(_) => return safe_error("identity_configuration_failed", ExitCode::from(5)),
    }
    ExitCode::SUCCESS
}

fn iam_health(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let health = runtime.health(declared_environment());
    let ready = health.identity_state == IdentityState::Ready;
    if write_json(&health) != ExitCode::SUCCESS {
        return ExitCode::from(5);
    }
    if ready {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

fn iam_workload(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    match runtime.workload_context(options.workload_id.as_deref().unwrap_or_default(), now) {
        Ok(context) => write_json(&context),
        Err(_) => safe_error("workload_context_refused", ExitCode::from(4)),
    }
}

fn iam_issue_grant(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    let Some(input) = options.input.as_deref() else {
        return safe_error("input_required", ExitCode::from(2));
    };
    let Ok(request) = read_json_input::<GrantRequest>(input) else {
        return safe_error("grant_request_invalid", ExitCode::from(2));
    };
    let Ok(grant) = runtime.issue_grant(&request, now) else {
        return safe_error("grant_request_refused", ExitCode::from(4));
    };
    let Some(output) = options.output.as_deref() else {
        return safe_error("output_required", ExitCode::from(2));
    };
    if write_private_json(output, &grant).is_err() {
        return safe_error("grant_output_failed", ExitCode::from(5));
    }
    println!("{{\"status\":\"grant-written\"}}");
    ExitCode::SUCCESS
}

fn iam_establish(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    let Some(input) = options.input.as_deref() else {
        return safe_error("input_required", ExitCode::from(2));
    };
    let Ok(grant) = read_json_input::<DemonstrationSignInGrant>(input) else {
        return safe_error("grant_invalid", ExitCode::from(2));
    };
    match runtime.establish_session(&grant, now) {
        Ok(outcome) => write_json(&outcome),
        Err(_) => safe_error("session_state_failure", ExitCode::from(5)),
    }
}

fn iam_terminate(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    match runtime.terminate_session(
        options.session_reference.as_deref().unwrap_or_default(),
        options.reason.as_deref().unwrap_or_default(),
        now,
    ) {
        Ok(outcome) => write_json(&outcome),
        Err(_) => safe_error("session_termination_refused", ExitCode::from(4)),
    }
}

fn iam_revoke(options: &Options) -> ExitCode {
    let Some(runtime) = iam_runtime(options) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Some(now) = resolve_now(options) else {
        return safe_error("clock_value_invalid", ExitCode::from(2));
    };
    match runtime.revoke_trust(options.reason.as_deref().unwrap_or_default(), now) {
        Ok(()) => println!("{{\"status\":\"trust-revoked\"}}"),
        Err(_) => return safe_error("trust_revocation_refused", ExitCode::from(4)),
    }
    ExitCode::SUCCESS
}

fn demonstration_configuration(
    environment_id: &str,
    now: OffsetDateTime,
) -> DemonstrationConfiguration {
    let assertion =
        |assertion_type, source: &str, actor: &str, application: &str| AuthoritativeAssertion {
            source_id: source.to_owned(),
            assertion_type,
            subject_id: actor.to_owned(),
            resource_id: application.to_owned(),
            purpose_codes: vec!["demonstrate-discovery".to_owned()],
            status: AssertionStatus::Active,
            effective_at: format_time(now - Duration::minutes(1)),
            expires_at: format_time(now + Duration::days(365)),
            version: "1.0.0".to_owned(),
        };
    DemonstrationConfiguration {
        environment_id: environment_id.to_owned(),
        policy_version: "1.0.0".to_owned(),
        relationship_source: "source-relationships".to_owned(),
        consent_source: "source-consents".to_owned(),
        supported_obligations: vec!["mark-synthetic".to_owned(), "restrict-realm".to_owned()],
        workloads: vec![WorkloadRegistration {
            workload_id: "workload-director".to_owned(),
            audiences: vec!["iam-01-service".to_owned()],
            contract_actions: vec!["I-004:request-grant".to_owned()],
            enabled: true,
        }],
        actors: vec![
            SyntheticActorRegistration {
                actor_id: "synthetic-reviewer".to_owned(),
                display_name: "Alex Reviewer".to_owned(),
                roles: vec!["reviewer".to_owned()],
                applications: vec!["workbench-app".to_owned()],
                purposes: vec!["demonstrate-discovery".to_owned()],
                synthetic_realm: "realm-synthetic-demo".to_owned(),
                enabled: true,
            },
            SyntheticActorRegistration {
                actor_id: "synthetic-coordinator".to_owned(),
                display_name: "Sam Coordinator".to_owned(),
                roles: vec!["coordinator".to_owned()],
                applications: vec!["workflow-app".to_owned()],
                purposes: vec!["demonstrate-discovery".to_owned()],
                synthetic_realm: "realm-synthetic-demo".to_owned(),
                enabled: true,
            },
        ],
        assertions: vec![
            assertion(
                AssertionType::Relationship,
                "source-relationships",
                "synthetic-reviewer",
                "workbench-app",
            ),
            assertion(
                AssertionType::Consent,
                "source-consents",
                "synthetic-reviewer",
                "workbench-app",
            ),
            assertion(
                AssertionType::Relationship,
                "source-relationships",
                "synthetic-coordinator",
                "workflow-app",
            ),
            assertion(
                AssertionType::Consent,
                "source-consents",
                "synthetic-coordinator",
                "workflow-app",
            ),
        ],
    }
}

fn parse_options(args: impl Iterator<Item = String>) -> Result<Options, &'static str> {
    let mut options = Options::default();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--state-dir" => set_path(&mut options.state_dir, args.next(), "state_directory")?,
            "--environment-id" => {
                set_string(&mut options.environment_id, args.next(), "environment")?;
            }
            "--now" => set_string(&mut options.now, args.next(), "clock")?,
            "--output" => set_path(&mut options.output, args.next(), "output")?,
            "--workload-id" => {
                set_string(&mut options.workload_id, args.next(), "workload")?;
            }
            "--session-reference" => set_string(
                &mut options.session_reference,
                args.next(),
                "session_reference",
            )?,
            "--reason" => set_string(&mut options.reason, args.next(), "reason")?,
            value if value.starts_with('-') => return Err("option_unsupported"),
            value => {
                if options.input.replace(PathBuf::from(value)).is_some() {
                    return Err("input_repeated");
                }
            }
        }
    }
    Ok(options)
}

fn set_path(
    target: &mut Option<PathBuf>,
    value: Option<String>,
    name: &'static str,
) -> Result<(), &'static str> {
    let Some(value) = value else {
        return Err(match name {
            "state_directory" => "state_directory_value_required",
            "output" => "output_value_required",
            _ => "option_value_required",
        });
    };
    if target.replace(PathBuf::from(value)).is_some() {
        return Err(match name {
            "state_directory" => "state_directory_repeated",
            "output" => "output_repeated",
            _ => "option_repeated",
        });
    }
    Ok(())
}

fn set_string(
    target: &mut Option<String>,
    value: Option<String>,
    name: &'static str,
) -> Result<(), &'static str> {
    let Some(value) = value else {
        return Err(match name {
            "environment" => "environment_value_required",
            "clock" => "clock_value_required",
            "workload" => "workload_value_required",
            "session_reference" => "session_reference_value_required",
            "reason" => "reason_value_required",
            _ => "option_value_required",
        });
    };
    if target.replace(value).is_some() {
        return Err("option_repeated");
    }
    Ok(())
}

fn only_base_options(options: &Options) -> bool {
    options.output.is_none()
        && options.workload_id.is_none()
        && options.session_reference.is_none()
        && options.reason.is_none()
        && options.input.is_none()
}

fn no_action_options(options: &Options) -> bool {
    only_base_options(options) && options.environment_id.is_none()
}

fn interaction_runtime(options: &Options, require_state: bool) -> Option<InteractionRuntime> {
    let state_dir = resolve_interaction_state_dir(options);
    if require_state && state_dir.is_none() {
        return None;
    }
    let state_dir = state_dir.unwrap_or_else(|| env::temp_dir().join("ppl-manifest-no-state"));
    let environment_id = options
        .environment_id
        .clone()
        .or_else(|| env::var("PPL_ENVIRONMENT_ID").ok())
        .unwrap_or_else(|| "env-local-development".to_owned());
    if environment_id.len() < 8 || environment_id.len() > 128 {
        return None;
    }
    Some(InteractionRuntime::new(RuntimeConfig {
        state_dir,
        environment_id,
    }))
}

fn iam_runtime(options: &Options) -> Option<IamRuntime> {
    resolve_state_dir(options)
        .map(|state_dir| IamRuntime::with_declared_environment(state_dir, declared_environment()))
}

fn declared_environment() -> DeclaredEnvironment {
    let environment_class = match env::var("PPL_ENVIRONMENT_CLASS").as_deref() {
        Ok("portable-isolated") => EnvironmentClass::PortableIsolated,
        Ok("hosted-shared") => EnvironmentClass::HostedShared,
        Ok("production-like") => EnvironmentClass::ProductionLike,
        Ok("production") => EnvironmentClass::Production,
        _ => EnvironmentClass::LocalScratch,
    };
    let information_profile = match env::var("PPL_INFORMATION_PROFILE").as_deref() {
        Ok("non-synthetic-authorised") => InformationProfile::NonSyntheticAuthorised,
        _ => InformationProfile::SyntheticOnly,
    };
    let required_trust_profile = match env::var("PPL_TRUST_PROFILE").as_deref() {
        Ok("managed") => TrustProfile::Managed,
        _ => TrustProfile::LocalSynthetic,
    };
    DeclaredEnvironment {
        environment_class,
        information_profile,
        required_trust_profile,
    }
}

fn resolve_state_dir(options: &Options) -> Option<PathBuf> {
    options
        .state_dir
        .clone()
        .or_else(|| env::var_os("PPL_STATE_DIR").map(PathBuf::from))
}

fn resolve_interaction_state_dir(options: &Options) -> Option<PathBuf> {
    options
        .state_dir
        .clone()
        .or_else(|| env::var_os("PPL_INTERACTION_STATE_DIR").map(PathBuf::from))
        .or_else(|| env::var_os("PPL_STATE_DIR").map(PathBuf::from))
}

fn resolve_now(options: &Options) -> Option<OffsetDateTime> {
    options.now.as_deref().map_or_else(
        || Some(OffsetDateTime::now_utc()),
        |value| parse_time(value).ok(),
    )
}

fn format_time(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn env_flag(name: &str) -> bool {
    matches!(env::var(name).as_deref(), Ok("1" | "true" | "yes"))
}

fn read_json_input<T: DeserializeOwned>(path: &Path) -> Result<T, ()> {
    let metadata = fs::metadata(path).map_err(|_| ())?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return Err(());
    }
    let input = fs::read(path).map_err(|_| ())?;
    serde_json::from_slice(&input).map_err(|_| ())
}

fn write_private_json(path: &Path, value: &impl serde::Serialize) -> Result<(), ()> {
    let bytes = serde_json::to_vec(value).map_err(|_| ())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|_| ())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| ())
}

fn write_json(value: &impl serde::Serialize) -> ExitCode {
    match serde_json::to_string(value) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(_) => safe_error("output_serialization_failure", ExitCode::from(5)),
    }
}

fn usage_error() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn safe_error(code: &str, exit: ExitCode) -> ExitCode {
    eprintln!("{{\"error\":\"{code}\"}}");
    exit
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Options, USAGE, parse_options};

    #[test]
    fn usage_names_every_supported_command() {
        for command in [
            "describe",
            "manifest",
            "healthcheck",
            "process",
            "serve",
            "iam-bootstrap",
            "iam-configure-demo",
            "iam-health",
            "iam-workload",
            "iam-issue-grant",
            "iam-establish",
            "iam-terminate",
            "iam-revoke",
        ] {
            assert!(USAGE.contains(command));
        }
    }

    #[test]
    fn options_are_order_independent() {
        let options = parse_options(
            [
                "--environment-id",
                "env-local-001",
                "input.json",
                "--state-dir",
                "/tmp/ppl-state",
                "--now",
                "2030-08-25T12:01:00Z",
                "--output",
                "/tmp/grant.json",
                "--workload-id",
                "workload-director",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("options must parse");

        assert_eq!(
            options,
            Options {
                state_dir: Some(PathBuf::from("/tmp/ppl-state")),
                environment_id: Some("env-local-001".to_owned()),
                now: Some("2030-08-25T12:01:00Z".to_owned()),
                output: Some(PathBuf::from("/tmp/grant.json")),
                workload_id: Some("workload-director".to_owned()),
                session_reference: None,
                reason: None,
                input: Some(PathBuf::from("input.json")),
            }
        );
    }

    #[test]
    fn repeated_input_is_refused() {
        assert_eq!(
            parse_options(["one.json".to_owned(), "two.json".to_owned()].into_iter()),
            Err("input_repeated")
        );
    }
}
