import { PageShell } from "./ui";
import { WarpAgentdReleasePanel } from "./WarpAgentdReleasePanel";
import { WarpGateWayReleasePanel } from "./WarpGateWayReleasePanel";
import styles from "./ReleasePage.module.css";

export function ReleasePage() {
  return (
    <PageShell
      title="版本发布"
      summary="发布 WarpAgentd / WarpGateWay 新版本，供各 WarpGateWay 实例或管理实例升级。"
    >
      <div className={styles.grid}>
        <WarpAgentdReleasePanel />
        <WarpGateWayReleasePanel />
      </div>
    </PageShell>
  );
}
