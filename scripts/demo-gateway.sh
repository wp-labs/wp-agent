#!/usr/bin/env bash
# 网关初始化演示（持续运行）：起 warp-insight-center（FileStore）+ center-web →
# 创建网关 → 获取 init_url → curl 验证 → 网关注册 → simulator 持续上报 →
# 持续刷新展示网关状态，直到 Ctrl+C。
#
# 用法：
#   ./scripts/demo-gateway.sh                    # 默认 gw-demo / tok-demo
#   ./scripts/demo-gateway.sh gw-prod tok-xxxx   # 自定义网关名与凭证
#   DEMO_DURATION=30 ./scripts/demo-gateway.sh   # 跑 30 秒后自动退出
#
# 可覆盖 env：CENTER_URL / WEB_URL / GATEWAY_NAME / TOKEN / REPORT_INTERVAL / DEMO_DURATION / DEMO_REFRESH / SKIP_CENTER_WEB
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CENTER_BIN="${REPO_ROOT}/target/debug/warp-insight-center"
SIM_BIN="${REPO_ROOT}/target/debug/insight-simulator"

CENTER_URL="${CENTER_URL:-http://127.0.0.1:3200}"
# 独立 WEB dev 端口（5174），避免与 demo-insight-center 的 5173 共用环境。
WEB_URL="${WEB_URL:-http://127.0.0.1:5174}"
SKIP_CENTER_WEB="${SKIP_CENTER_WEB:-0}"
GATEWAY_NAME="${GATEWAY_NAME:-${1:-gw-demo}}"
TOKEN="${TOKEN:-${2:-tok-demo}}"
REPORT_INTERVAL="${REPORT_INTERVAL:-3}"
DEMO_DURATION="${DEMO_DURATION:-0}"   # 0 = 持续直到 Ctrl+C
DEMO_REFRESH="${DEMO_REFRESH:-5}"
STATE_DIR="$(mktemp -d)"

CENTER_STARTED_BY_SCRIPT=0
CENTER_PID=""
CENTER_WEB_PID=""
SIM_PID=""

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

cleanup() {
  if [[ -n "${SIM_PID}" ]] && kill -0 "${SIM_PID}" 2>/dev/null; then
    kill "${SIM_PID}" 2>/dev/null || true
    wait "${SIM_PID}" 2>/dev/null || true
  fi
  if [[ -n "${CENTER_WEB_PID}" ]] && kill -0 "${CENTER_WEB_PID}" 2>/dev/null; then
    kill "${CENTER_WEB_PID}" 2>/dev/null || true
    wait "${CENTER_WEB_PID}" 2>/dev/null || true
  fi
  if [[ "${CENTER_STARTED_BY_SCRIPT}" == "1" && -n "${CENTER_PID}" ]] && kill -0 "${CENTER_PID}" 2>/dev/null; then
    kill "${CENTER_PID}" 2>/dev/null || true
    wait "${CENTER_PID}" 2>/dev/null || true
  fi
  rm -rf "${STATE_DIR}"
  echo
  echo "演示结束，已清理 simulator / center-web / center 进程。"
}
trap cleanup EXIT

center_ready() {
  local code
  code="$(curl -s -o /dev/null -w "%{http_code}" "${CENTER_URL%/}/api/v1/admin/gateways" || true)"
  [[ "${code}" == "200" || "${code}" == "401" ]]
}

center_web_status() {
  curl -s -o /dev/null -w "%{http_code}" "${WEB_URL%/}/" || true
}

wait_until() {
  # wait_until <描述> <cmd...>
  local desc="$1"
  shift
  echo "等待 ${desc} 就绪..."
  for _ in {1..100}; do
    if "$@" >/dev/null 2>&1; then
      echo "  ${desc} 就绪"
      return 0
    fi
    sleep 0.2
  done
  echo "  ${desc} 未就绪，中止。" >&2
  exit 1
}

# ── 启动组件 ──

