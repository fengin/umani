import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // 4. Build optimizations — chunk splitting for smaller main bundle
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Monaco Editor 体积很大，单独分包
          monaco: ["@monaco-editor/react"],
          // React 生态独立分包
          vendor: ["react", "react-dom", "react-router-dom"],
          // 状态管理 + 国际化
          utils: ["zustand", "i18next", "react-i18next"],
        },
      },
    },
    // 生产环境关闭 sourcemap 减小体积
    sourcemap: false,
    // chunk 大小警告阈值 (Monaco 较大)
    chunkSizeWarningLimit: 1500,
  },
}));

