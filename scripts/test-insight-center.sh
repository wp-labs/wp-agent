#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# WarpInsightCenter 纯 HTTP（无 TLS），默认 127.0.0.1:3100，env 配置。
CENTER_BASE_URL="${CENTER_BASE_URL:-http://127.0.0.1:3100}"
# 默认空 = dev 模式（中心不要求鉴权，前端免填 Admin Token 即可看真实数据）。
# 设置 ADMIN_API_TOKEN 则中心要求鉴权，脚本会额外校验 401 路径。
ADMIN_API_TOKEN="${ADMIN_API_TOKEN:-}"
GATEWAY_CREDENTIALS="${GATEWAY_CREDENTIALS:-gw-001:tok-a,gw-002:tok-b}"
# 开发期 PostgreSQL：设置 WARP_INSIGHT_CENTER_DATABASE_URL 则中心用 PgStore，
# 脚本启动中心前先检查数据库就绪；不设置则回退 JSON 文件 store（原行为）。
DATABASE_URL="${WARP_INSIGHT_CENTER_DATABASE_URL:-}"
CENTER_WEB_BASE_URL="${CENTER_WEB_BASE_URL:-http://127.0.0.1:5173}"
SKIP_CENTER_WEB="${SKIP_CENTER_WEB:-0}"

# 测试工作区放在仓库内 .run/test/，便于查找；仓库目录建不了则退回系统临时目录。
TEST_RUN_DIR="${REPO_ROOT}/.run/test"
if ! mkdir -p "${TEST_RUN_DIR}" 2>/dev/null; then
  TEST_RUN_DIR="${TMPDIR:-/tmp}"
fi
TMP_ROOT="$(mktemp -d "${TEST_RUN_DIR}/wist-center.XXXXXX")"
STORE_PATH="${TMP_ROOT}/center-store.json"
CENTER_LOG="${TMP_ROOT}/wist-center.log"
CENTER_WEB_LOG="${TMP_ROOT}/center-web.log"
RESPONSE_JSON="${TMP_ROOT}/response.json"
CENTER_PID=""
CENTER_STARTED_BY_SCRIPT=0
CENTER_WEB_PID=""
STOP_STARTED_SERVICES="${STOP_STARTED_SERVICES:-0}"
WAIT_FOR_EXIT="${WAIT_FOR_EXIT:-1}"

cleanup() {
  if [[ -n "${CENTER_WEB_PID}" ]] && kill -0 "${CENTER_WEB_PID}" 2>/dev/null; then
    kill "${CENTER_WEB_PID}" 2>/dev/null || true
    wait "${CENTER_WEB_PID}" 2>/dev/null || true
  fi
  CENTER_WEB_PID=""
  if [[ -n "${CENTER_PID}" ]] && kill -0 "${CENTER_PID}" 2>/dev/null; then
    kill "${CENTER_PID}" 2>/dev/null || true
    wait "${CENTER_PID}" 2>/dev/null || true
  fi
  CENTER_PID=""
  if [[ "${KEEP_WARP_INSIGHT_CENTER_TEST:-0}" != "1" && -z "${CENTER_PID}${CENTER_WEB_PID}" ]]; then
    rm -rf "${TMP_ROOT}"
  else
    echo "kept test workspace: ${TMP_ROOT}"
  fi
}
trap cleanup EXIT

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

center_listen_addr() {
  python3 - "$CENTER_BASE_URL" <<'PY'
import sys
from urllib.parse import urlparse

url = urlparse(sys.argv[1])
if url.scheme != "http":
    raise SystemExit(f"CENTER_BASE_URL must start with http://: {sys.argv[1]}")
if not url.hostname:
    raise SystemExit(f"CENTER_BASE_URL is missing hostname: {sys.argv[1]}")
port = url.port if url.port is not None else 80
host = url.hostname
# 环回保持环回；其它 host 绑 0.0.0.0 以便按 URL 的网络地址可达。
if host in {"127.0.0.1", "localhost"}:
    print(f"{host}:{port}")
else:
    print(f"0.0.0.0:{port}")
PY
}

# 管理面探测：dev 模式（无 token）→ 200；鉴权模式 → 401；两者都视为服务就绪。
gateway_list_status() {
  curl -s \
    -o "${RESPONSE_JSON}" \
    -w "%{http_code}" \
    "${CENTER_BASE_URL%/}/api/v1/admin/gateways" || true
}

center_ready() {
  local status
  status="$(gateway_list_status)"
  [[ "${status}" == "200" || "${status}" == "401" ]]
}

