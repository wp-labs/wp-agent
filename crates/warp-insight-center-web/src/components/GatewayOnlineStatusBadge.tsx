import type { GatewayStatus } from "../api";
import { Badge } from "./ui";

export function GatewayOnlineStatusBadge({ value }: { value: GatewayStatus }) {
  if (value === "online") {
    return <Badge tone="green">在线</Badge>;
  }
  return <Badge tone="gray">离线</Badge>;
}
