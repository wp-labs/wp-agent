#!/usr/bin/env bash
# 启动 warp-agentd（macOS P0 采集实例）。
#
# 运行布局（sysrun/warp-agentd/）：
#   agentd.toml            运行设定（[agent]/[control_plane]/[telemetry.logs]/[paths]）
#   tasks/macos-p0.toml    采集工作任务清单（由 agentd.toml file_inputs_file 引用）
#   bin/                   可选：放置独立 warp-agentd 可执行文件（脱离仓库运行）
#   conf/                  可选：备用配置目录
#   run|state|log/         运行时产物（已 gitignore）
#
# 用法：
#   ./start.sh              常驻后台（pid/log 落 log/）
#   ./start.sh --foreground 前台运行（联调看日志）
# 停止：./stop.sh
#
# 可覆盖 env：
#   WARP_AGENTD_BIN         warp-agentd 可执行文件（默认 bin/warp-agentd → 仓库 target/debug/warp-agentd）
#   WARP_AGENTD_CONFIG_DIR  配置目录（默认本目录，须含 agentd.toml）
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# sysrun/warp-agentd → 仓库根（target/debug 开发构建所在）
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"

CONFIG_DIR="${WARP_AGENTD_CONFIG_DIR:-${SCRIPT_DIR}}"
LOG_DIR="${CONFIG_DIR}/log"
PIDFILE="${LOG_DIR}/agentd.pid"

resolve_bin() {
  local local_bin="${SCRIPT_DIR}/bin/warp-agentd"
  if [[ -n "${WARP_AGENTD_BIN:-}" ]]; then
    printf '%s\n' "${WARP_AGENTD_BIN}"
  elif [[ -x "${local_bin}" ]]; then
    printf '%s\n' "${local_bin}"
  else
    printf '%s\n' "${REPO_ROOT}/target/debug/warp-agentd"
  fi
}

BIN="$(resolve_bin)"

if [[ "${1:-}" == "--foreground" ]]; then
  echo "warp-agentd foreground: ${BIN} --config-dir ${CONFIG_DIR}"
  exec "${BIN}" --config-dir "${CONFIG_DIR}"
fi

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  sed -n '1,28p' "${BASH_SOURCE[0]}" | sed 's/^#\{0,1\} //'
  exit 0
fi

if [[ ! -x "${BIN}" ]]; then
  echo "warp-agentd binary not found: ${BIN}" >&2
  echo "请先 cargo build -p warp-agentd，或设置 WARP_AGENTD_BIN 指向可执行文件" >&2
  exit 1
fi
if [[ ! -f "${CONFIG_DIR}/agentd.toml" ]]; then
  echo "missing agentd.toml: ${CONFIG_DIR}/agentd.toml" >&2
  exit 1
fi
mkdir -p "${LOG_DIR}"

if [[ -f "${PIDFILE}" ]]; then
  OLD_PID="$(cat "${PIDFILE}")"
  if kill -0 "${OLD_PID}" 2>/dev/null; then
    echo "warp-agentd already running (pid=${OLD_PID}, ${PIDFILE})" >&2
    exit 1
  fi
  echo "removing stale pidfile ${PIDFILE}" >&2
  rm -f "${PIDFILE}"
fi

nohup "${BIN}" --config-dir "${CONFIG_DIR}" >>"${LOG_DIR}/agentd.out" 2>&1 &
echo $! >"${PIDFILE}"

echo "warp-agentd started pid=$(cat "${PIDFILE}")"
echo "  binary    : ${BIN}"
echo "  config-dir: ${CONFIG_DIR}"
echo "  stdout log: ${LOG_DIR}/agentd.out"
echo "  stop      : ${SCRIPT_DIR}/stop.sh"
