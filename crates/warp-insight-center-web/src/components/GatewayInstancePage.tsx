import { PageShell } from "./ui";
import { GatewayCustomerBindPanel } from "./GatewayCustomerBindPanel";
import { GatewayInstanceCreatePanel } from "./GatewayInstanceCreatePanel";
import styles from "./GatewayInstancePage.module.css";

export function GatewayInstancePage() {
  return (
    <PageShell
      title="网关实例"
      summary="创建 GateWay 管理实例并为实例绑定客户 ID，建立实例与客户的归属关系。"
    >
      <div className={styles.grid}>
        <GatewayInstanceCreatePanel />
        <GatewayCustomerBindPanel />
      </div>
    </PageShell>
  );
}
