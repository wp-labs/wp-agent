import type { GatewayStatusView } from "../api";
import { GatewayStatusCard } from "./GatewayStatusCard";
import { LoadingDots } from "./ui";
import styles from "./GatewayStatusList.module.css";

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
        <div className={styles.emptyHint}>尚未有任何 WarpGateWay 实例上报状态。</div>
      </div>
    );
  }

  return (
    <div className={styles.grid}>
      {items.map((gateway) => (
        <GatewayStatusCard
          key={gateway.gatewayId}
          gateway={gateway}
          uptime={uptimes?.[gateway.gatewayId]}
        />
      ))}
    </div>
  );
}
