#!/usr/bin/env bash
# 网关 install-init 过程验证：给定置备引导 Token（BOOTSTRAP_TOKEN），对已运行的
# WarpInsightCenter 依次完成「安装材料准备 → 初始化（initial-config）→ 注册（/register）→
# 运行期状态上报验证」。模拟网关侧 onboarding 的完整握手。
#
# 用法：
#   ./scripts/verify-gateway-init.sh <bootstrap_token> [gateway_id] [center_url]
#   BOOTSTRAP_TOKEN=xxx ./scripts/verify-gateway-init.sh
#
# 可覆盖 env：CENTER_URL / GATEWAY_ID / IDENTITY_TOKEN / IMAGE
#   IDENTITY_TOKEN：网关自生成身份（X-Gateway-Identity-Token），默认 sim-identity-<gateway_id>。
#
# 前置：warp-insight-center 已运行，且该 gateway 已创建（create 时签发 bootstrap token）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
RUN_DIR="${REPO_ROOT}/.run"
mkdir -p "${RUN_DIR}"

CENTER_URL="${CENTER_URL:-${3:-http://127.0.0.1:3100}}"
GATEWAY_ID="${GATEWAY_ID:-${2:-gw-demo}}"
INSTANCE_ID="${INSTANCE_ID:-inst-${GATEWAY_ID#gw-}}"
BOOTSTRAP_TOKEN="${1:-${BOOTSTRAP_TOKEN:-}}"
IDENTITY_TOKEN="${IDENTITY_TOKEN:-sim-identity-${GATEWAY_ID}}"
IMAGE="${IMAGE:-warp-gateway:latest}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}
require_cmd curl
require_cmd python3

# 交互输入：未提供 token（参数/env 均无）时提示并等待用户输入。
if [[ -z "${BOOTSTRAP_TOKEN}" ]]; then
  read -r -p "请输入置备引导 Token（bootstrap token）: " BOOTSTRAP_TOKEN
fi
if [[ -z "${BOOTSTRAP_TOKEN}" ]]; then
  echo "bootstrap token 不能为空" >&2
  exit 1
fi
# 未提供 gateway_id / center_url 时交互提示（含默认值）；token 必须属于该网关。
if [[ -z "${2:-}" ]]; then
  read -r -p "网关 ID [${GATEWAY_ID}]: " _gw
  [[ -n "${_gw}" ]] && GATEWAY_ID="${_gw}"
fi
if [[ -z "${3:-}" ]]; then
  read -r -p "中心地址 [${CENTER_URL}]: " _cu
  [[ -n "${_cu}" ]] && CENTER_URL="${_cu}"
fi

now_rfc3339() {
  python3 -c 'from datetime import datetime, timezone; print(datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))'
}

init_url="${CENTER_URL%/}/api/v1/gateway/initial-config?instance_id=${GATEWAY_ID}"
instance_id="${INSTANCE_ID}"

echo "== 1. 安装材料准备（docker run，bootstrap 经 env 注入，不进 URL） =="
cat <<EOF
  init_url: ${init_url}
  docker run -d --name warp-gateway-${GATEWAY_ID} \\
    -e WARP_GATEWAY_INIT_URL="${init_url}" \\
    -e WARP_GATEWAY_BOOTSTRAP_TOKEN="${BOOTSTRAP_TOKEN}" \\
    ${IMAGE}
EOF

echo "== 2. 初始化：GET initial-config（Bearer bootstrap + X-Gateway-Identity-Token） =="
resp_tmp="$(mktemp)"
http_code="$(curl -s -o "${resp_tmp}" -w '%{http_code}' \
  "${init_url}" \
  -H "Authorization: Bearer ${BOOTSTRAP_TOKEN}" \
  -H "X-Gateway-Identity-Token: ${IDENTITY_TOKEN}")"
if [[ "${http_code}" != "200" ]]; then
  if [[ "${http_code}" == "429" ]]; then
    echo "  中心限流（429）：失败鉴权累积过多。请重启 warp-insight-center 清空限流，或用未 onboard 的新网关重试。" >&2
  else
    echo "  初始化失败：HTTP ${http_code} body=$(cat "${resp_tmp}")" >&2
    echo "  提示：该网关可能已 onboard（bootstrap 已消费）；请用新网关或新 bootstrap 重试。" >&2
  fi
  rm -f "${resp_tmp}"
  exit 1
