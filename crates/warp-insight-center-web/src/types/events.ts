export interface CreateGatewayInstanceRequested {
  gatewayName: string;
  requestedBy: string;
}

export interface BindGatewayCustomerRequested {
  gatewayId: string;
  customerId: string;
  requestedBy: string;
}

export interface GetGatewayInitialConfigRequested {
  instanceId: string;
  requestedBy: string;
}

export interface PublishWarpAgentdRequested {
  version: string;
  artifactUrl: string;
  requestedBy: string;
}

export interface PublishWarpGateWayRequested {
  version: string;
  artifactUrl: string;
  requestedBy: string;
}

export interface CreateUpgradePlanRequested {
  component: string;
  targetVersion: string;
  gatewayIds: string[];
  requestedBy: string;
}

export interface ApproveUpgradePlanRequested {
  planId: string;
  approvedBy: string;
}

export type AppEvent = CreateGatewayInstanceRequested | BindGatewayCustomerRequested | GetGatewayInitialConfigRequested | PublishWarpAgentdRequested | PublishWarpGateWayRequested | CreateUpgradePlanRequested | ApproveUpgradePlanRequested;
