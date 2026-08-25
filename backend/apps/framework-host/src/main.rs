//! Minimal process boundary for repository and deployment qualification.

use std::{env, process::ExitCode};

const USAGE: &str = "usage: ppl-framework-host <describe|healthcheck|serve>";

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("describe") => {
            println!("framework-host|repository-skeleton");
            println!("{}", ppl_iam_01::descriptor());
            ExitCode::SUCCESS
        }
        Some("healthcheck") => {
            println!("ok");
            ExitCode::SUCCESS
        }
        Some("serve") => {
            println!("framework-host repository skeleton ready");
            std::thread::park();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::USAGE;

    #[test]
    fn usage_names_every_supported_command() {
        assert!(USAGE.contains("describe"));
        assert!(USAGE.contains("healthcheck"));
        assert!(USAGE.contains("serve"));
    }
}
