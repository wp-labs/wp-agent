#!/usr/bin/env bash
# 停止 data-plane 的 wparse 后台实例（配合 start-wparse.sh）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_ROOT="${WPARSE_WORK_ROOT:-${SCRIPT_DIR}}"
PIDFILE="${WORK_ROOT}/data/logs/wparse.pid"

if [[ ! -f "${PIDFILE}" ]]; then
  echo "wparse not running (no pidfile ${PIDFILE})"
  exit 0
fi

PID="$(cat "${PIDFILE}")"
if kill -0 "${PID}" 2>/dev/null; then
  kill "${PID}"
  echo "stopped wparse pid=${PID}"
else
  echo "wparse not running (stale pidfile pid=${PID})"
fi
rm -f "${PIDFILE}"