# 从 DATABASE_URL 解析 host/port（TCP 探测兜底用）。
database_host_port() {
  python3 - "${DATABASE_URL}" <<'PY'
import sys
from urllib.parse import urlparse

url = urlparse(sys.argv[1])
if not url.hostname:
    raise SystemExit(f"DATABASE_URL missing hostname: {sys.argv[1]}")
port = url.port if url.port is not None else 5432
print(url.hostname)
print(port)
PY
}

# PG 是否就绪：优先 pg_isready（返回 0=accepting，1=rejecting，2=不可达），
# 没有 pg_isready 则退化为 TCP 端口连通性探测。
database_ready() {
  if command -v pg_isready >/dev/null 2>&1; then
    pg_isready -d "${DATABASE_URL}" >/dev/null 2>&1
    return $?
  fi
  local host
  local port
  host="$(database_host_port | sed -n '1p')"
  port="$(database_host_port | sed -n '2p')"
  python3 - "${host}" "${port}" <<'PY'
import socket
import sys

host, port = sys.argv[1], int(sys.argv[2])
try:
    with socket.create_connection((host, port), timeout=1):
        sys.exit(0)
except OSError:
    sys.exit(1)
PY
}

# 启动中心前确认数据库就绪（initdb 首次建表可能耗时，轮询等待）。
ensure_database() {
  if [[ -z "${DATABASE_URL}" ]]; then
    echo "WARP_INSIGHT_CENTER_DATABASE_URL not set; using JSON file store"
    return
  fi
  echo "checking database readiness at ${DATABASE_URL}"
  for _ in {1..150}; do
    if database_ready; then
      echo "database is ready"
      return
    fi
    sleep 0.2
  done
  echo "database is not reachable: ${DATABASE_URL}" >&2
  echo "start it with: docker compose -f ${REPO_ROOT}/crates/wist-center/docker-compose.yml up -d postgres" >&2
  exit 1
}

start_center() {
  local listen_addr
  listen_addr="$(center_listen_addr)"

  # 用 PgStore 时先确认 PG 就绪，避免中心启动即因连接失败退出。
  ensure_database

  require_cmd cargo
  echo "center service is not running; building and starting wist-center..."
  cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" -p wist-center

  (
    cd "${REPO_ROOT}"
    # 网关不再靠启动时 seed（不传 WARP_INSIGHT_CENTER_GATEWAY_CREDENTIALS），
    # 由 ensure_gateway_instances 通过接口创建。
    exec nohup env \
      WARP_INSIGHT_CENTER_LISTEN="${listen_addr}" \
      WARP_INSIGHT_CENTER_STORE_PATH="${STORE_PATH}" \
      WARP_INSIGHT_CENTER_DATABASE_URL="${DATABASE_URL}" \
      WARP_INSIGHT_CENTER_ADMIN_TOKEN="${ADMIN_API_TOKEN}" \
      "${REPO_ROOT}/target/debug/wist-center" \
      >"${CENTER_LOG}" 2>&1
  ) &
  CENTER_PID=$!

  CENTER_STARTED_BY_SCRIPT=1
  for _ in {1..100}; do
    if ! kill -0 "${CENTER_PID}" 2>/dev/null; then
      echo "wist-center failed to start; log:" >&2
      cat "${CENTER_LOG}" >&2
      exit 1
    fi
  if center_ready; then
    echo "started wist-center pid=${CENTER_PID}"
    return
  fi
  sleep 0.2
done

echo "wist-center did not become ready; log:" >&2
cat "${CENTER_LOG}" >&2
exit 1
}

ensure_center() {
  local status
  status="$(gateway_list_status)"

  if [[ "${status}" == "000" ]]; then
    start_center
    status="$(gateway_list_status)"
  fi
  if ! center_ready; then
    echo "center gateway-list endpoint check failed: ${CENTER_BASE_URL%/}/api/v1/admin/gateways returned ${status}." >&2
    exit 1
  fi
}

center_web_listen_options() {
  python3 - "$CENTER_WEB_BASE_URL" <<'PY'
import sys
from urllib.parse import urlparse

url = urlparse(sys.argv[1])
if url.scheme not in {"http", "https"}:
    raise SystemExit(f"CENTER_WEB_BASE_URL must start with http:// or https://: {sys.argv[1]}")
if not url.hostname:
    raise SystemExit(f"CENTER_WEB_BASE_URL is missing hostname: {sys.argv[1]}")
port = url.port if url.port is not None else (443 if url.scheme == "https" else 80)
host = url.hostname
if host in {"127.0.0.1", "localhost"}:
    print(host)
else:
    print("0.0.0.0")
print(port)
PY
}

