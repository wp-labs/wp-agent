import { Link } from "react-router-dom";
import type { GatewayInstance } from "../api";
import { useGatewayInstances } from "../hooks";
import {
  Badge,
  ErrorBanner,
  lifecycleLabel,
  lifecycleTone,
  LoadingDots,
} from "./ui";
import styles from "./GatewayInstanceList.module.css";

/** 按接入阶段展示实例，避免 Center 接入材料与运行监控入口混在同一序列中。 */
function InstanceGroup({
  id,
  title,
  description,
  instances,
  emptyText,
}: {
  id: string;
  title: string;
  description: string;
  instances: GatewayInstance[];
  emptyText: string;
}) {
  return (
    <section className={styles.group} aria-labelledby={id}>
      <header className={styles.groupHeader}>
        <div>
          <h3 id={id} className={styles.groupTitle}>
            {title}
          </h3>
          <p className={styles.groupDescription}>{description}</p>
        </div>
        <span className={styles.groupCount}>{instances.length}</span>
      </header>
      {instances.length === 0 ? (
        <div className={styles.empty}>{emptyText}</div>
      ) : (
        <div className={styles.list}>
          {instances.map((instance) => (
            <Link
              key={instance.gatewayId}
              to={
                instance.lifecycleState === "Running"
                  ? `/gateways/${encodeURIComponent(instance.gatewayId)}`
                  : `/instance/${encodeURIComponent(instance.gatewayId)}`
              }
              className={styles.item}
              data-state={instance.lifecycleState}
            >
              <div className={styles.itemMain}>
                <div className={styles.gatewayId}>{instance.gatewayId}</div>
                <div className={styles.instanceId}>
                  {instance.instanceId || "未上报实例"}
                </div>
              </div>
              <div className={styles.itemAside}>
                <Badge tone={lifecycleTone(instance.lifecycleState)}>
                  {lifecycleLabel(instance.lifecycleState)}
                </Badge>
                <span className={styles.actionLabel}>
                  {instance.lifecycleState === "Running"
                    ? "运行详情 →"
                    : "接入材料 →"}
                </span>
              </div>
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}

/** 网关实例总览：加载一次实例数据，并将接入中与运行中实例分区呈现。 */
export function GatewayInstanceList() {
  const { data, error, isLoading } = useGatewayInstances();
  const instances = data?.data ?? [];
  const onboardingInstances = instances.filter(
    (instance) => instance.lifecycleState !== "Running",
  );
  const runningInstances = instances.filter(
    (instance) => instance.lifecycleState === "Running",
  );

  return (
    <section className={styles.section}>
      <div className={styles.header}>
        <div>
          <h2 className={styles.title}>实例总览</h2>
          <div className={styles.subtitle}>
            按接入阶段分区展示，快速定位待部署实例和已运行网关。
          </div>
        </div>
        <div className={styles.summary} role="group" aria-label="实例数量统计">
          <span className={styles.summaryItem}>
            <strong>{onboardingInstances.length}</strong> 待接入
          </span>
          <span className={styles.summaryItem}>
            <strong>{runningInstances.length}</strong> 已运行
          </span>
        </div>
      </div>

      {isLoading && instances.length === 0 ? (
        <div className={styles.feedback}>
          <LoadingDots />
        </div>
      ) : null}
      {error ? (
        <div className={styles.feedback}>
          <ErrorBanner>实例列表加载失败：{String(error)}</ErrorBanner>
        </div>
      ) : null}

      {!isLoading && !error ? (
        <div className={styles.groups}>
          <InstanceGroup
            id="gateway-onboarding-title"
            title="待接入实例"
            description="尚未完成首次上线，需要继续部署并在 Gateway 管理台完成初始化。"
            instances={onboardingInstances}
            emptyText="当前没有待接入实例。"
          />
          <InstanceGroup
            id="gateway-running-title"
            title="已运行实例"
            description="已完成接入并持续上报状态，可进入运行详情查看监控数据。"
            instances={runningInstances}
            emptyText="当前没有已运行实例。"
          />
        </div>
      ) : null}
    </section>
  );
}
