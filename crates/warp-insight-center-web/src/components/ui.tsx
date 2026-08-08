import type {
  ReactNode,
  InputHTMLAttributes,
  ButtonHTMLAttributes,
} from "react";
import { GlobalTopNavigation } from "./GlobalTopNavigation";
import styles from "./ui.module.css";

// ── 设计系统原语：延续 warp-gateway-web 的管理台视觉 ──
// 浅灰背景、白色卡片、#1f6feb 主色、圆角徽标。

export function formatDateTime(value: string | Date): string {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(typeof value === "string" ? new Date(value) : value);
}

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
        <h1 className={styles.pageTitle}>{title}</h1>
        <p className={styles.pageSummary}>{summary}</p>
      </header>
      {children}
    </div>
  );
}

export type BadgeTone = "green" | "amber" | "red" | "gray" | "blue";

export function Badge({
  tone,
  children,
}: {
  tone: BadgeTone;
  children: ReactNode;
}) {
  return (
    <span className={`${styles.badge} ${styles[`badge${tone}`]}`}>
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
  tone?: "accent" | "green" | "amber" | "red";
}) {
  const toneClass = tone ? styles[`metricTone${tone}`] : undefined;
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