center_web_status() {
  curl -s -o /dev/null -w "%{http_code}" "${CENTER_WEB_BASE_URL%/}/" || true
}

start_center_web_service() {
  local host
  local port

  host="$(center_web_listen_options | sed -n '1p')"
  port="$(center_web_listen_options | sed -n '2p')"

  require_cmd npm
  if [[ ! -d "${REPO_ROOT}/crates/center-web/node_modules" ]]; then
    echo "center-web dependencies are missing: crates/center-web/node_modules" >&2
    echo "run npm install in crates/center-web before running this script." >&2
    exit 1
  fi

  echo "center-web is not running; starting center-web..."
  (
    cd "${REPO_ROOT}/crates/center-web"
    exec nohup npm run dev -- --host "${host}" --port "${port}" --strictPort \
      >"${CENTER_WEB_LOG}" 2>&1
  ) &
  CENTER_WEB_PID=$!

  for _ in {1..100}; do
    if ! kill -0 "${CENTER_WEB_PID}" 2>/dev/null; then
      echo "center-web failed to start; log:" >&2
      cat "${CENTER_WEB_LOG}" >&2
      exit 1
    fi
    if [[ "$(center_web_status)" == "200" ]]; then
      echo "started center-web pid=${CENTER_WEB_PID}"
      echo "center web url: ${CENTER_WEB_BASE_URL}"
      return
    fi
    sleep 0.1
  done

  echo "center-web did not become ready; log:" >&2
  cat "${CENTER_WEB_LOG}" >&2
  exit 1
}

ensure_center_web_service() {
  local status
  status="$(center_web_status)"

  if [[ "${status}" == "000" ]]; then
    start_center_web_service
    status="$(center_web_status)"
  fi
  if [[ "${status}" != "200" ]]; then
    echo "center-web endpoint check failed: ${CENTER_WEB_BASE_URL%/}/ returned ${status}." >&2
    exit 1
  fi
}

# 前端 vite proxy → 后端：带 Admin Token 通过前端端口拉网关列表，验证全链路。
check_center_web_proxy() {
  local status
  status="$(curl -s -o "${RESPONSE_JSON}" -w "%{http_code}" \
    -H "authorization: Bearer ${ADMIN_API_TOKEN}" \
    "${CENTER_WEB_BASE_URL%/}/api/v1/admin/gateways")"
  if [[ "${status}" != "200" ]]; then
    echo "center-web proxy to backend failed: ${CENTER_WEB_BASE_URL%/}/api/v1/admin/gateways returned ${status}." >&2
    exit 1
  fi
  validate_gateway_list
  echo "center-web proxy → backend gateway list ok"
}

# ── 创建网关实例（通过接口，替代启动时 seed）──

# POST /api/v1/admin/gateways/instances，对齐前端 createGatewayInstance 契约。
create_gateway_instance() {
  local gateway_name="$1"
  local token="$2"
  curl -s -o "${RESPONSE_JSON}" -w "%{http_code}" \
    -X POST "${CENTER_BASE_URL%/}/api/v1/admin/gateways/instances" \
    -H "authorization: Bearer ${ADMIN_API_TOKEN}" \
    -H "content-type: application/json" \
    -d "{\"gateway_name\":\"${gateway_name}\",\"requested_by\":\"test-script\",\"token\":\"${token}\"}" || true
}

# 解析 GATEWAY_CREDENTIALS（"gw-001:tok-a,gw-002:tok-b"）逐个创建；
# 201=新建 / 409=已存在（上次残留），两者都视为就绪继续；其它视为失败。
ensure_gateway_instances() {
  echo "creating gateway instances via API..."
  IFS=',' read -r -a credential_pairs <<< "${GATEWAY_CREDENTIALS}"
  for entry in "${credential_pairs[@]}"; do
    entry="$(printf '%s' "${entry}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [[ -z "${entry}" ]] && continue
    gateway_name="${entry%%:*}"
    token="${entry#*:}"
    if [[ -z "${gateway_name}" || -z "${token}" ]]; then
      echo "invalid gateway credential entry: ${entry} (expected gateway_name:token)" >&2
      exit 1
    fi
    status="$(create_gateway_instance "${gateway_name}" "${token}")"
    if [[ "${status}" != "201" && "${status}" != "409" ]]; then
      echo "create gateway instance ${gateway_name} returned ${status}" >&2
      cat "${RESPONSE_JSON}" >&2
      exit 1
    fi
    echo "gateway instance ready: ${gateway_name} (${status})"
  done
}

