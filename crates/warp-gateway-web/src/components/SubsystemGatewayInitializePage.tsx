import { useMemo, useState, type FormEvent } from "react";
import {
  ApiError,
  buildGatewayInitialConfigCurl,
  type GatewayInitialConfig,
} from "../api";
import { useGatewayInitialConfig } from "../hooks";
import { CopyButton } from "./CopyButton";
import { SubsystemAdminTopNavigation } from "./SubsystemAdminTopNavigation";
import styles from "./SubsystemGatewayInitializePage.module.css";

function errorMessage(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.status === 401) return "网关凭证无效或已失效，请核对 Center 创建回执。";
    if (error.status === 429) return "认证失败次数过多，请稍后再试。";
    return `控制中心返回 HTTP ${error.status}，请核对初始化 URL 和服务状态。`;
  }
  if (error instanceof TypeError) {
    return "无法访问控制中心。若 Gateway 页面与 Center 不同源，请检查网络和 CORS 配置。";
  }
  return "初始化配置响应不符合当前契约，请检查 Center 与 Gateway 版本。";
}

function ConfigResult({ config }: { config: GatewayInitialConfig }) {
  return (
    <section className={styles.result} aria-live="polite">
      <header className={styles.resultHeader}>
        <div>
          <div className={styles.successEyebrow}>配置获取成功</div>
          <h2 className={styles.sectionTitle}>Gateway 初始配置</h2>
        </div>
        <span className={styles.successBadge}>已验证</span>
      </header>
      <dl className={styles.resultGrid}>
        <div className={styles.resultItem}>
          <dt>控制中心端点</dt>
          <dd>{config.control_center_endpoint}</dd>
        </div>
        <div className={styles.resultItem}>
          <dt>协议版本</dt>
          <dd>{config.protocol_version}</dd>
        </div>
        <div className={styles.resultItem}>
          <dt>TLS 要求</dt>
          <dd>{config.server_tls_required ? "必须使用 TLS" : "允许非 TLS"}</dd>
        </div>
        <div className={styles.resultItem}>
          <dt>注册 Token 引用</dt>
          <dd>{config.enrollment_token_id || "未返回"}</dd>
        </div>
      </dl>
      {config.trust_bundle ? (
        <div className={styles.trustBundle}>
          <div className={styles.trustBundleHeader}>
            <div>
              <h3>控制中心信任根</h3>
              <p>
                Server Name：{config.trust_bundle.server_name} · Expected SAN：
                {config.trust_bundle.expected_san}
              </p>
            </div>
            <CopyButton
              text={config.trust_bundle.ca_bundle}
              label="复制 PEM"
              className={styles.secondaryButton}
            />
          </div>
          <pre className={styles.pemCode}>{config.trust_bundle.ca_bundle}</pre>
        </div>
      ) : (
        <p className={styles.notice}>控制中心未返回独立信任根。</p>
      )}
    </section>
  );
}

/**
 * Gateway 自身初始化页：消费 Center 创建实例时交付的 init_url 与网关凭证，
 * 只负责获取和展示 Gateway 初始配置，不处理 Agent 安装配置。
 */
export function SubsystemGatewayInitializePage() {
  const [initUrl, setInitUrl] = useState("");
  const [gatewayToken, setGatewayToken] = useState("");
  const initialConfig = useGatewayInitialConfig();
  const curl = useMemo(
    () => buildGatewayInitialConfigCurl(initUrl, gatewayToken),
    [gatewayToken, initUrl],
  );
  const canSubmit = initUrl.trim().length > 0 && gatewayToken.trim().length > 0;

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) return;
    initialConfig.mutate({ initUrl, gatewayToken });
  }

  return (
    <div className={styles.container}>
      <SubsystemAdminTopNavigation />
      <main className={styles.main}>
        <header className={styles.pageHeader}>
          <div className={styles.eyebrow}>Gateway Bootstrap</div>
          <h1 className={styles.pageTitle}>初始化 Gateway</h1>
          <p className={styles.pageSummary}>
            使用 WarpInsight Center 创建实例时交付的初始化 URL 与网关凭证，拉取该 Gateway 的初始连接配置。
          </p>
        </header>

        <div className={styles.workspace}>
          <section className={styles.formPanel}>
            <div className={styles.sectionHeading}>
              <span className={styles.step}>01</span>
              <div>
                <h2 className={styles.sectionTitle}>连接控制中心</h2>
                <p>两项内容均来自 Center 的 Gateway 实例创建回执。</p>
              </div>
            </div>
            <form className={styles.form} onSubmit={handleSubmit}>
              <label className={styles.field}>
                <span>Center 初始化 URL</span>
                <input
                  type="url"
                  value={initUrl}
                  onChange={(event) => setInitUrl(event.target.value)}
                  placeholder="https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo"
                  autoComplete="url"
                  required
                />
                <small>必须包含 Center 分配的 instance_id。</small>
              </label>
              <label className={styles.field}>
                <span>Gateway 注册凭证</span>
                <input
                  type="password"
                  value={gatewayToken}
                  onChange={(event) => setGatewayToken(event.target.value)}
                  placeholder="输入创建回执中的网关凭证"
                  autoComplete="off"
                  required
                />
                <small>凭证仅作为本次请求的 Bearer Header，不写入 URL。</small>
              </label>
              {initialConfig.isError ? (
                <div className={styles.errorBanner} role="alert">
                  {errorMessage(initialConfig.error)}
                </div>
              ) : null}
              <button
                type="submit"
                className={styles.primaryButton}
                disabled={!canSubmit || initialConfig.isPending}
              >
                {initialConfig.isPending ? "正在获取配置…" : "获取初始配置"}
              </button>
            </form>
          </section>

          <aside className={styles.commandPanel}>
            <div className={styles.sectionHeading}>
              <span className={styles.step}>02</span>
              <div>
                <h2 className={styles.sectionTitle}>终端调用</h2>
                <p>也可以复制等价 curl 命令在 Gateway 主机上验证。</p>
              </div>
            </div>
            <div className={styles.codeHeader}>
              <span>bash</span>
              <CopyButton
                text={curl}
                label="复制命令"
                copiedLabel="已复制"
                className={styles.codeCopyButton}
              />
            </div>
            <pre className={styles.curlCode}>{curl}</pre>
            <div className={styles.securityNote}>
              <strong>凭证安全</strong>
              <span>命令包含真实凭证时属于敏感材料，请勿粘贴到工单、聊天或日志。</span>
            </div>
          </aside>
        </div>

        {initialConfig.data ? <ConfigResult config={initialConfig.data} /> : null}
      </main>
    </div>
  );
}
