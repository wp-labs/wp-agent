import { useEffect, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  ADMIN_AUTH_CHANGED_EVENT,
  approveUpgradePlan,
  bindGatewayCustomer,
  createGatewayInstance,
  createUpgradePlan,
  fetchGatewayAgents,
  fetchGatewayList,
  fetchGatewayInitialConfig,
  fetchGatewayStatus,
  fetchGatewayStatusView,
  fetchGatewayUptime,
  getAdminApiToken,
  publishWarpAgentd,
  publishWarpGateWay,
  type ApproveUpgradePlanCommand,
  type BindGatewayCustomerCommand,
  type CreateGatewayInstanceCommand,
  type CreateUpgradePlanCommand,
  type GetGatewayInitialConfigCommand,
  type PublishReleaseCommand,
} from "../api";

// 当 Admin Token 变化时触发重渲染，使查询能立即从禁用切到启用。
function useAuthVersion() {
  const [, setAuthVersion] = useState(0);
  useEffect(() => {
    const onAuthChanged = () => setAuthVersion((version) => version + 1);
    window.addEventListener(ADMIN_AUTH_CHANGED_EVENT, onAuthChanged);
    return () =>
      window.removeEventListener(ADMIN_AUTH_CHANGED_EVENT, onAuthChanged);
  }, []);
}

export function useGatewayStatusView() {
  useAuthVersion();
  const enabled = Boolean(getAdminApiToken());
  return useQuery({
    queryKey: ["gateway-status-view"],
    queryFn: fetchGatewayStatusView,
    // 有真实后端时 5s 轮询刷新；未配置 token 时也能以 example 数据渲染。
    refetchInterval: enabled ? 5_000 : 30_000,
  });
}

export function useGatewayStatus(gatewayId: string) {
  useAuthVersion();
  const enabled = Boolean(getAdminApiToken());
  return useQuery({
    queryKey: ["gateway-status", gatewayId],
    queryFn: () => fetchGatewayStatus(gatewayId),
    refetchInterval: enabled ? 5_000 : 30_000,
    enabled: Boolean(gatewayId),
  });
}

export function useGatewayAgents(gatewayId: string) {
  useAuthVersion();
  const enabled = Boolean(getAdminApiToken());
  return useQuery({
    queryKey: ["gateway-agents", gatewayId],
    queryFn: () => fetchGatewayAgents(gatewayId),
    refetchInterval: enabled ? 5_000 : 30_000,
    enabled: Boolean(gatewayId),
  });
}

export function useGatewayList() {
  useAuthVersion();
  const enabled = Boolean(getAdminApiToken());
  return useQuery({
    queryKey: ["gateway-list"],
    queryFn: fetchGatewayList,
    refetchInterval: enabled ? 5_000 : 30_000,
  });
}

/** 一次拉取多个网关的在线率，返回 { gateway_id: uptime|null } 映射（列表页用）。 */
export function useGatewayUptimes(gatewayIds: string[]) {
  useAuthVersion();
  const enabled = Boolean(getAdminApiToken()) && gatewayIds.length > 0;
  return useQuery({
    queryKey: ["gateway-uptimes", gatewayIds],
    queryFn: async () => {
      const results = await Promise.all(
        gatewayIds.map((id) => fetchGatewayUptime(id)),
      );
      const map: Record<string, number | null> = {};
      for (const result of results) {
        map[result.data.gatewayId] = result.data.uptime;
      }
      return map;
    },
    enabled,
    refetchInterval: enabled ? 5_000 : 30_000,
  });
}

export function useCreateGatewayInstance() {
  return useMutation({
    mutationFn: (command: CreateGatewayInstanceCommand) =>
      createGatewayInstance(command),
  });
}

export function useBindGatewayCustomer() {
  return useMutation({
    mutationFn: (command: BindGatewayCustomerCommand) =>
      bindGatewayCustomer(command),
  });
}

export function useGatewayInitialConfig() {
  return useMutation({
    mutationFn: (command: GetGatewayInitialConfigCommand) =>
      fetchGatewayInitialConfig(command),
  });
}

export function usePublishWarpAgentd() {
  return useMutation({
    mutationFn: (command: PublishReleaseCommand) => publishWarpAgentd(command),
  });
}

export function usePublishWarpGateWay() {
  return useMutation({
    mutationFn: (command: PublishReleaseCommand) =>
      publishWarpGateWay(command),
  });
}

export function useCreateUpgradePlan() {
  return useMutation({
    mutationFn: (command: CreateUpgradePlanCommand) => createUpgradePlan(command),
  });
}

export function useApproveUpgradePlan() {
  return useMutation({
    mutationFn: (command: ApproveUpgradePlanCommand) =>
      approveUpgradePlan(command),
  });
}
