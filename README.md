# wp-agent

This repository currently contains the design docs and a minimal Rust workspace skeleton
for the first implementation wave.

Workspace layout:

- `crates/wp-agent-contracts`
  Shared contract types and versioned schema objects.
- `crates/wp-agent-validate`
  Static validators for plans, results, config, and state.
- `crates/wp-agent-shared`
  Shared errors, IDs, paths, and common runtime helpers.
- `crates/wp-agentd`
  Edge daemon skeleton.
- `crates/wp-agent-exec`
  ActionPlan runtime skeleton.
- `crates/wp-agent-upgrader`
  Upgrade helper skeleton.
- `crates/wp-agent-gateway`
  Southbound gateway/server skeleton.
- `crates/wp-agent-control`
  Control-center core skeleton.

The current code is intentionally minimal and is meant to anchor the module boundaries
defined under `doc/design`.

