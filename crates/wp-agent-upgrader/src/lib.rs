//! Upgrade helper skeleton.

pub mod prepare;
pub mod rollback;
pub mod switch;

pub fn run() {
    prepare::execute();
    switch::execute();
    rollback::execute();
    eprintln!("wp-agent-upgrader skeleton");
}
