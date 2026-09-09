import type { GatewayHealth } from "../api";
import { Badge } from "./ui";

export function GatewayHealthBadge({ value }: { value: GatewayHealth }) {
  if (value === "healthy") return <Badge tone="green">健康</Badge>;
  if (value === "degraded") return <Badge tone="amber">降级</Badge>;
  if (value === "unhealthy") return <Badge tone="red">不健康</Badge>;
  return <Badge tone="gray">未知</Badge>;
}
