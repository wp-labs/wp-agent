import type { FormEvent } from "react";
import { usePublishWarpAgentd } from "../hooks";
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

export function WarpAgentdReleasePanel() {
  const mutation = usePublishWarpAgentd();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      version: String(data.get("version") ?? ""),
      artifactUrl: String(data.get("artifactUrl") ?? ""),
      requestedBy: String(data.get("requestedBy") ?? ""),
    });
  }

  const release = mutation.data?.data;

  return (
    <SectionCard
      title="WarpAgentd 发布"
      subtitle="平台维护工程师发布 WarpAgentd 新版本，供各 WarpGateWay 实例拉取升级。"
    >
      <form onSubmit={handleSubmit}>
        <FormStack
          actions={
            <PrimaryButton type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? "发布中…" : "发布"}
            </PrimaryButton>
          }
        >
          <FormField label="版本号" hint="例如：v2.4.1">
            <TextInput name="version" required placeholder="请输入版本号" />
          </FormField>
          <FormField label="产物地址（artifact_url）">
            <TextInput
              name="artifactUrl"
              required
              placeholder="https://artifacts.example.com/warp-agentd/v2.4.1"
            />
          </FormField>
          <FormField label="发布者（requested_by）">
            <TextInput name="requestedBy" defaultValue="admin" required />
          </FormField>
        </FormStack>
      </form>
      {mutation.error ? (
        <ErrorBanner>发布失败：{String(mutation.error)}</ErrorBanner>
      ) : null}
      {release ? (
        <ReceiptCard
          title="发布回执"
          fields={[
            ["版本", release.version],
            ["状态", release.status],
            ["发布时间", formatDateTime(release.publishedAt)],
          ]}
        />
      ) : null}
    </SectionCard>
  );
}
