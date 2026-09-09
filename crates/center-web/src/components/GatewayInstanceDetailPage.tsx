import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  readGatewayInitCurl,
  type GatewayInstanceLifecycleState,
} from "../api";
import { useGatewayInstances, useGatewayLifecycle } from "../hooks";
import { GatewayCustomerBindPanel } from "./GatewayCustomerBindPanel";
import {
  Badge,
  type BadgeTone,
  ErrorBanner,
  formatDateTime,
  LoadingDots,
  PageShell,
  lifecycleLabel,
} from "./ui";
import styles from "./GatewayInstanceDetailPage.module.css";

function lifecycleTone(state: GatewayInstanceLifecycleState): BadgeTone {
  switch (state) {
    case "Running":
      return "green";
    case "Initializing":
      return "blue";
    case "Provisioned":
      return "gray";
    case "Failed":
      return "red";
  }
}

/** 展示未上线网关实例的接入材料、生命周期和下一步部署动作。 */
export function GatewayInstanceDetailPage() {
  const { gatewayId = "" } = useParams();
  const {
    data: instancesData,
    error: instancesError,
    isLoading,
  } = useGatewayInstances();
  const [copied, setCopied] = useState(false);
  const [curlCopied, setCurlCopied] = useState(false);
  const [gatewayToken, setGatewayToken] = useState("");
  const [initCurl, setInitCurl] = useState<string | null>(null);
  const instance = instancesData?.data.find(
    (item) => item.gatewayId === gatewayId,
  );
  const {
    data: lifecycleData,
    error: lifecycleError,
    isLoading: lifecycleLoading,
  } = useGatewayLifecycle(instance?.gatewayId ?? "");
  const initEndpoint =
    instance?.initUrl ??
    `/api/v1/gateway/initial-config?instance_id=${encodeURIComponent(gatewayId)}`;
  // init_url 不携带凭证（token 不进 URL），凭证走 config.toml / Authorization Header。
  const generatedInitUrl = initEndpoint;
  const initUrl = generatedInitUrl;
  const displayInitCurl =
    initCurl ?? `curl -H "Authorization: Bearer <gateway-identity-token>" "${initEndpoint.split("#", 1)[0]}"`;

  useEffect(() => {
    setInitCurl(readGatewayInitCurl(gatewayId));
  }, [gatewayId]);

  async function copyInitUrl() {
    try {
      await navigator.clipboard.writeText(initUrl);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      // 浏览器禁用剪贴板时保留可选中的 URL，不阻断部署接入流程。
    }
  }

  async function copyInitCurl() {
    try {
      await navigator.clipboard.writeText(displayInitCurl);
      setCurlCopied(true);
      window.setTimeout(() => setCurlCopied(false), 1500);
    } catch {
      // 剪贴板不可用时保留可选中的命令文本。
    }
  }

  function handleGenerateInitUrl() {
    if (!gatewayToken.trim()) return;
    setInitCurl(
      `curl -H "Authorization: Bearer ${gatewayToken.trim()}" "${initEndpoint.split("#", 1)[0]}"`,
    );
  }

  return (
    <PageShell
      title={instance ? `实例 ${instance.gatewayId}` : "实例详情"}
      summary="查看未上线实例的部署接入材料与生命周期；Gateway 初始化页面位于网关自身管理台。"
    >
      <Link to="/instance" className={styles.backLink}>
        ← 返回网关管理
      </Link>

      {isLoading && !instance ? <LoadingDots /> : null}

      {instancesError ? (
        <div className={styles.feedback}>
          <ErrorBanner>实例信息加载失败：{String(instancesError)}</ErrorBanner>
        </div>
      ) : null}

      {!isLoading && !instancesError && !instance ? (
        <section className={styles.notFound}>
          <h2>未找到该网关实例</h2>
          <p>实例可能已删除，或当前管理凭证无权查看。</p>
          <Link to="/instance" className={styles.primaryLink}>
            返回实例列表
          </Link>
        </section>
      ) : null}

      {instance ? (
        <div className={styles.content}>
          <section className={styles.hero}>
            <div>
              <div className={styles.eyebrow}>Gateway Instance</div>
              <h2 className={styles.gatewayId}>{instance.gatewayId}</h2>
              <p className={styles.instanceId}>
                {instance.instanceId || "实例尚未上报 instance_id"}
              </p>
            </div>
            <Badge tone={lifecycleTone(instance.lifecycleState)}>
              {lifecycleLabel(instance.lifecycleState)}
            </Badge>
          </section>

          <section className={styles.card}>
            <header className={styles.cardHeader}>
              <h2 className={styles.cardTitle}>Gateway 接入材料</h2>
              <p className={styles.cardSubtitle}>
                这些材料由 Center 生成，供 Gateway 部署后访问控制中心并完成首次接入。
              </p>
            </header>
            <div className={styles.accessGrid}>
              <div className={styles.endpointColumn}>
                <div className={styles.tokenBuilder}>
                  <div>
                    <span className={styles.infoLabel}>Gateway 身份 Token</span>
                    <p className={styles.tokenHint}>
                      创建 Gateway 实例时使用的身份凭证；用于生成本次 init_url 和 Center 接入 curl。
                    </p>
                  </div>
                  <div className={styles.tokenRow}>
                    <input
                      className={styles.tokenInput}
                      type="password"
                      value={gatewayToken}
                      onChange={(event) => setGatewayToken(event.target.value)}
                      placeholder="输入 Gateway 身份 Token"
                      autoComplete="off"
                    />
                    <button
                      type="button"
                      className={styles.generateButton}
                      onClick={handleGenerateInitUrl}
                      disabled={!gatewayToken.trim()}
                    >
                      生成接入 URL
                    </button>
                  </div>
                </div>
                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>Center 接入 URL</span>
                  <div className={styles.urlRow}>
                    <code className={styles.url}>{initUrl}</code>
                    <button
                      type="button"
                      className={styles.copyButton}
                      onClick={copyInitUrl}
                    >
                      {copied ? "已复制" : "复制"}
                    </button>
                  </div>
                </div>
                <div className={styles.curlBlock}>
                  <div className={styles.curlHeader}>
                    <div>
                        <span className={styles.infoLabel}>Center 接入 curl</span>
                      <p className={styles.curlHint}>
                        使用置备引导 Token 验证 Center 接入接口，命令可直接复制到终端执行。
                      </p>
                    </div>
                    <button
                      type="button"
                      className={styles.curlCopyButton}
                      onClick={copyInitCurl}
                    >
                      {curlCopied ? "已复制" : "复制命令"}
                    </button>
                  </div>
                  <pre className={styles.curlCode}>{displayInitCurl}</pre>
                  {!initCurl ? (
                    <p className={styles.curlPlaceholder}>
                      当前页面没有保存创建回执，请输入 Gateway 身份 Token 后生成完整命令。
                    </p>
                  ) : null}
                </div>
              </div>
              <div className={styles.metadataColumn}>
                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>实例 ID</span>
                  <strong>{instance.instanceId || "待首次上报生成"}</strong>
                </div>
                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>创建时间</span>
                  <strong>{formatDateTime(instance.createdAt)}</strong>
                </div>
                <div className={styles.infoItem}>
                  <span className={styles.infoLabel}>初始化完成</span>
                  <strong>
                    {instance.initializedAt
                      ? formatDateTime(instance.initializedAt)
                      : "尚未完成"}
                  </strong>
                </div>
              </div>
            </div>
            <p className={styles.hint}>
              Gateway 初始化页面属于网关自身管理台；本页只保存 Center 侧实例接入材料，不承载 Gateway 初始化流程。
            </p>
          </section>

          <section className={styles.card}>
            <header className={styles.cardHeader}>
              <h2 className={styles.cardTitle}>生命周期</h2>
              <p className={styles.cardSubtitle}>
                实例从创建到首次上线的状态变化记录。
              </p>
            </header>
            <div className={styles.timeline}>
              {lifecycleLoading ? <LoadingDots /> : null}
              {lifecycleError ? (
                <ErrorBanner>
                  生命周期加载失败：{String(lifecycleError)}
                </ErrorBanner>
              ) : null}
              {!lifecycleLoading &&
              !lifecycleError &&
              (lifecycleData?.data.length ?? 0) === 0 ? (
                <p className={styles.timelineEmpty}>暂无生命周期事件。</p>
              ) : null}
              {(lifecycleData?.data ?? []).map((event) => (
                <div
                  key={`${event.toState}-${event.at}`}
                  className={styles.event}
                >
                  <span className={styles.eventDot} />
                  <div>
                    <strong>{lifecycleLabel(event.toState)}</strong>
                    <span className={styles.eventMeta}>
                      {formatDateTime(event.at)}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </section>

          <GatewayCustomerBindPanel gatewayId={instance.gatewayId} />
        </div>
      ) : null}
    </PageShell>
  );
}
