import { useMemo } from "react";
import { useGatewayList, useGatewayStatusView, useGatewayUptimes } from "../hooks";
import { isRateLimitedError } from "../api";
import { ErrorBanner, PageShell } from "./ui";
import { GatewayStatusList } from "./GatewayStatusList";
import {
  ExampleDataTag,
  GatewayStatusOverviewMetrics,
} from "./GatewayStatusOverviewMetrics";

export function GatewayListPage() {
  const { data: statusData, isLoading, isError, error } = useGatewayStatusView();
  const { data: listData } = useGatewayList();

  const gatewayIds = useMemo(
    () => (statusData?.data ?? []).map((gateway) => gateway.gatewayId),
    [statusData],
  );
  const { data: uptimesData } = useGatewayUptimes(gatewayIds);
  const uptimes = uptimesData ?? {};

  const averageUptime = useMemo(() => {
    const values = Object.values(uptimes).filter(
      (value): value is number => typeof value === "number",
    );
    if (values.length === 0) return null;
    return values.reduce((sum, value) => sum + value, 0) / values.length;
  }, [uptimes]);

  return (
    <PageShell
      title="网关列表"
      summary="查看各 WarpGateWay 实例的在线状态、版本、健康状态与最后上报时间，形成全局网关视图。"
    >
      {isError ? (
        isRateLimitedError(error) ? (
          <ErrorBanner>访问过于频繁，请稍候重试。</ErrorBanner>
        ) : (
          <ErrorBanner>无法连接 WarpInsightCenter 管理服务，请确认后端已启动。</ErrorBanner>
        )
      ) : null}
      <GatewayStatusOverviewMetrics list={listData?.data} averageUptime={averageUptime} />
      <ExampleDataTag source={statusData?.source} />
      <GatewayStatusList
        items={statusData?.data}
        uptimes={uptimes}
        loading={isLoading}
      />
    </PageShell>
  );
}
