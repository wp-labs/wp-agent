#!/usr/bin/env bash
# 启动 warp-gateway 数据平台（wparse，接收/解析 Agent 数据）。
#
# 使用本目录 ../bin（sysrun/warp-gateway/bin）下自带的 wparse 二进制（版本与 data-plane 工程配套）。
# 用法：
#   ./start-wparse.sh               # 后台常驻，pid/log 落 data/logs/
#   ./start-wparse.sh --foreground  # 前台运行（联调看日志）
# 停止：./stop-wparse.sh
#
# 可覆盖环境变量：
#   WPARSE_BIN         wparse 可执行文件路径（默认 ../bin/wparse）
#   WPARSE_WORK_ROOT   工程根目录（默认本脚本所在目录，即 data-plane）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_ROOT="${WPARSE_WORK_ROOT:-${SCRIPT_DIR}}"
BIN_DIR="$(cd "${SCRIPT_DIR}/../bin" && pwd)"
WPARSE="${WPARSE_BIN:-${BIN_DIR}/wparse}"

CONF_FILE="${WORK_ROOT}/conf/wparse.toml"
LOG_DIR="${WORK_ROOT}/data/logs"

if [[ ! -x "${WPARSE}" ]]; then
  echo "wparse binary not found or not executable: ${WPARSE}" >&2
  echo "expect one at ${BIN_DIR}/wparse (sysrun/warp-gateway/bin/wparse)" >&2
  exit 1
fi
if [[ ! -f "${CONF_FILE}" ]]; then
  echo "missing wparse config: ${CONF_FILE}" >&2
  exit 1
fi
mkdir -p "${LOG_DIR}"

start_daemon() {
  exec "${WPARSE}" daemon --work-root "${WORK_ROOT}"
}

if [[ "${1:-}" == "--foreground" ]]; then
  echo "wparse foreground: ${WPARSE} daemon --work-root ${WORK_ROOT}"
  start_daemon
fi

PIDFILE="${LOG_DIR}/wparse.pid"
if [[ -f "${PIDFILE}" ]]; then
  OLD_PID="$(cat "${PIDFILE}")"
  if kill -0 "${OLD_PID}" 2>/dev/null; then
    echo "wparse already running (pid=${OLD_PID}, ${PIDFILE})" >&2
    exit 1
  fi
  echo "removing stale pidfile ${PIDFILE}" >&2
  rm -f "${PIDFILE}"
fi

nohup "${WPARSE}" daemon --work-root "${WORK_ROOT}" >>"${LOG_DIR}/wparse-daemon.log" 2>&1 &
echo $! >"${PIDFILE}"

echo "wparse started pid=$(cat "${PIDFILE}")"
echo "  binary    : ${WPARSE}"
echo "  work-root : ${WORK_ROOT}"
echo "  stdout log: ${LOG_DIR}/wparse-daemon.log"
echo "  engine log: ${LOG_DIR}/wparse.log"
echo "  stop      : ${SCRIPT_DIR}/stop-wparse.sh"