# ── 接收链路：网关上报状态 ──

report_gateway_status() {
  local gateway_id="$1"
  local token="$2"
  local status="$3"
  local health="$4"
  curl -s -o "${RESPONSE_JSON}" -w "%{http_code}" \
    -X POST "${CENTER_BASE_URL%/}/api/v1/gateway/status" \
    -H "authorization: Bearer ${token}" \
    -H "content-type: application/json" \
    -d "{\"gateway_id\":\"${gateway_id}\",\"instance_id\":\"inst-${gateway_id#gw-}\",\"version\":\"v2.4.1\",\"status\":\"${status}\",\"health\":\"${health}\",\"reported_at\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" || true
}

# ── 管理面读取：网关列表 ──

admin_get() {
  local path="$1"
  curl -s -o "${RESPONSE_JSON}" -w "%{http_code}" \
    -H "authorization: Bearer ${ADMIN_API_TOKEN}" \
    "${CENTER_BASE_URL%/}${path}" || true
}

# ── 校验 ──

validate_gateway_list() {
  python3 - "${RESPONSE_JSON}" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
lst = payload.get("list") or payload
required = ["gateway_count", "online_count", "offline_count", "degraded_count"]
missing = [k for k in required if k not in lst]
if missing:
    raise SystemExit(f"gateway list missing fields: {missing}")
if lst["gateway_count"] < 2:
    raise SystemExit(f"expected >= 2 gateways, got {lst['gateway_count']}")
print("gateway list ok:")
print(json.dumps(lst, indent=2, sort_keys=True))
PY
}

validate_gateway_status_list() {
  local expected_gateway="$1"
  python3 - "${RESPONSE_JSON}" "${expected_gateway}" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
statuses = payload.get("statuses") or payload
if not isinstance(statuses, list):
    raise SystemExit(f"gateway status list is not an array: {payload}")
ids = [s.get("gateway_id") for s in statuses]
if sys.argv[2] not in ids:
    raise SystemExit(f"expected reported gateway {sys.argv[2]} in status list, got {ids}")
print(f"gateway status list ok: {ids}")
PY
}

validate_stored_gateway() {
  local store_file="$1"
  local gateway_id="$2"
  python3 - "${store_file}" "${gateway_id}" <<'PY'
import json
import pathlib
import sys

snapshot = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
gw = (snapshot.get("gateways") or {}).get(sys.argv[2])
if not gw:
    raise SystemExit(f"gateway {sys.argv[2]} not found in center store")
if not gw.get("version") or not gw.get("status") or not gw.get("health") or not gw.get("last_seen_at"):
    raise SystemExit(f"gateway {sys.argv[2]} missing reported status fields: {gw}")
if not gw.get("credential_token_hash"):
    raise SystemExit(f"gateway {sys.argv[2]} missing credential hash")
print("stored gateway ok:")
print(json.dumps({
    "gateway_id": gw["gateway_id"],
    "status": gw.get("status"),
    "health": gw.get("health"),
    "version": gw.get("version"),
}, indent=2, sort_keys=True))
PY
}

require_cmd curl
require_cmd python3

echo "testing wist-center against ${CENTER_BASE_URL}"
echo "admin api token: ${ADMIN_API_TOKEN}"
echo "gateway credentials: ${GATEWAY_CREDENTIALS}"
if [[ -n "${DATABASE_URL}" ]]; then
  echo "database: ${DATABASE_URL} (PostgreSQL store)"
else
  echo "database: not set (JSON file store: ${STORE_PATH})"
fi
echo "workspace: ${TMP_ROOT}"

echo "checking center service..."
ensure_center

ensure_gateway_instances

if [[ "${SKIP_CENTER_WEB}" != "1" ]]; then
  echo "checking center-web service..."
  ensure_center_web_service
  echo "checking center-web proxy → backend..."
  check_center_web_proxy
fi

echo "checking gateway status report (valid credential)..."
status="$(report_gateway_status gw-001 tok-a online healthy)"
if [[ "${status}" != "200" ]]; then
  echo "gateway status report returned ${status}" >&2
  cat "${RESPONSE_JSON}" >&2
  exit 1
