#!/usr/bin/env bash
# 完整演示：WarpInsightCenter + VictoriaMetrics 时序链路。
# 一键起 PG+VM → 起 center → 接口创建网关 → simulator 每 3s 上报 →
# 持续展示 VM 时序数据（gateway_up / uptime / PG 快照），退出自动清理。
#
# 用法：
#   ./scripts/demo-insight-center.sh                # 持续运行直到 Ctrl+C
#   DEMO_DURATION=30 ./scripts/demo-insight-center.sh  # 跑 30 秒后自动退出清理
#
# 可覆盖 env：CENTER_URL / VM_URL / PG_URL / GATEWAYS / REPORT_INTERVAL / DEMO_DURATION / DEMO_REFRESH
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_DIR="${REPO_ROOT}/crates/warp-insight-center"
CENTER_BIN="${REPO_ROOT}/target/debug/warp-insight-center"
SIM_BIN="${REPO_ROOT}/target/debug/insight-simulator"

# ── 可覆盖配置 ──
CENTER_URL="${CENTER_URL:-http://127.0.0.1:3100}"
VM_URL="${VM_URL:-http://127.0.0.1:8428}"
PG_URL="${PG_URL:-postgres://demo:demo@127.0.0.1:55432/insight_demo}"
# 管理前端（vite 代理 /api → center）；SKIP_CENTER_WEB=1 可跳过。
CENTER_WEB_URL="${CENTER_WEB_URL:-http://127.0.0.1:5173}"
SKIP_CENTER_WEB="${SKIP_CENTER_WEB:-0}"
# 演示网关：gateway_name:token，可多个（每个起一个 simulator）。
GATEWAYS="${GATEWAYS:-gw-demo-a:tok-demo-a,gw-demo-b:tok-demo-b}"
REPORT_INTERVAL="${REPORT_INTERVAL:-3}"
# 演示时长（秒）；0 = 持续直到 Ctrl+C。
DEMO_DURATION="${DEMO_DURATION:-0}"
DEMO_REFRESH="${DEMO_REFRESH:-5}"

# ── 运行状态 ──
CENTER_STARTED_BY_SCRIPT=0
CENTER_PID=""
CENTER_WEB_PID=""
SIM_PIDS=()

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

cleanup() {
  for pid in "${SIM_PIDS[@]:-}"; do
    if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" 2>/dev/null || true
      wait "${pid}" 2>/dev/null || true
    fi
  done
  if [[ -n "${CENTER_WEB_PID}" ]] && kill -0 "${CENTER_WEB_PID}" 2>/dev/null; then
    kill "${CENTER_WEB_PID}" 2>/dev/null || true
    wait "${CENTER_WEB_PID}" 2>/dev/null || true
  fi
  if [[ "${CENTER_STARTED_BY_SCRIPT}" == "1" && -n "${CENTER_PID}" ]] && kill -0 "${CENTER_PID}" 2>/dev/null; then
    kill "${CENTER_PID}" 2>/dev/null || true
    wait "${CENTER_PID}" 2>/dev/null || true
  fi
  echo
  echo "演示结束，已清理本次启动的 simulator / center-web / center 进程。"
}
trap cleanup EXIT

# ── 服务就绪探测 ──

vm_ready() {
  curl -s -o /dev/null "${VM_URL%/}/health"
}

pg_ready() {
  docker exec "$(docker compose -f "${COMPOSE_DIR}/docker-compose.yml" ps -q postgres 2>/dev/null)" \
    pg_isready -U demo -d insight_demo >/dev/null 2>&1
}

center_ready() {
  local status
  status="$(curl -s -o /dev/null -w "%{http_code}" "${CENTER_URL%/}/api/v1/admin/gateways" || true)"
  [[ "${status}" == "200" || "${status}" == "401" ]]
}

wait_until() {
  # wait_until <描述> <cmd...>
  local desc="$1"
  shift
  echo "等待 ${desc} 就绪..."
  for _ in {1..150}; do
    if "$@" >/dev/null 2>&1; then
      echo "  ${desc} 就绪"
      return 0
    fi
    sleep 0.2
  done
  echo "  ${desc} 未就绪，中止。" >&2
  exit 1
}

