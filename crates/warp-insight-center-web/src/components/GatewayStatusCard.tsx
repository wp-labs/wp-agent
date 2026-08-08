import type { GatewayStatusView } from "../api";
import { GatewayHealthBadge } from "./GatewayHealthBadge";
import { GatewayInstanceText } from "./GatewayInstanceText";
import { GatewayLastSeenAtText } from "./GatewayLastSeenAtText";
import { GatewayOnlineStatusBadge } from "./GatewayOnlineStatusBadge";
import { GatewayVersionText } from "./GatewayVersionText";
import styles from "./GatewayStatusCard.module.css";

export function GatewayStatusCard({ gateway }: { gateway: GatewayStatusView }) {
  return (
    <article className={styles.card}>
      <div className={styles.top}>
        <div className={styles.heading}>
          <GatewayInstanceText value={gateway.instanceId} />
          <div className={styles.gatewayId}>{gateway.gatewayId}</div>
        </div>
        <GatewayHealthBadge value={gateway.health} />
      </div>
      <div className={styles.details}>
        <div className={styles.item}>
          <div className={styles.label}>状态</div>
          <GatewayOnlineStatusBadge value={gateway.status} />
        </div>
        <div className={styles.item}>
          <div className={styles.label}>版本</div>
          <GatewayVersionText value={gateway.version} />
        </div>
        <div className={styles.item}>
          <div className={styles.label}>最后上报</div>
          <GatewayLastSeenAtText value={gateway.lastSeenAt} />
        </div>
      </div>
    </article>
  );
}
