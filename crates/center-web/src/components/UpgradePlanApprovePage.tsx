import { useApproveUpgradePlan, useUpgradePlans } from "../hooks";
import { Badge, ErrorBanner, LoadingDots, PageShell } from "./ui";
import styles from "./UpgradePlanApprovePage.module.css";

/** 批准升级计划：独立的批准工作区，按计划列表逐项批准。 */
export function UpgradePlanApprovePage() {
  const { data, isLoading } = useUpgradePlans();
  const plans = data?.data ?? [];
  const approve = useApproveUpgradePlan();

  function handleApprove(planId: string) {
    approve.mutate({ planId, approvedBy: "admin" });
  }

  return (
    <PageShell
      title="批准升级计划"
      summary="查看待批准的升级计划并逐项批准，同意后网关/Agent 按步骤执行升级。"
    >
      {isLoading && plans.length === 0 ? <LoadingDots /> : null}
      {plans.length === 0 ? (
        <div className={styles.empty}>暂无升级计划。</div>
      ) : (
        <div className={styles.list}>
          {plans.map((plan) => (
            <div key={plan.planId} className={styles.item}>
              <div className={styles.itemMain}>
                <div className={styles.itemTitle}>{plan.planId}</div>
                <div className={styles.itemSub}>
                  {plan.targets
                    .map((target) => `${target.component} ${target.targetVersion}`)
                    .join("，")}
                  {" · "}
                  {plan.targetCount} 个网关 · {plan.steps.length} 步
                </div>
              </div>
              <Badge tone={plan.status === "approved" ? "green" : "amber"}>
                {plan.status === "approved" ? "已批准" : "待批准"}
              </Badge>
              {plan.status === "pending" ? (
                <button
                  type="button"
                  className={styles.approveButton}
                  disabled={approve.isPending}
                  onClick={() => handleApprove(plan.planId)}
                >
                  {approve.isPending ? "批准中…" : "批准"}
                </button>
              ) : null}
            </div>
          ))}
        </div>
      )}
      {approve.error ? (
        <ErrorBanner>批准失败：{String(approve.error)}</ErrorBanner>
      ) : null}
      {approve.data ? (
        <div className={styles.approvedNotice}>
          已批准：{approve.data.data.planId}
        </div>
      ) : null}
    </PageShell>
  );
}