ensure_center() {
  echo "== 1. 启动 warp-insight-center（FileStore，dev 模式 admin 免鉴权）=="
  if center_ready; then
    echo "  center 已在运行（${CENTER_URL}），复用。"
    return
  fi
  require_cmd cargo
  cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" -p warp-insight-center >/dev/null
  host="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${CENTER_URL}'); print(u.hostname or '127.0.0.1')")"
  port="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${CENTER_URL}'); print(u.port or 80)")"
  WARP_INSIGHT_CENTER_LISTEN="${host}:${port}" \
  WARP_INSIGHT_CENTER_STORE_PATH="${STATE_DIR}/store.json" \
  WARP_INSIGHT_CENTER_PUBLIC_URL="${CENTER_URL}" \
    "${CENTER_BIN}" >/tmp/warp-gateway-demo-center.log 2>&1 &
  CENTER_PID=$!
  CENTER_STARTED_BY_SCRIPT=1
  wait_until "center" center_ready
}

ensure_gateway_web() {
  echo "== 5. 启动网关管理前端 gateway-web =="
  if [[ "${SKIP_CENTER_WEB}" == "1" ]]; then
    echo "  已跳过（SKIP_CENTER_WEB=1）"
    return
  fi
  if [[ "$(center_web_status)" == "200" ]]; then
    echo "  gateway-web 已在运行（${WEB_URL}），复用。"
    return
  fi
  require_cmd npm
  local web_dir="${REPO_ROOT}/crates/warp-gateway-web"
  if [[ ! -d "${web_dir}/node_modules" ]]; then
    echo "  gateway-web 依赖缺失：${web_dir}/node_modules（先 cd 到该目录执行 npm install）" >&2
    exit 1
  fi
  host="$(python3 -c "from urllib.parse import urlparse; print(urlparse('${WEB_URL}').hostname or '127.0.0.1')")"
  port="$(python3 -c "from urllib.parse import urlparse; print(urlparse('${WEB_URL}').port or 80)")"
  (
    cd "${web_dir}"
    exec nohup npm run dev -- --host "${host}" --port "${port}" --strictPort \
      >/tmp/warp-gateway-demo-web.log 2>&1
  ) &
  CENTER_WEB_PID=$!
  echo "  启动 gateway-web：${WEB_URL} (pid=$!)"
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
    echo "  gateway-web 就绪"
  else
    echo "  gateway-web 未就绪（日志 /tmp/warp-gateway-demo-web.log），跳过，不影响演示。"
    CENTER_WEB_PID=""
  fi
}

setup_gateway() {
  echo "== 2. 创建网关实例 =="
  code="$(curl -s -o "${STATE_DIR}/create.json" -w "%{http_code}" \
    -X POST "${CENTER_URL%/}/api/v1/admin/gateways/instances" \
    -H 'content-type: application/json' \
    -d "{\"gateway_name\":\"${GATEWAY_NAME}\",\"requested_by\":\"demo\",\"token\":\"${TOKEN}\"}")"
  if [[ "${code}" == "201" ]]; then
    echo "  创建成功 (201)"
  elif [[ "${code}" == "409" ]]; then
    echo "  已存在 (409，复用)"
  else
    echo "  创建失败: HTTP ${code}" >&2
    cat "${STATE_DIR}/create.json" >&2
    exit 1
  fi

  init_url="$(python3 - "${STATE_DIR}/create.json" <<'PY'
import json, sys
try:
    install = json.load(open(sys.argv[1]))["install"]
except Exception:
    install = {}
print(install.get("init_url", ""))
PY
)"
  if [[ -z "${init_url}" ]]; then
    echo "  ✗ 响应无 install.init_url（网关可能创建于无凭证时代，换名字重试）" >&2
    exit 1
  fi
  echo "  init_url = ${init_url}"
  echo "  init_curl = curl -H \"Authorization: Bearer ${TOKEN}\" \"${init_url}\""

  echo
  echo "== 3. curl 验证 init_url（Bearer 注册凭证）=="
  curl -s -w "\n  HTTP %{http_code}\n" \
    -H "Authorization: Bearer ${TOKEN}" "${init_url}"

  echo
  echo "== 4. 网关注册（POST /api/v1/gateway/register）=="
  now_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  register_code="$(curl -s -o "${STATE_DIR}/register.json" -w "%{http_code}" \
    -X POST "${CENTER_URL%/}/api/v1/gateway/register" \
    -H 'content-type: application/json' \
    -d "{\"enrollment_token\":\"${TOKEN}\",\"instance_id\":\"inst-${GATEWAY_NAME}\",\"requested_at\":\"${now_ts}\"}")"
  echo "  HTTP ${register_code}"
  python3 - "${STATE_DIR}/register.json" <<'PY'
