import { PageShell } from "./ui";
import { GatewayInstanceCreatePanel } from "./GatewayInstanceCreatePanel";
import { GatewayInstanceList } from "./GatewayInstanceList";
import styles from "./GatewayInstancePage.module.css";

/** 网关实例工作台：承载按需创建和按生命周期分组浏览两个主流程。 */
export function GatewayInstancePage() {
  return (
    <PageShell
      title="网关管理"
      summary="创建并管理 Gateway 实例；未上线实例进入初始化流程，已运行实例进入运行监控。"
    >
      <div className={styles.content}>
        <GatewayInstanceCreatePanel />
        <GatewayInstanceList />
      </div>
    </PageShell>
  );
}
