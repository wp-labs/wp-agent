use std::fs;
use std::path::PathBuf;

use wp_agent_contracts::action_plan::ActionPlanContract;
use wp_agent_contracts::action_result::ActionResultContract;
use wp_agent_contracts::agent_config::AgentConfigContract;
use wp_agent_contracts::state_exec::AgentRuntimeState;
use wp_agent_contracts::state_logs::LogStateContract;
use wp_agent_validate::action_plan::validate_action_plan;
use wp_agent_validate::action_result::validate_action_result;
use wp_agent_validate::config::validate_config;
use wp_agent_validate::state::{validate_execution_state, validate_log_state};

fn fixture_text(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(relative);
    fs::read_to_string(path).expect("read fixture")
}

#[test]
fn action_plan_valid_fixture_passes() {
    let fixture: ActionPlanContract =
        serde_json::from_str(&fixture_text("contracts/action-plan/valid/basic.json"))
            .expect("deserialize action plan fixture");

    validate_action_plan(&fixture).expect("valid action plan");
}

#[test]
fn action_plan_invalid_kind_fixture_fails() {
    let fixture: ActionPlanContract =
        serde_json::from_str(&fixture_text("contracts/action-plan/invalid/bad-kind.json"))
            .expect("deserialize action plan fixture");

    let err = validate_action_plan(&fixture).expect_err("invalid action plan");
    assert_eq!(err.code, "invalid_kind");
}

#[test]
fn action_plan_invalid_window_fixture_fails() {
    let fixture: ActionPlanContract = serde_json::from_str(&fixture_text(
        "contracts/action-plan/invalid/expired-window.json",
    ))
    .expect("deserialize action plan fixture");

    let err = validate_action_plan(&fixture).expect_err("invalid action plan");
    assert_eq!(err.code, "expired_or_non_increasing_window");
}

#[test]
fn action_result_valid_fixture_passes() {
    let fixture: ActionResultContract =
        serde_json::from_str(&fixture_text("contracts/action-result/valid/basic.json"))
            .expect("deserialize action result fixture");

    validate_action_result(&fixture).expect("valid action result");
}

#[test]
fn action_result_invalid_fixture_fails() {
    let fixture: ActionResultContract = serde_json::from_str(&fixture_text(
        "contracts/action-result/invalid/missing-step-records.json",
    ))
    .expect("deserialize action result fixture");

    let err = validate_action_result(&fixture).expect_err("invalid action result");
    assert_eq!(err.code, "missing_step_records");
}

#[test]
fn config_valid_fixture_passes() {
    let fixture: AgentConfigContract =
        serde_json::from_str(&fixture_text("contracts/config/valid/standalone.json"))
            .expect("deserialize config fixture");

    validate_config(&fixture).expect("valid config");
}

#[test]
fn config_invalid_fixture_fails() {
    let fixture: AgentConfigContract = serde_json::from_str(&fixture_text(
        "contracts/config/invalid/managed-missing-endpoint.json",
    ))
    .expect("deserialize config fixture");

    let err = validate_config(&fixture).expect_err("invalid config");
    assert_eq!(err.code, "missing_control_plane_endpoint");
}

#[test]
fn runtime_state_valid_fixture_passes() {
    let fixture: AgentRuntimeState =
        serde_json::from_str(&fixture_text("contracts/state/runtime-valid.json"))
            .expect("deserialize runtime state fixture");

    validate_execution_state(&fixture).expect("valid runtime state");
}

#[test]
fn log_state_valid_fixture_passes() {
    let fixture: LogStateContract =
        serde_json::from_str(&fixture_text("contracts/state/logs-valid.json"))
            .expect("deserialize log state fixture");

    validate_log_state(&fixture).expect("valid log state");
}
