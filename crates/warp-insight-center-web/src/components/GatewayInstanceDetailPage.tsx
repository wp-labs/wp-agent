import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { GatewayInstanceLifecycleState } from "../api";
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

/** 展示未上线网关实例的初始化入口、生命周期和下一步部署动作。 */
export function GatewayInstanceDetailPage() {
  const { gatewayId = "" } = useParams();
  const {
    data: instancesData,
    error: instancesError,
    isLoading,
  } = useGatewayInstances();
  const [copied, setCopied] = useState(false);
  const instance = instancesData?.data.find(
    (item) => item.gatewayId === gatewayId,
  );
  const {
    data: lifecycleData,
    error: lifecycleError,
    isLoading: lifecycleLoading,
  } = useGatewayLifecycle(instance?.gatewayId ?? "");
  const initUrl =
    instance?.initUrl ??
    `/api/v1/gateway/initial-config?instance_id=${encodeURIComponent(gatewayId)}`;

  async function copyInitUrl() {
    try {
      await navigator.clipboard.writeText(initUrl);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      // 浏览器禁用剪贴板时保留可选中的 URL，不阻断初始化流程。
    }
  }

  return (
    <PageShell
      title={instance ? `实例 ${instance.gatewayId}` : "实例详情"}
      summary="查看未上线实例的初始化入口与生命周期，完成 Gateway 部署后将自动进入运行态。"
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
              <h2 className={styles.cardTitle}>初始化接入</h2>
              <p className={styles.cardSubtitle}>
                将下面的 URL 配置到
                Gateway，启动后即可拉取控制中心地址、策略版本与遥测配置。
              </p>
            </header>
            <div className={styles.infoGrid}>
              <div className={styles.infoItem}>
                <span className={styles.infoLabel}>初始化 URL</span>
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
            <p className={styles.hint}>
              创建时生成的 Docker
              安装命令包含网关凭证，请回到创建回执或部署流水线获取；控制中心不会在实例列表中重复展示凭证。
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
