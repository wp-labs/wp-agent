import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// /api 代理目标用 WARP_INSIGHT_WEB_PROXY_TARGET 覆盖（默认网关本机 API https://localhost:3000）。
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": {
        target:
          process.env.WARP_INSIGHT_WEB_PROXY_TARGET ?? "https://localhost:3000",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});
