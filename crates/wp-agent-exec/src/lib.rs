//! `ActionPlan` runtime skeleton.

pub mod result_writer;
pub mod runtime;
pub mod workdir;

pub fn run() {
    let cwd = std::env::current_dir().expect("current_dir");
    let workdir = workdir::open(&cwd);
    runtime::execute(&workdir);
    result_writer::write(&workdir);
    eprintln!("wp-agent-exec skeleton finished");
}
