import { Link, useParams } from "react-router-dom";
import {
  useGatewayAgents,
  useGatewayStatus,
  useGatewayUptimes,
} from "../hooks";
import { formatRelativeTime, LoadingDots, PageShell } from "./ui";
import { GatewayHealthBadge } from "./GatewayHealthBadge";
import { GatewayOnlineStatusBadge } from "./GatewayOnlineStatusBadge";
import { GatewayVersionText } from "./GatewayVersionText";
import styles from "./GatewayDetailPage.module.css";

export function GatewayDetailPage() {
  const { gatewayId = "" } = useParams();
  const { data: statusData, isLoading } = useGatewayStatus(gatewayId);
  const { data: agentsData } = useGatewayAgents(gatewayId);
  const { data: uptimesData } = useGatewayUptimes([gatewayId]);

  const gateway = statusData?.data ?? null;
  const agents = agentsData?.data ?? [];
  const uptime = gatewayId ? uptimesData?.[gatewayId] : undefined;
  const uptimeText =
    uptime === null || uptime === undefined
      ? "—"
      : `${(uptime * 100).toFixed(1)}%`;

  return (
    <PageShell
      title={gateway ? `网关 ${gateway.gatewayId}` : "网关详情"}
      summary="查看该网关的状态与上报的 Agent 状态。"
    >
      <Link to="/" className={styles.backLink}>
        ← 返回网关列表
      </Link>

      {isLoading && !gateway ? <LoadingDots /> : null}

      {gateway ? (
        <section className={styles.section}>
          <div className={styles.card}>
            <div className={styles.cardHeader}>
              <div className={styles.gatewayId}>{gateway.gatewayId}</div>
              <div className={styles.instanceId}>{gateway.instanceId}</div>
            </div>
            <div className={styles.metrics}>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>状态</div>
                <GatewayOnlineStatusBadge value={gateway.status} />
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>健康</div>
                <GatewayHealthBadge value={gateway.health} />
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>在线率（1h）</div>
                <div className={styles.metricValue}>{uptimeText}</div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>版本</div>
                <GatewayVersionText value={gateway.version} />
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>最后上报</div>
                <div>{formatRelativeTime(gateway.lastSeenAt)}</div>
              </div>
            </div>
          </div>
        </section>
      ) : null}

      <section className={styles.section}>
        <div className={styles.sectionHeader}>
          <div className={styles.sectionTitle}>Agent 状态</div>
          <span className={styles.sectionMeta}>{agents.length} 个</span>
        </div>
        {agents.length === 0 ? (
          <div className={styles.empty}>该网关尚未上报 Agent 状态。</div>
        ) : (
          <div className={styles.agentGrid}>
            {agents.map((agent) => (
              <div key={agent.agentId} className={styles.agentCard}>
                <div className={styles.agentHeader}>
                  <div className={styles.agentId}>{agent.agentId}</div>
                  <GatewayOnlineStatusBadge value={agent.status} />
                </div>
                <div className={styles.agentMeta}>
                  <span>版本 {agent.version}</span>
                  <GatewayHealthBadge value={agent.health} />
                </div>
                <div className={styles.agentLastSeen}>
                  {formatRelativeTime(agent.lastSeenAt)}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </PageShell>
  );
}
