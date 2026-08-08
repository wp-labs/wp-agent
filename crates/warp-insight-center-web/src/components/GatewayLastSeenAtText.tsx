import { formatDateTime } from "./ui";
import styles from "./GatewayLastSeenAtText.module.css";

export function GatewayLastSeenAtText({ value }: { value: string }) {
  return <span className={styles.text}>{formatDateTime(value)}</span>;
}