# ── 启动各组件 ──

ensure_infra() {
  echo "== 1. 启动基础设施 (PostgreSQL + VictoriaMetrics) =="
  docker compose -f "${COMPOSE_DIR}/docker-compose.yml" up -d
  wait_until "VictoriaMetrics" vm_ready
  wait_until "PostgreSQL" pg_ready
}

ensure_center() {
  echo "== 2. 启动 warp-insight-center（PG 快照 + VM 时序推送）=="
  if center_ready; then
    echo "  center 已在运行（${CENTER_URL}），复用。"
    return
  fi
  require_cmd cargo
  cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" -p warp-insight-center >/dev/null
  local host
  local port
  # 只绑定中心地址；CENTER_URL 用 http://host:port 形式。
  host="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${CENTER_URL}'); print(u.hostname or '127.0.0.1')")"
  port="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${CENTER_URL}'); print(u.port or 80)")"
  (
    cd "${REPO_ROOT}"
    exec nohup env \
      WARP_INSIGHT_CENTER_LISTEN="${host}:${port}" \
      WARP_INSIGHT_CENTER_PUBLIC_URL="${CENTER_URL}" \
      WARP_INSIGHT_CENTER_DATABASE_URL="${PG_URL}" \
      WARP_INSIGHT_CENTER_VICTORIAMETRICS_URL="${VM_URL}" \
      "${CENTER_BIN}" \
      >/tmp/warp-insight-center-demo.log 2>&1
  ) &
  CENTER_PID=$!
  CENTER_STARTED_BY_SCRIPT=1
  wait_until "center" center_ready
}

create_gateways() {
  echo "== 3. 通过接口创建网关并获取 init_url =="
  IFS=',' read -r -a pairs <<< "${GATEWAYS}"
  for entry in "${pairs[@]}"; do
    entry="$(printf '%s' "${entry}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [[ -z "${entry}" ]] && continue
    gateway_name="${entry%%:*}"
    token="${entry#*:}"
    status="$(curl -s -o /tmp/demo-create-response.json -w "%{http_code}" \
      -X POST "${CENTER_URL%/}/api/v1/admin/gateways/instances" \
      -H 'content-type: application/json' \
      -d "{\"gateway_name\":\"${gateway_name}\",\"requested_by\":\"demo\",\"token\":\"${token}\"}")"
    if [[ "${status}" == "201" ]]; then
      # 创建成功：解析响应取 install.init_url（网关初始化入口）。
      local init_url
      init_url="$(python3 - /tmp/demo-create-response.json <<'PY'
import json, sys
try:
    install = json.load(open(sys.argv[1]))["install"]
except Exception:
    install = {}
print(install.get("init_url", ""))
PY
)"
      echo "  创建网关 ${gateway_name} (201)"
      if [[ -n "${init_url}" ]]; then
        echo "    init_url = ${init_url}"
        if [[ "${init_url}" == "${CENTER_URL%/}/api/v1/gateway/initial-config?instance_id=${gateway_name}" ]]; then
          echo "    ✓ init_url 指向当前 center 的 initial-config 端点"
        else
          echo "    ✗ init_url 与 CENTER_URL 不一致（检查 WARP_INSIGHT_CENTER_PUBLIC_URL）" >&2
        fi
        # 用 curl 实际调用 init_url（Bearer 用注册 token），验证网关可拿到初始配置。
        echo "    curl init_url（Bearer ${token}）:"
        curl -s -w "\n    HTTP %{http_code}\n" \
          -H "Authorization: Bearer ${token}" "${init_url}" | sed 's/^/      /'
      else
        echo "    ✗ 响应无 install.init_url" >&2
      fi
    elif [[ "${status}" == "409" ]]; then
      echo "  网关 ${gateway_name} 已存在 (409，复用)"
    else
      echo "  创建网关 ${gateway_name} 失败: HTTP ${status}" >&2
      cat /tmp/demo-create-response.json >&2
      exit 1
    fi
  done
}

