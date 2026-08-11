#!/usr/bin/env bash
# 生成 WarpInsightCenter 控制面 CA 与服务器证书
#
# 产物：
#   control-center.pem        信任根（CA 证书，公开）→ 作为 Gateway 的
#                             control_center.trust_bundle 分发（config.toml）
#   control-center-server.key 中心服务器私钥（机密，留在控制中心）
#   control-center-server.crt 中心 TLS 服务器证书（由 CA 签发）
#
# 用法：
#   ./generate-control-center-cert.sh [输出目录] [服务端 CN]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 默认输出到 ~/.warpinsight-center/ca（本地开发）；服务器部署可用绝对路径覆盖
OUT_DIR="${1:-${HOME}/.warpinsight-center/ca}"
SERVER_CN="${2:-control-center.warp-insight.local}"
CA_CN="${CA_CN:-WarpInsight Control Center CA}"
DAYS="${DAYS:-3650}"

command -v openssl >/dev/null 2>&1 || { echo "错误: 需要 openssl" >&2; exit 1; }

mkdir -p "${OUT_DIR}"
# 展示用绝对路径（支持相对/绝对 OUT_DIR）
DISPLAY_DIR="$(cd "${OUT_DIR}" && pwd)"

echo "==> [1/3] 生成 CA 私钥 + 自签信任根（control-center.pem）"
openssl genrsa -out "${OUT_DIR}/control-center-ca.key" 4096
openssl req -x509 -new -key "${OUT_DIR}/control-center-ca.key" \
  -sha256 -days "${DAYS}" \
  -subj "/CN=${CA_CN}" \
  -out "${OUT_DIR}/control-center.pem"

echo "==> [2/3] 生成控制中心服务器私钥 + CSR"
openssl genrsa -out "${OUT_DIR}/control-center-server.key" 4096
openssl req -new -key "${OUT_DIR}/control-center-server.key" \
  -subj "/CN=${SERVER_CN}" \
  -out "${OUT_DIR}/control-center-server.csr"

echo "==> [3/3] 用 CA 签发中心服务器证书"
openssl x509 -req -in "${OUT_DIR}/control-center-server.csr" \
  -CA "${OUT_DIR}/control-center.pem" \
  -CAkey "${OUT_DIR}/control-center-ca.key" \
  -CAcreateserial -days "${DAYS}" -sha256 \
  -out "${OUT_DIR}/control-center-server.crt"

rm -f "${OUT_DIR}/control-center-server.csr"

echo
echo "生成完成："
echo "  信任根（分发给所有 Gateway，作为 control_center.trust_bundle）:"
echo "    ${DISPLAY_DIR}/control-center.pem"
echo "  控制中心 TLS 服务器证书:"
echo "    ${DISPLAY_DIR}/control-center-server.crt"
echo "  控制中心服务器私钥（机密，只留在中心，勿分发）:"
echo "    ${DISPLAY_DIR}/control-center-server.key"
echo
echo "部署提示："
echo "  - control-center.pem 由部署时预置到各 Gateway（config.toml trust_bundle 路径）"
echo "  - 服务器证书+私钥配置到控制中心 HTTPS 服务"
echo "  - 如需 mTLS，可用本 CA 为每个 Gateway 签发客户端证书"
