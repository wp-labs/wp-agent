# warp-insight

This repository currently contains the design docs and a minimal Rust workspace skeleton
for the first implementation wave.

Workspace layout:

- `crates/wist-contracts`
  Shared contract types and versioned schema objects.
- `crates/wist-validate`
  Static validators for plans, results, config, and state.
- `crates/wist-shared`
  Shared errors, IDs, paths, and common runtime helpers.
- `crates/warp-gateway`
  Admin WEB backend skeleton for install links, Agent status, and remote upgrades.
- `crates/gateway-web`
  Browser WEB frontend for the WarpGateWay console.
- `crates/warp-agentd`
  Edge daemon skeleton.
- `crates/wist-exec`
  ActionPlan runtime skeleton.
- `crates/wist-upgrader`
  Upgrade helper skeleton.
- `crates/wist-gateway`
  Southbound gateway/server skeleton.
- `crates/warp-insight-control`
  Control-center core skeleton.

The current code is intentionally minimal and is meant to anchor the module boundaries
defined under `doc/design`.