center_web_status() {
  curl -s -o /dev/null -w "%{http_code}" "${CENTER_WEB_URL%/}/" || true
}

# 启动管理前端（vite，代理 /api → center）。依赖缺失或未就绪仅跳过，不影响演示。
ensure_center_web() {
  echo "== 5. 启动管理前端 center-web =="
  if [[ "${SKIP_CENTER_WEB}" == "1" ]]; then
    echo "  已跳过（SKIP_CENTER_WEB=1）"
    return
  fi
  if [[ "$(center_web_status)" == "200" ]]; then
    echo "  center-web 已在运行（${CENTER_WEB_URL}），复用。"
    return
  fi
  require_cmd npm
  local web_dir="${REPO_ROOT}/crates/warp-insight-center-web"
  if [[ ! -d "${web_dir}/node_modules" ]]; then
    echo "  center-web 依赖缺失：${web_dir}/node_modules（先 cd 到该目录执行 npm install）" >&2
    exit 1
  fi
  local host
  local port
  host="$(python3 -c "from urllib.parse import urlparse; print(urlparse('${CENTER_WEB_URL}').hostname or '127.0.0.1')")"
  port="$(python3 -c "from urllib.parse import urlparse; print(urlparse('${CENTER_WEB_URL}').port or 80)")"
  (
    cd "${web_dir}"
    exec nohup env WARP_INSIGHT_WEB_PROXY_TARGET="${CENTER_URL}" \
      npm run dev -- --host "${host}" --port "${port}" --strictPort \
      >/tmp/warp-insight-center-web-demo.log 2>&1
  ) &
  CENTER_WEB_PID=$!
  echo "  启动 center-web：${CENTER_WEB_URL} (pid=$!)"
  local ok=0
  for _ in {1..100}; do
    if [[ "$(center_web_status)" == "200" ]]; then
      ok=1
      break
    fi
    if ! kill -0 "${CENTER_WEB_PID}" 2>/dev/null; then
      break
    fi
    sleep 0.2
  done
  if [[ "${ok}" == "1" ]]; then
    echo "  center-web 就绪"
  else
    echo "  center-web 未就绪（日志 /tmp/warp-insight-center-web-demo.log），跳过，不影响演示。"
    CENTER_WEB_PID=""
  fi
}

start_simulators() {
  echo "== 4. 启动 simulator（每 ${REPORT_INTERVAL}s 上报）=="
  require_cmd cargo
  if [[ ! -x "${SIM_BIN}" ]]; then
    cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" -p insight-simulator >/dev/null
  fi
  IFS=',' read -r -a pairs <<< "${GATEWAYS}"
  for entry in "${pairs[@]}"; do
    entry="$(printf '%s' "${entry}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [[ -z "${entry}" ]] && continue
    gateway_name="${entry%%:*}"
    token="${entry#*:}"
    instance_id="inst-${gateway_name#gw-}"
    "${SIM_BIN}" gateway \
      --center-url "${CENTER_URL}" \
      --gateway-id "${gateway_name}" --instance-id "${instance_id}" \
      --token "${token}" --interval "${REPORT_INTERVAL}" \
      --report-agents \
      >"/tmp/insight-simulator-${gateway_name}.log" 2>&1 &
    SIM_PIDS+=($!)
    echo "  simulator[${gateway_name}] pid=$! → 每 ${REPORT_INTERVAL}s 上报（含 2 个模拟 Agent）"
  done
}

# ── 演示循环：展示 VM 时序数据 ──

