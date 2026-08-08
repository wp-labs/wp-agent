import type { FormEvent } from "react";
import { useCreateGatewayInstance } from "../hooks";
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

export function GatewayInstanceCreatePanel() {
  const mutation = useCreateGatewayInstance();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      gatewayName: String(data.get("gatewayName") ?? ""),
      requestedBy: String(data.get("requestedBy") ?? ""),
    });
  }

  const instance = mutation.data?.data;

  return (
    <SectionCard
      title="创建网关实例"
      subtitle="客户服务工程师创建 GateWay 管理实例，作为后续接入与状态聚合的管理对象。"
    >
      <form onSubmit={handleSubmit}>
        <FormStack
          actions={
            <PrimaryButton type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "创建中…" : "创建实例"}
            </PrimaryButton>
          }
        >
          <FormField label="网关名称" hint="例如：gw-prod-east">
            <TextInput
              name="gatewayName"
              required
              placeholder="请输入网关名称"
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
      {instance ? (
        <ReceiptCard
          title="创建回执"
          fields={[
            ["网关 ID", instance.gatewayId],
            ["实例 ID", instance.instanceId],
            ["状态", instance.status],
            ["创建时间", formatDateTime(instance.createdAt)],
          ]}
        />
      ) : null}
    </SectionCard>
  );
}