import json, sys
try:
    data = json.load(open(sys.argv[1]))
    print(f"  {json.dumps(data, ensure_ascii=False, indent=2)}")
except Exception as e:
    print(f"  (响应解析失败: {e})")
PY
}

start_simulator() {
  echo "== 6. 启动 simulator（每 ${REPORT_INTERVAL}s 上报）=="
  require_cmd cargo
  if [[ ! -x "${SIM_BIN}" ]]; then
    cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" -p insight-simulator >/dev/null
  fi
  "${SIM_BIN}" gateway \
    --center-url "${CENTER_URL}" \
    --gateway-id "${GATEWAY_NAME}" --instance-id "inst-${GATEWAY_NAME}" \
    --token "${TOKEN}" --interval "${REPORT_INTERVAL}" \
    --report-agents \
    >"/tmp/warp-gateway-demo-sim-${GATEWAY_NAME}.log" 2>&1 &
  SIM_PID=$!
  echo "  simulator[${GATEWAY_NAME}] pid=$! → 每 ${REPORT_INTERVAL}s 上报"
}

# ── 持续循环：刷新网关状态，直到 Ctrl+C ──

demo_tick() {
  local now_ts
  now_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "----- ${now_ts} -----"

  local status_raw
  status_raw="$(curl -s "${CENTER_URL%/}/api/v1/admin/gateways/status")"
  python3 - "${status_raw}" <<'PY'
import json, sys
try:
    statuses = json.loads(sys.argv[1])["statuses"]
except Exception:
    statuses = []
if not statuses:
    print("  (尚无网关状态上报)")
else:
    for s in sorted(statuses, key=lambda x: x.get("gateway_id", "")):
        print(f"  {s.get('gateway_id','?')} {s.get('status','?')}/{s.get('health','?')} last_seen={s.get('last_seen_at','?')}")
PY

  local agents_raw
  agents_raw="$(curl -s "${CENTER_URL%/}/api/v1/admin/gateways/${GATEWAY_NAME}/agents")"
  python3 - "${agents_raw}" <<'PY'
import json, sys
try:
    agents = json.loads(sys.argv[1])
except Exception:
    agents = []
if not agents:
    print("  (无 Agent 上报)")
else:
    parts = [f"{a['agent_id']}={a['status']}/{a['health']}" for a in agents]
    print(f"  Agents: " + " ".join(parts))
PY
}

demo_loop() {
  echo
  echo "== 7. 持续演示（每 ${DEMO_REFRESH}s 刷新）=="
  if [[ "${SKIP_CENTER_WEB}" != "1" ]]; then
    echo "  管理页面：${WEB_URL}（网关列表 / 初始化 / 状态卡片）"
  fi
  echo "  按 Ctrl+C 停止"
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

echo "网关初始化演示（持续运行）"
echo "  center: ${CENTER_URL}"
echo "  web:    ${WEB_URL}（SKIP_CENTER_WEB=1 可跳过）"
echo "  gateway: ${GATEWAY_NAME} / token: ${TOKEN}"
echo "  上报间隔: ${REPORT_INTERVAL}s"
echo

ensure_center
setup_gateway
ensure_gateway_web
start_simulator
demo_loop
