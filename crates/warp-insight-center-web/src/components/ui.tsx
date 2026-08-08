import type {
  ReactNode,
  InputHTMLAttributes,
  ButtonHTMLAttributes,
} from "react";
import { GlobalTopNavigation } from "./GlobalTopNavigation";
import styles from "./ui.module.css";

export function formatDateTime(value: string | Date): string {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(typeof value === "string" ? new Date(value) : value);
}

/** 相对时间："x 秒前 / x 分钟前 / x 小时前 / x 天前"。 */
export function formatRelativeTime(value: string | Date): string {
  const then =
    typeof value === "string" ? new Date(value).getTime() : value.getTime();
  const diffMs = Date.now() - then;
  if (diffMs < 0) return "刚刚";
  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 60) return `${seconds} 秒前`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} 分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.floor(hours / 24);
  return `${days} 天前`;
}

/** 提供控制中心页面共享的导航、标题层级和主内容布局。 */
export function PageShell({
  title,
  summary,
  children,
}: {
  title: string;
  summary: string;
  children: ReactNode;
}) {
  return (
    <div className={styles.pageShell}>
      <GlobalTopNavigation />
      <header className={styles.pageHeader}>
        <div className={styles.pageEyebrow}>WarpInsight Control Plane</div>
        <h1 className={styles.pageTitle}>{title}</h1>
        <p className={styles.pageSummary}>{summary}</p>
      </header>
      <main className={styles.pageMain}>{children}</main>
    </div>
  );
}

export type BadgeTone = "green" | "amber" | "red" | "gray" | "blue";

const BADGE_TONE_CLASS: Record<BadgeTone, string> = {
  green: styles.badgeGreen,
  amber: styles.badgeAmber,
  red: styles.badgeRed,
  gray: styles.badgeGray,
  blue: styles.badgeBlue,
};

type MetricTone = "accent" | "green" | "amber" | "red";

const METRIC_TONE_CLASS: Record<MetricTone, string> = {
  accent: styles.metricToneAccent,
  green: styles.metricToneGreen,
  amber: styles.metricToneAmber,
  red: styles.metricToneRed,
};

export function Badge({
  tone,
  children,
}: {
  tone: BadgeTone;
  children: ReactNode;
}) {
  return (
    <span className={`${styles.badge} ${BADGE_TONE_CLASS[tone]}`}>
      {children}
    </span>
  );
}

export function SectionCard({
  title,
  subtitle,
  children,
}: {
  title: ReactNode;
  subtitle?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className={styles.card}>
      <header className={styles.cardHeader}>
        <h2 className={styles.cardTitle}>{title}</h2>
        {subtitle ? <p className={styles.cardSubtitle}>{subtitle}</p> : null}
      </header>
      {children}
    </section>
  );
}

export function MetricCard({
  label,
  value,
  tone,
}: {
  label: string;
  value: ReactNode;
  tone?: MetricTone;
}) {
  const toneClass = tone ? METRIC_TONE_CLASS[tone] : undefined;
  return (
    <div className={styles.metric}>
      <div className={styles.metricLabel}>{label}</div>
      <div className={`${styles.metricValue} ${toneClass ?? ""}`}>{value}</div>
    </div>
  );
}

export function FormField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className={styles.field}>
      <span className={styles.fieldLabel}>{label}</span>
      {children}
      {hint ? <span className={styles.fieldHint}>{hint}</span> : null}
    </label>
  );
}

export function TextInput(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className={styles.input} {...props} />;
}

export function FormStack({
  children,
  actions,
}: {
  children: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <div className={styles.formStack}>
      {children}
      {actions ? <div className={styles.formActions}>{actions}</div> : null}
    </div>
  );
}

export function PrimaryButton(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button className={styles.primaryButton} {...props}>
      {props.children}
    </button>
  );
}

export function ReceiptCard({
  title,
  fields,
}: {
  title: string;
  fields: [label: string, value: ReactNode][];
}) {
  return (
    <div className={styles.receipt}>
      <div className={styles.receiptTitle}>{title}</div>
      <dl className={styles.receiptGrid}>
        {fields.map(([label, value]) => (
          <div key={label} className={styles.receiptItem}>
            <dt className={styles.receiptLabel}>{label}</dt>
            <dd className={styles.receiptValue}>{value}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

export function ErrorBanner({ children }: { children: ReactNode }) {
  return <div className={styles.errorBanner}>{children}</div>;
}

export function LoadingDots() {
  return <span className={styles.loading}>加载中…</span>;
}

export function ExampleTag() {
  return <span className={styles.exampleTag}>示例数据</span>;
}
