import styles from "./GatewayInstanceText.module.css";

export function GatewayInstanceText({ value }: { value: string }) {
  return <div className={styles.instance}>{value}</div>;
}
