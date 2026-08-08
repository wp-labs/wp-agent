import type { FormEvent } from "react";
import { useCreateUpgradePlan } from "../hooks";
import {
  ErrorBanner,
  FormField,
  FormStack,
  PrimaryButton,
  ReceiptCard,
  SectionCard,
  TextInput,
  formatDateTime,
} from "./ui";

export function UpgradePlanCreatePanel() {
  const mutation = useCreateUpgradePlan();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const gatewayIds = String(data.get("gatewayIds") ?? "")
      .split(",")
      .map((id) => id.trim())
      .filter(Boolean);
    mutation.mutate({
      component: String(data.get("component") ?? ""),
      targetVersion: String(data.get("targetVersion") ?? ""),
      gatewayIds,
      requestedBy: String(data.get("requestedBy") ?? ""),
    });
  }

  const plan = mutation.data?.data;

  return (
    <SectionCard
      title="创建升级计划"
      subtitle="平台维护工程师创建升级计划，指定目标组件、目标版本与升级范围，用于分批升级。"
    >
      <form onSubmit={handleSubmit}>
        <FormStack
          actions={
            <PrimaryButton type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "创建中…" : "创建计划"}
            </PrimaryButton>
          }
        >
          <FormField label="目标组件" hint="warp-agentd 或 warp-gateway">
            <TextInput
              name="component"
              required
              placeholder="warp-agentd"
            />
          </FormField>
          <FormField label="目标版本" hint="例如：v2.4.1">
            <TextInput name="targetVersion" required placeholder="请输入目标版本" />
          </FormField>
          <FormField label="升级范围（网关 ID）" hint="逗号分隔，例如：gw-001,gw-002">
            <TextInput
              name="gatewayIds"
              required
              placeholder="gw-001,gw-002"
            />
          </FormField>
          <FormField label="申请者（requested_by）">
            <TextInput name="requestedBy" defaultValue="admin" required />
          </FormField>
        </FormStack>
      </form>
      {mutation.error ? (
        <ErrorBanner>创建失败：{String(mutation.error)}</ErrorBanner>
      ) : null}
      {plan ? (
        <ReceiptCard
          title="计划回执"
          fields={[
            ["计划 ID", plan.planId],
            ["组件", plan.component],
            ["目标版本", plan.targetVersion],
            ["目标数", String(plan.targetCount)],
            ["状态", plan.status],
            ["创建时间", formatDateTime(plan.createdAt)],
          ]}
        />
      ) : null}
    </SectionCard>
  );
}
