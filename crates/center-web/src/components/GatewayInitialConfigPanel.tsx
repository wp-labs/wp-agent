import type { FormEvent } from "react";
import { useGatewayInitialConfig } from "../hooks";
import {
  ErrorBanner,
  FormField,
  FormStack,
  PrimaryButton,
  ReceiptCard,
  SectionCard,
  TextInput,
} from "./ui";

export function GatewayInitialConfigPanel() {
  const mutation = useGatewayInitialConfig();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      instanceId: String(data.get("instanceId") ?? ""),
      requestedBy: String(data.get("requestedBy") ?? ""),
    });
  }

  const config = mutation.data?.data;

  return (
    <SectionCard
      title="网关初始配置"
      subtitle="查看指定 WarpGateWay 实例的初始配置信息，用于协助排查与配置核对。"
    >
      <form onSubmit={handleSubmit}>
        <FormStack
          actions={
            <PrimaryButton type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "查询中…" : "查询配置"}
            </PrimaryButton>
          }
        >
          <FormField label="实例 ID" hint="例如：inst-7f2a">
            <TextInput name="instanceId" required placeholder="请输入实例 ID" />
          </FormField>
          <FormField label="申请者（requested_by）">
            <TextInput name="requestedBy" defaultValue="admin" required />
          </FormField>
        </FormStack>
      </form>
      {mutation.error ? (
        <ErrorBanner>查询失败：{String(mutation.error)}</ErrorBanner>
      ) : null}
      {config ? (
        <ReceiptCard
          title="初始配置"
          fields={[
            ["控制中心地址", config.controlCenterEndpoint],
            ["策略版本", config.policyVersion],
            ["遥测输出", config.telemetryOutput],
          ]}
        />
      ) : null}
    </SectionCard>
  );
}
