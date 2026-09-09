#!/usr/bin/env bash
# 停止 warp-gateway data-plane 观测栈。
#
# 用法：
#   ./stop.sh     停止容器（保留 metrics_data / logs_data 数据卷）
#   ./stop.sh -v  停止并删除数据卷（清空已采集的指标/日志）
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${SCRIPT_DIR}/docker-compose.yml"
COMPOSE_CMD=()

if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE_CMD=(docker-compose)
else
  echo "未检测到 docker compose 或 docker-compose。" >&2
  exit 1
fi

if [[ "${1:-}" == "-v" ]]; then
  echo "停止并删除数据卷..."
  "${COMPOSE_CMD[@]}" -f "${COMPOSE_FILE}" down -v
elif [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  echo "用法: ./stop.sh [-v]" >&2
  exit 0
else
  echo "停止服务（保留数据卷）..."
  "${COMPOSE_CMD[@]}" -f "${COMPOSE_FILE}" down
fi
