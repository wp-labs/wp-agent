import { PageShell } from "./ui";
import { GatewayInitialConfigPanel } from "./GatewayInitialConfigPanel";
import styles from "./GatewayConfigPage.module.css";

export function GatewayConfigPage() {
  return (
    <PageShell
      title="网关初始配置"
      summary="查看指定 WarpGateWay 实例的初始配置信息，包括控制中心地址、策略版本与遥测输出。"
    >
      <div className={styles.panel}>
        <GatewayInitialConfigPanel />
      </div>
    </PageShell>
  );
}
