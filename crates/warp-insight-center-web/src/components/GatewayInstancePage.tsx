import { PageShell } from "./ui";
import { GatewayInstanceCreatePanel } from "./GatewayInstanceCreatePanel";
import { GatewayInstanceList } from "./GatewayInstanceList";
import styles from "./GatewayInstancePage.module.css";

/** 网关实例工作台：承载按需创建和按生命周期分组浏览两个主流程。 */
export function GatewayInstancePage() {
  return (
    <PageShell
      title="网关管理"
      summary="创建并管理 Gateway 实例；未上线实例查看 Center 接入材料，Gateway 初始化在网关管理台完成。"
    >
      <div className={styles.content}>
        <GatewayInstanceCreatePanel />
        <GatewayInstanceList />
      </div>
    </PageShell>
  );
}
