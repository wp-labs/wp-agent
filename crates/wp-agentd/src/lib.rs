//! Edge daemon skeleton.

use std::path::Path;

pub mod bootstrap;
pub mod config_runtime;
pub mod local_exec;
pub mod reporting_pipeline;
pub mod scheduler;
pub mod self_observability;
pub mod state_store;

pub fn run() {
    let root = std::env::current_dir().expect("current_dir");
    let config_root = root.join(".wp-agentd");
    let config = config_runtime::load_or_init(&config_root).expect("load config");
    let paths = &config.paths;
    let root_dir = Path::new(&paths.root_dir);
    let run_dir = Path::new(&paths.run_dir);
    let state_dir = Path::new(&paths.state_dir);
    let log_dir = Path::new(&paths.log_dir);

    bootstrap::initialize(root_dir, run_dir, state_dir, log_dir).expect("bootstrap");
    let runtime_path = state_store::agent_runtime::path_for(state_dir);
    let runtime_state = state_store::agent_runtime::load_or_default(&runtime_path)
        .expect("load default runtime state");
    state_store::agent_runtime::store(&runtime_path, &runtime_state).expect("write runtime state");
    let queue_path = state_store::execution_queue::path_for(state_dir);
    let queue_state =
        state_store::execution_queue::load_or_default(&queue_path).expect("load execution queue");
    state_store::execution_queue::store(&queue_path, &queue_state).expect("write execution queue");
    self_observability::register();
    eprintln!("wp-agentd initialized at {}", paths.root_dir);
}