fi
# 中心 initial-config 返回 JSON：{ config, regist_token }。解析 regist_token 并按 JSON 重建 config.toml。
gateway_dir="${RUN_DIR}/gateways/${GATEWAY_ID}/${INSTANCE_ID}"
mkdir -p "${gateway_dir}"
config_file="${gateway_dir}/config.toml"
regist_token="$(python3 - "${resp_tmp}" "${config_file}" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))
regist = data.get("regist_token") or ""
cfg = data.get("config", {})
lines = [
    "version = 1", "",
    "[control_center]",
    f"endpoint = \"{cfg.get('control_center_endpoint','')}\"",
    "trust_bundle = \"/etc/warp-gateway/ca/control-center.pem\"",
    f"server_tls_required = {str(cfg.get('server_tls_required', False)).lower()}",
    "",
    "[enrollment]",
    f"token_id = \"{cfg.get('enrollment_token_id','')}\"",
]
if regist:
    lines.append(f"token = \"{regist}\"")
lines += ["", "[protocol]", f"version = \"{cfg.get('protocol_version','1.0')}\"", ""]
open(sys.argv[2], "w").write("\n".join(lines))
print(regist)
PY
)"
rm -f "${resp_tmp}"
if [[ -z "${regist_token}" ]]; then
  echo "  初始化成功，但响应未解析到 RegistToken" >&2
  exit 1
fi
echo "  初始化成功：RegistToken 已派生（${#regist_token} 字符）"
echo "  config.toml 已保存：${config_file}"

echo "== 3. 注册：POST /register（RegistToken 一次性消费 → RUNTIME_TOKEN） =="
register_tmp="$(mktemp)"
http_code="$(curl -s -o "${register_tmp}" -w '%{http_code}' \
  -X POST "${CENTER_URL%/}/api/v1/gateway/register" \
  -H 'content-type: application/json' \
  -d "{\"enrollment_token\":\"${regist_token}\",\"instance_id\":\"${instance_id}\",\"requested_at\":\"$(now_rfc3339)\"}")"
if [[ "${http_code}" != "200" ]]; then
  echo "  注册失败：HTTP ${http_code} body=$(cat "${register_tmp}")" >&2
  rm -f "${register_tmp}"
  exit 1
fi
runtime_token="$(python3 - "${register_tmp}" <<'PY'
import json, sys
try:
    data = json.load(open(sys.argv[1]))
    print(data["result"]["credential_bundle"]["bearer_token"])
except Exception as exc:
    sys.stderr.write(f"register 响应解析失败: {exc}\n")
PY
)"
rm -f "${register_tmp}"
if [[ -z "${runtime_token}" ]]; then
  echo "  注册成功，但未解析到 RUNTIME_TOKEN" >&2
  exit 1
fi
echo "  注册成功：RUNTIME_TOKEN 已签发（${#runtime_token} 字符）"
# 保存到固定路径，供 demo-gateway.sh（后续）自动读取，无需手工输入。
runtime_token_file="${gateway_dir}/runtime-token"
printf '%s' "${runtime_token}" > "${runtime_token_file}"
echo "  RUNTIME_TOKEN = ${runtime_token}"
echo "  已保存：${runtime_token_file}（demo-gateway.sh 会自动读取）"

# 生成网关自管配置（warp-gateway.toml，含 admin_api_token）并并入 config.toml。
echo "== 4. 生成网关自管配置并并入 config.toml（含 admin_api_token） =="
gw_bin="${REPO_ROOT}/target/debug/warp-gateway"
if [[ ! -x "${gw_bin}" ]]; then
  require_cmd cargo
  cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" -p warp-gateway >/dev/null
fi
"${gw_bin}" init-config "${gateway_dir}/warp-gateway.toml" >/dev/null 2>&1 || true
if [[ -f "${gateway_dir}/warp-gateway.toml" ]]; then
  {
    echo
    cat "${gateway_dir}/warp-gateway.toml"
  } >> "${config_file}"
  echo "  admin_api_token 已并入：${config_file}"
fi

echo "== 4. 验证：POST /status（用 RUNTIME_TOKEN 上报运行状态） =="
status_code="$(curl -s -o /dev/null -w '%{http_code}' \
  -X POST "${CENTER_URL%/}/api/v1/gateway/status" \
  -H "Authorization: Bearer ${runtime_token}" \
  -H 'content-type: application/json' \
  -d "{\"gateway_id\":\"${GATEWAY_ID}\",\"instance_id\":\"${instance_id}\",\"version\":\"v2.4.1\",\"status\":\"online\",\"health\":\"healthy\",\"memory_bytes\":2147483648,\"cpu_percent\":35.0,\"reported_at\":\"$(now_rfc3339)\"}")"
if [[ "${status_code}" != "200" ]]; then
  echo "  状态上报验证失败：HTTP ${status_code}" >&2
  exit 1
fi
echo "  状态上报验证成功：HTTP 200"

echo
echo "== 完成：安装 → 初始化 → 注册 → 运行期上报 全链验证通过 =="