demo_tick() {
  local now_ts
  now_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  echo "----- ${now_ts} -----"
  echo "[VictoriaMetrics 时序]"

  # gateway_up 当前值（每个网关一个时间序列）。数据走 argv，heredoc 只提供脚本（避免 stdin 冲突）。
  local up_raw
  up_raw="$(curl -s "${VM_URL%/}/api/v1/query" --data-urlencode 'query=gateway_up')"
  python3 - "${up_raw}" <<'PY'
import json, sys
try:
    data = json.loads(sys.argv[1])["data"]["result"]
except Exception:
    data = []
if not data:
    print("  gateway_up: (暂无数据)")
else:
    parts = []
    for m in sorted(data, key=lambda x: x["metric"].get("gateway_id", "")):
        parts.append(f"{m['metric'].get('gateway_id','?')}={m['value'][1]}")
    print("  gateway_up: " + " ".join(parts))
PY

  # 最近 1 分钟在线率（uptime）。
  local up_raw_1m
  up_raw_1m="$(curl -s "${VM_URL%/}/api/v1/query" --data-urlencode 'query=avg_over_time(gateway_up[1m])')"
  python3 - "${up_raw_1m}" <<'PY'
import json, sys
try:
    data = json.loads(sys.argv[1])["data"]["result"]
except Exception:
    data = []
if not data:
    print("  uptime(1m): (暂无数据)")
else:
    parts = []
    for m in sorted(data, key=lambda x: x["metric"].get("gateway_id", "")):
        parts.append(f"{m['metric'].get('gateway_id','?')}={m['value'][1]}")
    print("  uptime(1m): " + " ".join(parts))
PY

  echo "[PostgreSQL 快照（当前状态）]"
  local snap_raw
  snap_raw="$(curl -s "${CENTER_URL%/}/api/v1/admin/gateways/status")"
  python3 - "${snap_raw}" <<'PY'
import json, sys
try:
    statuses = json.loads(sys.argv[1])["statuses"]
except Exception:
    statuses = []
if not statuses:
    print("  (暂无上报)")
else:
    for s in sorted(statuses, key=lambda x: x.get("gateway_id", "")):
        print(f"  {s.get('gateway_id','?')} {s.get('status','?')}/{s.get('health','?')} last_seen={s.get('last_seen_at','?')}")
PY

  echo "[Agent 状态（gateway 上报其下 agent）]"
  IFS=',' read -r -a cred_pairs <<< "${GATEWAYS}"
  for entry in "${cred_pairs[@]:-}"; do
    [[ -z "${entry}" ]] && continue
    gateway_name="${entry%%:*}"
    local agents_raw
    agents_raw="$(curl -s "${CENTER_URL%/}/api/v1/admin/gateways/${gateway_name}/agents")"
    python3 - "${gateway_name}" "${agents_raw}" <<'PY'
import json, sys
gateway_id, raw = sys.argv[1], sys.argv[2]
try:
    agents = json.loads(raw)
except Exception:
    agents = []
if not agents:
    print(f"  {gateway_id}: (无 Agent 上报)")
else:
    parts = [f"{a['agent_id']}={a['status']}/{a['health']}" for a in agents]
    print(f"  {gateway_id}: " + " ".join(parts))
PY
  done
}

demo_loop() {
  echo "== 6. 演示数据流（每 ${DEMO_REFRESH}s 刷新）=="
  echo "  按 Ctrl+C 停止"
  if [[ "${SKIP_CENTER_WEB}" != "1" ]]; then
    echo "  管理页面：${CENTER_WEB_URL}（实时网关列表 / 状态卡片，走 vite 代理到 center）"
  fi
  echo
  local elapsed=0
  while true; do
    demo_tick
    echo
    if [[ "${DEMO_DURATION}" != "0" && "${elapsed}" -ge "${DEMO_DURATION}" ]]; then
      echo "演示时长 ${DEMO_DURATION}s 已到，结束。"
      break
    fi
    sleep "${DEMO_REFRESH}"
    elapsed=$((elapsed + DEMO_REFRESH))
  done
}

# ── 主流程 ──

require_cmd curl
require_cmd python3
require_cmd docker

echo "完整演示：WarpInsightCenter 网关状态 → PostgreSQL(快照) + VictoriaMetrics(时序)"
echo "  center: ${CENTER_URL}"
echo "  vm:     ${VM_URL}"
echo "  pg:     ${PG_URL}"
echo "  web:    ${CENTER_WEB_URL}（SKIP_CENTER_WEB=1 可跳过）"
echo "  gateways: ${GATEWAYS}"
echo "  上报间隔: ${REPORT_INTERVAL}s"
echo

ensure_infra
ensure_center
create_gateways
start_simulators
ensure_center_web
demo_loop
