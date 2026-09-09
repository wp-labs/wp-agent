import type { GatewayHistory, GatewayHistorySample } from "../api";
import { formatBytes, formatPercent } from "./ui";
import styles from "./GatewayHistoryChart.module.css";

interface GatewayHistoryChartProps {
  history?: GatewayHistory | null;
  loading?: boolean;
  source?: "real" | "example";
  title?: string;
  compact?: boolean;
}

type SampleValue = (sample: GatewayHistorySample) => number | null;

/** 在网关概览中展示最近窗口的 CPU、内存趋势和在线时间轴。 */
export function GatewayHistoryChart({
  history,
  loading,
  source,
  title = "最近 1 小时",
  compact = false,
}: GatewayHistoryChartProps) {
  const samples = history?.samples ?? [];
  if (loading && samples.length === 0) {
    return (
      <div className={`${styles.state} ${compact ? styles.compact : ""}`}>
        正在加载最近 1 小时趋势…
      </div>
    );
  }
  if (samples.length === 0) {
    return (
      <div className={`${styles.state} ${compact ? styles.compact : ""}`}>
        <strong>最近 1 小时暂无历史样本</strong>
        <span>配置 VictoriaMetrics 并持续上报后将在这里生成趋势。</span>
      </div>
    );
  }

  const firstAt = samples[0]?.at;
  const lastAt = samples[samples.length - 1]?.at;
  return (
    <div className={`${styles.panel} ${compact ? styles.compact : ""}`}>
      <div className={styles.header}>
        <div>
          <div className={styles.title}>{title}</div>
          <div className={styles.timeRange}>
            {formatSampleTime(firstAt)}–{formatSampleTime(lastAt)}
          </div>
        </div>
        <div className={styles.headerMeta}>
          {source === "example" ? (
            <span className={styles.example}>示例</span>
          ) : null}
          <span>{samples.length} 个采样</span>
        </div>
      </div>

      <TrendRow
        label="CPU"
        value={formatPercent(
          latestValue(samples, (sample) => sample.cpuPercent),
        )}
        samples={samples}
        valueOf={(sample) => sample.cpuPercent}
        tone="blue"
      />
      <TrendRow
        label="内存"
        value={formatBytes(
          latestValue(samples, (sample) => sample.memoryBytes),
        )}
        samples={samples}
        valueOf={(sample) => sample.memoryBytes}
        tone="green"
      />
      <div className={styles.availabilityRow}>
        <span className={styles.rowLabel}>在线</span>
        <div className={styles.availability} aria-label="最近 1 小时在线状态">
          {samples.map((sample) => (
            <span
              key={sample.at}
              className={
                sample.online === null
                  ? styles.unknownSegment
                  : sample.online >= 0.5
                    ? styles.onlineSegment
                    : styles.offlineSegment
              }
              title={`${formatSampleTime(sample.at)} ${
                sample.online === null
                  ? "无数据"
                  : sample.online >= 0.5
                    ? "在线"
                    : "离线"
              }`}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function TrendRow({
  label,
  value,
  samples,
  valueOf,
  tone,
}: {
  label: string;
  value: string;
  samples: GatewayHistorySample[];
  valueOf: SampleValue;
  tone: "blue" | "green";
}) {
  const points = buildPolyline(samples, valueOf);
  return (
    <div className={styles.trendRow}>
      <span className={styles.rowLabel}>{label}</span>
      <svg
        className={styles.sparkline}
        viewBox="0 0 240 36"
        preserveAspectRatio="none"
        role="img"
        aria-label={`${label} 最近 1 小时趋势`}
      >
        <line className={styles.guide} x1="0" y1="18" x2="240" y2="18" />
        {points ? (
          <polyline
            className={tone === "blue" ? styles.blueLine : styles.greenLine}
            points={points}
          />
        ) : null}
      </svg>
      <strong className={styles.rowValue}>{value}</strong>
    </div>
  );
}

/** 按实际采样时间和当前序列值域生成 SVG 折线点。 */
function buildPolyline(
  samples: GatewayHistorySample[],
  valueOf: SampleValue,
): string {
  const points = samples.flatMap((sample) => {
    const value = valueOf(sample);
    return value === null ? [] : [{ at: sample.at, value }];
  });
  if (points.length === 0) return "";
  const minAt = points[0]?.at ?? 0;
  const maxAt = points[points.length - 1]?.at ?? minAt;
  const values = points.map((point) => point.value);
  const minValue = Math.min(...values);
  const maxValue = Math.max(...values);
  const timeRange = Math.max(maxAt - minAt, 1);
  const valueRange = Math.max(
    maxValue - minValue,
    Math.abs(maxValue) * 0.08,
    1,
  );
  return points
    .map((point) => {
      const x = ((point.at - minAt) / timeRange) * 240;
      const y = 31 - ((point.value - minValue) / valueRange) * 26;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function latestValue(
  samples: GatewayHistorySample[],
  valueOf: SampleValue,
): number | null {
  for (let index = samples.length - 1; index >= 0; index -= 1) {
    const value = valueOf(samples[index]);
    if (value !== null) return value;
  }
  return null;
}

function formatSampleTime(at?: number): string {
  if (at === undefined) return "—";
  return new Intl.DateTimeFormat("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(at * 1000));
}
