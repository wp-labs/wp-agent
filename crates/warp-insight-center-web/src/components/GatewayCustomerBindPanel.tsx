import type { FormEvent } from "react";
import { useBindGatewayCustomer } from "../hooks";
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

export function GatewayCustomerBindPanel() {
  const mutation = useBindGatewayCustomer();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      gatewayId: String(data.get("gatewayId") ?? ""),
      customerId: String(data.get("customerId") ?? ""),
      requestedBy: String(data.get("requestedBy") ?? ""),
    });
  }

  const binding = mutation.data?.data;

  return (
    <SectionCard
      title="绑定客户"
      subtitle="为 GateWay 管理实例绑定客户 ID，建立实例与客户的归属关系。"
    >
      <form onSubmit={handleSubmit}>
        <FormStack
          actions={
            <PrimaryButton type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "绑定中…" : "绑定客户"}
            </PrimaryButton>
          }
        >
          <FormField label="网关 ID" hint="例如：gw-001">
            <TextInput name="gatewayId" required placeholder="请输入网关 ID" />
          </FormField>
          <FormField label="客户 ID" hint="例如：cust-acme">
            <TextInput name="customerId" required placeholder="请输入客户 ID" />
          </FormField>
          <FormField label="申请者（requested_by）">
            <TextInput name="requestedBy" defaultValue="admin" required />
          </FormField>
        </FormStack>
      </form>
      {mutation.error ? (
        <ErrorBanner>绑定失败：{String(mutation.error)}</ErrorBanner>
      ) : null}
      {binding ? (
        <ReceiptCard
          title="绑定回执"
          fields={[
            ["网关 ID", binding.gatewayId],
            ["客户 ID", binding.customerId],
            ["状态", binding.status],
            ["绑定时间", formatDateTime(binding.boundAt)],
          ]}
        />
      ) : null}
    </SectionCard>
  );
}
