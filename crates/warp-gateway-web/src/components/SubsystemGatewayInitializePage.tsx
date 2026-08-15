import { useState, type FormEvent } from "react";
import { ApiError } from "../api";
import { useGatewayInitialConfig } from "../hooks";
import { CopyButton } from "./CopyButton";
import { SubsystemAdminTopNavigation } from "./SubsystemAdminTopNavigation";
import styles from "./SubsystemGatewayInitializePage.module.css";

function errorMessage(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.status === 401)
      return "网关凭证缺失、无效或已失效：请填写 config.toml 中的 token，或从 Center 重新生成初始化材料。";
    if (error.status === 429) return "认证失败次数过多，请稍后再试。";
    return `控制中心返回 HTTP ${error.status}，请核对初始化 URL 和服务状态。`;
  }
  if (error instanceof TypeError) {
    return "无法访问控制中心。若 Gateway 页面与 Center 不同源，请检查网络和 CORS 配置。";
  }
  return "初始化配置响应不符合当前契约，请检查 Center 与 Gateway 版本。";
}

function ConfigResult({ config }: { config: string }) {
  return (
    <section className={styles.result} aria-live="polite" aria-labelledby="gateway-initial-config-result">
      <header className={styles.resultHeader}>
        <div>
          <div className={styles.resultEyebrow}>config.toml</div>
          <h2 id="gateway-initial-config-result" className={styles.sectionTitle}>
            网关初始配置
          </h2>
        </div>
        <span className={styles.successBadge}>获取成功</span>
      </header>
      <div className={styles.tomlBlock}>
        <pre>{config}</pre>
      </div>
      <CopyButton text={config} label="复制 config.toml" className={styles.secondaryButton} />
    </section>
  );
}

/**
 * Gateway 自身初始化页：消费 Center 创建实例时交付的 init_url，
 * 只负责获取和展示 Gateway 初始配置，不处理 Agent 安装配置。
 */
export function SubsystemGatewayInitializePage() {
  const [initUrl, setInitUrl] = useState("");
  const [token, setToken] = useState("");
  const initialConfig = useGatewayInitialConfig();
  const canSubmit = initUrl.trim().length > 0;

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) return;
    initialConfig.mutate({ initUrl, token: token.trim() || undefined });
  }

  return (
    <div className={styles.container}>
      <SubsystemAdminTopNavigation />
      <main className={styles.main}>
        <header className={styles.pageHeader}>
          <div className={styles.eyebrow}>Gateway / InitializeGatewayViaUrl</div>
          <h1 className={styles.pageTitle}>通过 URL 初始化</h1>
          <p className={styles.pageSummary}>
            Gateway 安装完成后，访问控制中心提供的初始化 URL，获取网关初始配置并完成接入。
          </p>
        </header>

        <section className={styles.usecaseCard} aria-labelledby="gateway-initialize-usecase">
          <header className={styles.usecaseMeta}>
            <div className={styles.usecaseMetaCopy}>
              <span className={styles.usecaseTag}>USECASE</span>
              <h2 id="gateway-initialize-usecase">通过 URL 初始化 Gateway</h2>
              <p>输入控制中心初始化 URL，获取 GatewayInitialConfig，完成网关初始化。</p>
            </div>
            <div className={styles.usecaseFlow} aria-label="初始化流程">
              <span>InitializeGatewayViaUrl</span>
              <span className={styles.flowArrow}>→</span>
              <span>GatewayInitialConfig</span>
            </div>
          </header>

          <form className={styles.form} onSubmit={handleSubmit}>
            <label className={`${styles.field} ${styles.urlField}`}>
              <span>控制中心初始化 URL</span>
              <input
                type="url"
                value={initUrl}
                onChange={(event) => setInitUrl(event.target.value)}
                placeholder="https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo"
                autoComplete="url"
                required
              />
              <small>使用 Center 创建实例时交付的 init_url，必须包含 instance_id。</small>
            </label>
            <label className={styles.field}>
              <span>网关注册凭证（可选）</span>
              <input
                type="password"
                value={token}
                onChange={(event) => setToken(event.target.value)}
                placeholder="Bearer 凭证（config.toml 中的 token）"
                autoComplete="off"
              />
              <small>token 不进 URL，仅用于 Authorization Header 调 init_url；留空则网关无凭证返回 401。</small>
            </label>
            {initialConfig.isError ? (
              <div className={styles.errorBanner} role="alert">
                {errorMessage(initialConfig.error)}
              </div>
            ) : null}
            <div className={styles.formActions}>
              <button
                type="submit"
                className={styles.primaryButton}
                disabled={!canSubmit || initialConfig.isPending}
              >
                {initialConfig.isPending ? "正在初始化…" : "获取初始配置"}
              </button>
              <span className={styles.actionHint}>触发 InitializeGatewayViaUrl</span>
            </div>
          </form>
        </section>

        {initialConfig.data ? <ConfigResult config={initialConfig.data} /> : null}
      </main>
    </div>
  );
}
