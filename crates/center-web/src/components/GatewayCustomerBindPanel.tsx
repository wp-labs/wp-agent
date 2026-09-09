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

/** 绑定客户到网关实例；详情页传入 gatewayId 时只允许编辑客户信息。 */
export function GatewayCustomerBindPanel({
  gatewayId,
}: {
  gatewayId?: string;
}) {
  const mutation = useBindGatewayCustomer();
  const currentGatewayId = gatewayId?.trim() ?? "";

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      gatewayId: currentGatewayId || String(data.get("gatewayId") ?? ""),
      customerId: String(data.get("customerId") ?? ""),
      // 绑定接口仍要求 requested_by；当前管理端没有独立用户字段，沿用默认管理身份。
      requestedBy: "admin",
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
          {!currentGatewayId ? (
            <FormField label="网关 ID" hint="例如：gw-001">
              <TextInput
                name="gatewayId"
                required
                placeholder="请输入网关 ID"
              />
            </FormField>
          ) : null}
          <FormField label="客户 ID" hint="例如：cust-acme">
            <TextInput name="customerId" required placeholder="请输入客户 ID" />
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
