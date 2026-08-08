import type { GatewayListView } from "../api";
import { ExampleTag, MetricCard } from "./ui";
import styles from "./GatewayStatusOverviewMetrics.module.css";

export function GatewayStatusOverviewMetrics({
  list,
  averageUptime,
}: {
  list?: GatewayListView | null;
  averageUptime?: number | null;
}) {
  const uptimeText =
    averageUptime === null || averageUptime === undefined
      ? "—"
      : `${(averageUptime * 100).toFixed(1)}%`;
  return (
    <div className={styles.section}>
      <div className={styles.header}>
        <div className={styles.headerTitle}>全局网关概览</div>
        {list ? (
          <span className={styles.headerMeta}>
            更新于 {new Date(list.updatedAt).toLocaleTimeString("zh-CN")}
          </span>
        ) : null}
      </div>
      <div className={styles.grid}>
        <MetricCard label="网关总数" value={list?.gatewayCount ?? "—"} tone="accent" />
        <MetricCard label="在线" value={list?.onlineCount ?? "—"} tone="green" />
        <MetricCard label="降级" value={list?.degradedCount ?? "—"} tone="amber" />
        <MetricCard label="离线" value={list?.offlineCount ?? "—"} tone="red" />
        <MetricCard label="平均在线率" value={uptimeText} tone="green" />
      </div>
    </div>
  );
}

export function ExampleDataTag({ source }: { source?: "real" | "example" }) {
  if (source !== "example") return null;
  return (
    <div className={styles.exampleRow}>
      <ExampleTag />
      <span className={styles.exampleHint}>
        后端管理接口未就绪，当前展示内置示例数据。
      </span>
    </div>
  );
}
