import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// /api 代理目标用 WARP_INSIGHT_WEB_PROXY_TARGET 覆盖（默认 127.0.0.1:3100）。
// demo-gateway.sh 用独立端口 + 指向自己的 center，避免与 demo-insight-center 共用环境。
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": {
        target: process.env.WARP_INSIGHT_WEB_PROXY_TARGET ?? "http://127.0.0.1:3100",
        changeOrigin: true,
      },
    },
  },
});
