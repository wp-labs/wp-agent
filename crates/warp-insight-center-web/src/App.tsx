import { Routes, Route, Navigate } from "react-router-dom";

import { GatewayListPage } from "./components/GatewayListPage";
import { GatewayDetailPage } from "./components/GatewayDetailPage";
import { GatewayInstancePage } from "./components/GatewayInstancePage";
import { GatewayConfigPage } from "./components/GatewayConfigPage";
import { ReleasePage } from "./components/ReleasePage";
import { UpgradePlanPage } from "./components/UpgradePlanPage";

export function App() {
  return (
    <Routes>
      <Route path="/" element={<GatewayListPage />} />
      <Route path="/gateways/:gatewayId" element={<GatewayDetailPage />} />
      <Route path="/instance" element={<GatewayInstancePage />} />
      <Route path="/config" element={<GatewayConfigPage />} />
      <Route path="/release" element={<ReleasePage />} />
      <Route path="/upgrade-plan" element={<UpgradePlanPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
