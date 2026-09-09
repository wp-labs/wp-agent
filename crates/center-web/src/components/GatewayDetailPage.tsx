import { Link, useParams } from "react-router-dom";
import {
  useAgentHistories,
  useGatewayAgents,
  useGatewayHistory,
  useGatewayLifecycle,
  useGatewayStatus,
  useGatewayUptimes,
} from "../hooks";
import {
  Badge,
  formatBytes,
  formatPercent,
  formatRelativeTime,
  lifecycleLabel,
  lifecycleTone,
  LoadingDots,
  PageShell,
} from "./ui";
import { GatewayHealthBadge } from "./GatewayHealthBadge";
import { GatewayHistoryChart } from "./GatewayHistoryChart";
import { GatewayOnlineStatusBadge } from "./GatewayOnlineStatusBadge";
import { GatewayVersionText } from "./GatewayVersionText";
import styles from "./GatewayDetailPage.module.css";

export function GatewayDetailPage() {
  const { gatewayId = "" } = useParams();
  const { data: statusData, isLoading } = useGatewayStatus(gatewayId);
  const { data: agentsData } = useGatewayAgents(gatewayId);
  const agentIds = agentsData?.data?.map((agent) => agent.agentId) ?? [];
  const { data: agentHistoriesData } = useAgentHistories(gatewayId, agentIds);
  const { data: historyData, isLoading: isHistoryLoading } =
    useGatewayHistory(gatewayId);
  const { data: uptimesData } = useGatewayUptimes([gatewayId]);
  const { data: lifecycleData } = useGatewayLifecycle(gatewayId);
  const lifecycle = lifecycleData?.data ?? [];

  const gateway = statusData?.data ?? null;
  const agents = agentsData?.data ?? [];
  const uptime = gatewayId ? uptimesData?.[gatewayId] : undefined;
  const uptimeText =
    uptime === null || uptime === undefined
      ? "—"
      : `${(uptime * 100).toFixed(1)}%`;
  // 当前生命周期状态 = 最近一次转变的目标状态（明显位置展示）。
  const currentState =
    lifecycle.length > 0 ? lifecycle[lifecycle.length - 1].toState : null;

  return (
    <PageShell
      title={gateway ? `网关 ${gateway.gatewayId}` : "网关详情"}
      summary="查看该网关的状态与上报的 Agent 状态。"
    >
      <Link to="/" className={styles.backLink}>
        ← 返回网关态势
      </Link>

      {isLoading && !gateway ? <LoadingDots /> : null}

      {gateway ? (
        <section className={styles.section}>
          <div className={styles.card}>
            <div className={styles.cardHeader}>
              <div className={styles.headerRow}>
                <div className={styles.gatewayId}>{gateway.gatewayId}</div>
                {currentState ? (
                  <Badge tone={lifecycleTone(currentState)}>
                    {lifecycleLabel(currentState)}
                  </Badge>
                ) : null}
              </div>
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
                <div className={styles.metricLabel}>内存</div>
                <div>{formatBytes(gateway.memoryBytes)}</div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>CPU</div>
                <div>{formatPercent(gateway.cpuPercent)}</div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>版本</div>
                <GatewayVersionText value={gateway.version} />
              </div>
              <div className={styles.metric}>
                <div className={styles.metricLabel}>最后上报</div>
                <div>{formatRelativeTime(gateway.lastSeenAt)}</div>
              </div>
              <GatewayHistoryChart
                history={historyData?.data}
                loading={isHistoryLoading}
                source={historyData?.source}
              />
            </div>
          </div>
        </section>
      ) : null}

      <section className={styles.section}>
        <div className={styles.sectionHeader}>
          <div className={styles.sectionTitle}>生命周期过程</div>
          <span className={styles.sectionMeta}>{lifecycle.length} 次转变</span>
        </div>
        {lifecycle.length === 0 ? (
          <div className={styles.empty}>尚无生命周期记录。</div>
        ) : (
          <div className={styles.timeline}>
            {lifecycle.map((event, index) => (
              <div key={index} className={styles.timelineItem}>
                <span className={styles.timelineDot} aria-hidden="true" />
                <div className={styles.timelineBody}>
                  <div className={styles.timelineText}>
                    {event.fromState
                      ? lifecycleLabel(event.fromState)
                      : "创建实例"}
                    {" → "}
                    {lifecycleLabel(event.toState)}
                  </div>
                  <div className={styles.timelineTime}>
                    {formatRelativeTime(event.at)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

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
                <div className={styles.agentMetrics}>
                  <span>内存 {formatBytes(agent.memoryBytes)}</span>
                  <span>CPU {formatPercent(agent.cpuPercent)}</span>
                  <span>时延 {agent.adminLatencyMs ?? "—"}ms</span>
                </div>
                <GatewayHistoryChart
                  history={agentHistoriesData?.[agent.agentId]?.data}
                  source={agentHistoriesData?.[agent.agentId]?.source}
                  compact
                  title="Agent 最近 1 小时"
                />
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
