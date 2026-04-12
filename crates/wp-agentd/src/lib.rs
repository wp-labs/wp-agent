//! Edge daemon skeleton.

pub mod bootstrap;
pub mod config_runtime;
pub mod self_observability;
pub mod state_store;

pub fn run() {
    let root = std::env::current_dir().expect("current_dir");
    let config = config_runtime::load_default(root.join(".wp-agentd"));
    let paths = &config.paths;
    let root_dir = std::path::Path::new(&paths.root_dir);
    let run_dir = std::path::Path::new(&paths.run_dir);
    let state_dir = std::path::Path::new(&paths.state_dir);
    let log_dir = std::path::Path::new(&paths.log_dir);

    bootstrap::initialize(root_dir, run_dir, state_dir, log_dir).expect("bootstrap");
    self_observability::register();
    eprintln!("wp-agentd initialized at {}", paths.root_dir);
}
