//! Self-observability placeholders.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthState {
    Idle,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHealthSnapshot {
    pub state: HealthState,
    pub queue_depth: usize,
    pub running_count: usize,
    pub reporting_count: usize,
    pub updated_at: String,
}

pub fn register() {
    eprintln!("self-observability registered");
}

pub fn emit(snapshot: &RuntimeHealthSnapshot) {
    eprintln!(
        "health state={:?} queue={} running={} reporting={} updated_at={}",
        snapshot.state,
        snapshot.queue_depth,
        snapshot.running_count,
        snapshot.reporting_count,
        snapshot.updated_at
    );
}
