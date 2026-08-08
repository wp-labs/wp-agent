import { Link } from "react-router-dom";
import type { GatewayStatusView } from "../api";
import { formatBytes, formatPercent, formatRelativeTime } from "./ui";
import { GatewayHealthBadge } from "./GatewayHealthBadge";
import { GatewayInstanceText } from "./GatewayInstanceText";
import { GatewayOnlineStatusBadge } from "./GatewayOnlineStatusBadge";
import { GatewayVersionText } from "./GatewayVersionText";
import styles from "./GatewayStatusCard.module.css";

/** 展示单个网关的关键状态，并提供进入网关详情页的完整点击区域。 */
export function GatewayStatusCard({
  gateway,
  uptime,
}: {
  gateway: GatewayStatusView;
  uptime?: number | null;
}) {
  const isOffline = gateway.status === "offline";
  const stale =
    Date.now() - new Date(gateway.lastSeenAt).getTime() > 5 * 60_000;
  const uptimeText =
    uptime === null || uptime === undefined
      ? "—"
      : `${(uptime * 100).toFixed(1)}%`;
  const uptimeLow = uptime !== null && uptime !== undefined && uptime < 0.9;

  return (
    <article
      className={`${styles.card} ${isOffline ? styles.offlineCard : ""}`}
    >
      <Link
        to={`/gateways/${encodeURIComponent(gateway.gatewayId)}`}
        className={styles.cardLink}
        aria-label={`查看网关 ${gateway.gatewayId} 详情`}
      >
        <div className={styles.top}>
          <div className={styles.heading}>
            <div className={styles.gatewayType}>WarpGateWay</div>
            <div className={styles.gatewayId}>
              {gateway.gatewayId}
              <span className={styles.openHint} aria-hidden="true">
                ↗
              </span>
            </div>
            <GatewayInstanceText value={gateway.instanceId} />
          </div>
          <div className={styles.statusGroup}>
            <GatewayHealthBadge value={gateway.health} />
            <GatewayOnlineStatusBadge value={gateway.status} />
          </div>
        </div>
        <div className={styles.details}>
          <div className={styles.item}>
            <div className={styles.label}>在线率（1h）</div>
            <div
              className={`${styles.uptimeValue} ${uptimeLow ? styles.uptimeLow : ""}`}
            >
              {uptimeText}
            </div>
          </div>
          <div className={styles.item}>
            <div className={styles.label}>内存</div>
            <div>{formatBytes(gateway.memoryBytes)}</div>
          </div>
          <div className={styles.item}>
            <div className={styles.label}>CPU</div>
            <div>{formatPercent(gateway.cpuPercent)}</div>
          </div>
          <div className={styles.item}>
            <div className={styles.label}>版本</div>
            <GatewayVersionText value={gateway.version} />
          </div>
          <div className={styles.item}>
            <div className={styles.label}>最后上报</div>
            <div className={stale ? styles.staleValue : undefined}>
              {formatRelativeTime(gateway.lastSeenAt)}
            </div>
            {stale ? (
              <div className={styles.staleHint}>长时间未上报</div>
            ) : null}
          </div>
        </div>
      </Link>
    </article>
  );
}
