#!/usr/bin/env bash
# 停止 warp-agentd（配合 start.sh）。
# 用法：./stop.sh
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="${WARP_AGENTD_CONFIG_DIR:-${SCRIPT_DIR}}"
PIDFILE="${CONFIG_DIR}/log/agentd.pid"

if [[ ! -f "${PIDFILE}" ]]; then
  echo "warp-agentd not running (no pidfile ${PIDFILE})"
  exit 0
fi

PID="$(cat "${PIDFILE}")"
if kill -0 "${PID}" 2>/dev/null; then
  kill "${PID}"
  echo "stopped warp-agentd pid=${PID}"
else
  echo "warp-agentd not running (stale pidfile pid=${PID})"
fi
rm -f "${PIDFILE}"
