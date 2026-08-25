//! Local M1 framework-host adapter and process boundary.

use std::{env, fs, path::PathBuf, process::ExitCode};

use ppl_contracts::{InteractionEnvelope, OutcomeStatus};
use ppl_int_01::{InteractionRuntime, InteractionState, RuntimeConfig, parse_time};

const USAGE: &str = "usage: ppl-framework-host <describe|manifest|healthcheck|process|serve> [--state-dir PATH] [--environment-id ID] [--now RFC3339] [INPUT.json]";
const MAX_INPUT_BYTES: u64 = 1_048_576;

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    state_dir: Option<PathBuf>,
    environment_id: Option<String>,
    now: Option<String>,
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
        "manifest" if options.input.is_none() => manifest(&options),
        "healthcheck" if options.input.is_none() && options.now.is_none() => healthcheck(&options),
        "process" => process(&options),
        "serve" if options.input.is_none() && options.now.is_none() => serve(&options),
        _ => usage_error(),
    }
}

fn describe() -> ExitCode {
    println!("framework-host|in-development|local-assurance-only");
    println!("{}", ppl_iam_01::descriptor());
    println!("{}", ppl_int_01::descriptor());
    ExitCode::SUCCESS
}

fn manifest(options: &Options) -> ExitCode {
    let Some(runtime) = runtime(options, false) else {
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
            "{{\"softwareStatus\":\"healthy\",\"interactionState\":\"not-configured\",\"maturity\":\"in-development\"}}"
        );
        return ExitCode::SUCCESS;
    }
    let Some(runtime) = runtime(options, true) else {
        return safe_error("state_configuration_invalid", ExitCode::from(2));
    };
    let health = runtime.health();
    let ready = health.interaction_state == InteractionState::Ready;
    if write_json(&health) != ExitCode::SUCCESS {
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
    let Some(runtime) = runtime(options, true) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let Ok(metadata) = fs::metadata(input) else {
        return safe_error("input_unavailable", ExitCode::from(2));
    };
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return safe_error("input_size_or_type_invalid", ExitCode::from(2));
    }
    let Ok(input) = fs::read(input) else {
        return safe_error("input_unavailable", ExitCode::from(2));
    };
    let envelope: InteractionEnvelope = match serde_json::from_slice(&input) {
        Ok(envelope) => envelope,
        Err(_) => return safe_error("interaction_invalid", ExitCode::from(2)),
    };
    let now = match options.now.as_deref() {
        Some(value) => match parse_time(value) {
            Ok(value) => value,
            Err(_) => return safe_error("clock_value_invalid", ExitCode::from(2)),
        },
        None => time::OffsetDateTime::now_utc(),
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
    let Some(runtime) = runtime(options, true) else {
        return safe_error("state_configuration_required", ExitCode::from(2));
    };
    let health = runtime.health();
    if health.interaction_state != InteractionState::Ready {
        return safe_error("interaction_not_ready", ExitCode::from(3));
    }
    println!(
        "{{\"softwareStatus\":\"healthy\",\"interactionState\":\"ready\",\"maturity\":\"in-development\",\"listener\":\"none\"}}"
    );
    std::thread::park();
    ExitCode::SUCCESS
}

fn parse_options(args: impl Iterator<Item = String>) -> Result<Options, &'static str> {
    let mut options = Options::default();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--state-dir" => {
                let value = args.next().ok_or("state_directory_value_required")?;
                if options.state_dir.replace(PathBuf::from(value)).is_some() {
                    return Err("state_directory_repeated");
                }
            }
            "--environment-id" => {
                let value = args.next().ok_or("environment_value_required")?;
                if options.environment_id.replace(value).is_some() {
                    return Err("environment_repeated");
                }
            }
            "--now" => {
                let value = args.next().ok_or("clock_value_required")?;
                if options.now.replace(value).is_some() {
                    return Err("clock_repeated");
                }
            }
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

fn runtime(options: &Options, require_state: bool) -> Option<InteractionRuntime> {
    let state_dir = resolve_state_dir(options);
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

fn resolve_state_dir(options: &Options) -> Option<PathBuf> {
    options
        .state_dir
        .clone()
        .or_else(|| env::var_os("PPL_STATE_DIR").map(PathBuf::from))
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

    use super::{USAGE, parse_options};

    #[test]
    fn usage_names_every_supported_command() {
        for command in ["describe", "manifest", "healthcheck", "process", "serve"] {
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
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("options must parse");

        assert_eq!(options.environment_id.as_deref(), Some("env-local-001"));
        assert_eq!(options.input, Some(PathBuf::from("input.json")));
        assert_eq!(options.state_dir, Some(PathBuf::from("/tmp/ppl-state")));
        assert_eq!(options.now.as_deref(), Some("2030-08-25T12:01:00Z"));
    }

    #[test]
    fn repeated_input_is_refused() {
        assert_eq!(
            parse_options(["one.json".to_owned(), "two.json".to_owned()].into_iter()),
            Err("input_repeated")
        );
    }
}
