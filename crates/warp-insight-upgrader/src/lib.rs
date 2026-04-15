//! Upgrade helper skeleton.

pub mod prepare;
pub mod rollback;
pub mod switch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Prepare,
    Switch,
    Rollback,
}

pub fn run() -> Result<(), String> {
    let stage = parse_cli_args(std::env::args().skip(1))?;
    execute_stage(stage);
    Ok(())
}

fn parse_cli_args<I>(mut args: I) -> Result<Stage, String>
where
    I: Iterator<Item = String>,
{
    match args.next().as_deref() {
        Some("prepare") => Ok(Stage::Prepare),
        Some("switch") => Ok(Stage::Switch),
        Some("rollback") => Ok(Stage::Rollback),
        Some(other) => Err(format!("unsupported subcommand: {other}")),
        None => Err("missing subcommand: expected one of prepare|switch|rollback".to_string()),
    }
}

fn execute_stage(stage: Stage) {
    match stage {
        Stage::Prepare => prepare::execute(),
        Stage::Switch => switch::execute(),
        Stage::Rollback => rollback::execute(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Stage, parse_cli_args};

    #[test]
    fn parse_prepare_subcommand() {
        let stage = parse_cli_args(["prepare".to_string()].into_iter()).expect("parse prepare");
        assert_eq!(stage, Stage::Prepare);
    }

    #[test]
    fn parse_requires_supported_subcommand() {
        let err = parse_cli_args(std::iter::empty()).expect_err("missing subcommand should fail");
        assert!(err.contains("missing subcommand"));
    }
}
