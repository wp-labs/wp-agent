import styles from "./GatewayVersionText.module.css";

export function GatewayVersionText({ value }: { value: string }) {
  return <span className={styles.version}>{value}</span>;
}
