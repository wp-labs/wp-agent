import type { GatewayStatus } from "../api";
import { Badge } from "./ui";

/** 将网关连接状态映射为一致的语义色徽标。 */
export function GatewayOnlineStatusBadge({ value }: { value: GatewayStatus }) {
  if (value === "online") {
    return <Badge tone="green">在线</Badge>;
  }
  return <Badge tone="red">离线</Badge>;
}
