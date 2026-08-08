import { PageShell } from "./ui";
import { UpgradePlanApprovePanel } from "./UpgradePlanApprovePanel";
import { UpgradePlanCreatePanel } from "./UpgradePlanCreatePanel";
import styles from "./UpgradePlanPage.module.css";

export function UpgradePlanPage() {
  return (
    <PageShell
      title="升级计划"
      summary="创建与批准 WarpAgentd / WarpGateWay 升级计划，用于分批升级。"
    >
      <div className={styles.grid}>
        <UpgradePlanCreatePanel />
        <UpgradePlanApprovePanel />
      </div>
    </PageShell>
  );
}
