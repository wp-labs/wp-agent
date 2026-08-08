import type { GatewayStatusView } from "../api";
import { GatewayStatusCard } from "./GatewayStatusCard";
import { LoadingDots } from "./ui";
import styles from "./GatewayStatusList.module.css";

/** 根据查询状态渲染网关卡片集合及对应的加载、空数据状态。 */
export function GatewayStatusList({
  items,
  uptimes,
  loading,
}: {
  items?: GatewayStatusView[];
  uptimes?: Record<string, number | null>;
  loading?: boolean;
}) {
  if (loading && (!items || items.length === 0)) {
    return (
      <div className={styles.wrapper}>
        <LoadingDots />
      </div>
    );
  }

  if (!items || items.length === 0) {
    return (
      <div className={styles.empty}>
        <div className={styles.emptyTitle}>暂无网关</div>
        <div className={styles.emptyHint}>
          尚未有任何 WarpGateWay 实例上报状态。
        </div>
      </div>
    );
  }

  return (
    <section
      className={styles.section}
      aria-labelledby="gateway-status-list-title"
    >
      <div className={styles.header}>
        <div>
          <h2 id="gateway-status-list-title" className={styles.title}>
            网关运行状态
          </h2>
          <p className={styles.subtitle}>
            选择网关查看实例与 Agent 的详细运行信息
          </p>
        </div>
        <span className={styles.count}>{items.length} 个网关</span>
      </div>
      <div className={styles.grid}>
        {items.map((gateway) => (
          <GatewayStatusCard
            key={gateway.gatewayId}
            gateway={gateway}
            uptime={uptimes?.[gateway.gatewayId]}
          />
        ))}
      </div>
    </section>
  );
}