fi
python3 - "${RESPONSE_JSON}" <<'PY'
import json, pathlib, sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
receipt = payload.get("receipt") or {}
if receipt.get("gateway_id") != "gw-001":
    raise SystemExit(f"unexpected receipt: {payload}")
if not receipt.get("accepted_at"):
    raise SystemExit(f"receipt missing accepted_at: {payload}")
print(f"status report accepted: gateway_id={receipt['gateway_id']}")
PY

echo "checking gateway status report (invalid credential → 401)..."
status="$(report_gateway_status gw-001 wrong-token online healthy)"
if [[ "${status}" != "401" ]]; then
  echo "invalid credential should be rejected with 401, got ${status}" >&2
  exit 1
fi

echo "checking second gateway report (gw-002 offline/degraded)..."
status="$(report_gateway_status gw-002 tok-b offline degraded)"
if [[ "${status}" != "200" ]]; then
  echo "gw-002 status report returned ${status}" >&2
  exit 1
fi

echo "checking gateway list aggregate..."
status="$(admin_get /api/v1/admin/gateways)"
if [[ "${status}" != "200" ]]; then
  echo "gateway list returned ${status}" >&2
  cat "${RESPONSE_JSON}" >&2
  exit 1
fi
validate_gateway_list

echo "checking gateway status card list..."
status="$(admin_get /api/v1/admin/gateways/status)"
if [[ "${status}" != "200" ]]; then
  echo "gateway status list returned ${status}" >&2
  exit 1
fi
validate_gateway_status_list gw-001

echo "checking single gateway status..."
status="$(admin_get /api/v1/admin/gateways/gw-001/status)"
if [[ "${status}" != "200" ]]; then
  echo "single gateway status returned ${status}" >&2
  exit 1
fi

echo "checking unknown gateway → 404..."
status="$(admin_get /api/v1/admin/gateways/gw-999/status)"
if [[ "${status}" != "404" ]]; then
  echo "unknown gateway should return 404, got ${status}" >&2
  exit 1
fi

if [[ -n "${ADMIN_API_TOKEN}" ]]; then
  echo "checking admin auth (missing token → 401)..."
  status="$(curl -s -o /dev/null -w "%{http_code}" "${CENTER_BASE_URL%/}/api/v1/admin/gateways")"
  if [[ "${status}" != "401" ]]; then
    echo "missing admin token should return 401, got ${status}" >&2
    exit 1
  fi
else
  echo "dev mode (no ADMIN_API_TOKEN): skipping admin auth check"
fi

if [[ "${CENTER_STARTED_BY_SCRIPT}" == "1" && -n "${DATABASE_URL}" ]]; then
  echo "using PostgreSQL store; reported status already validated via admin API (skip JSON store file)"
elif [[ "${CENTER_STARTED_BY_SCRIPT}" == "1" ]]; then
  echo "validating stored gateway state..."
  validate_stored_gateway "${STORE_PATH}" gw-001
else
  echo "reusing an externally running center; skipping store validation (unknown store path)"
fi

echo "wist-center test passed"
echo "center api url: ${CENTER_BASE_URL}"
if [[ "${SKIP_CENTER_WEB}" != "1" ]]; then
  echo "center web url: ${CENTER_WEB_BASE_URL}"
fi
if [[ -n "${CENTER_PID}" ]]; then
  echo "started center pid: ${CENTER_PID}"
  echo "center log: ${CENTER_LOG}"
  echo "store: ${STORE_PATH}"
fi
if [[ -n "${CENTER_WEB_PID}" ]]; then
  echo "started center-web pid: ${CENTER_WEB_PID}"
  echo "center-web log: ${CENTER_WEB_LOG}"
fi
if [[ "${WAIT_FOR_EXIT}" == "1" && -t 0 ]]; then
  echo ""
  echo "全部验证通过。中心与 WEB 持续运行中。"
  echo "工作区: ${TMP_ROOT}"
  echo "页面:   ${CENTER_WEB_BASE_URL}  （导航栏填 Admin Token 查看真实网关列表）"
  echo "按回车退出并清理所有服务..."
  read -r _ || true
  echo "收到退出，清理中..."
  STOP_STARTED_SERVICES=1
else
  if [[ -n "${CENTER_PID}${CENTER_WEB_PID}" && "${STOP_STARTED_SERVICES}" != "1" ]]; then
    echo "services started by this script are still running."
    echo "stop them manually with: kill ${CENTER_WEB_PID:-} ${CENTER_PID:-}"
    echo "or run with STOP_STARTED_SERVICES=1 to clean them up automatically."
  fi
fi
