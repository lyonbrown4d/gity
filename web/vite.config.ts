import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { compression } from "vite-plugin-compression2";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function normalizeProxyTarget(raw: string | undefined): string {
  if (!raw || raw.trim().length === 0) {
    return "http://127.0.0.1:8080";
  }
  try {
    const parsed = new URL(raw);
    return parsed.origin;
  } catch {
    return raw;
  }
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const rawTarget = env.VITE_DEV_API_PROXY_TARGET || env.DEV_API_PROXY_TARGET;
  const proxyTarget = normalizeProxyTarget(rawTarget);

  return {
    plugins: [
      react(),
      compression({
        threshold: 10 * 1024,
        algorithms: ["gzip", "brotliCompress"],
        deleteOriginalAssets: false,
      }),
    ],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
    build: {
      rollupOptions: {
        output: {
          manualChunks(id) {
            if (!id.includes("node_modules")) {
              return;
            }
            if (id.includes("@refinedev") || id.includes("@tanstack/react-query")) {
              return "refine";
            }
            if (id.includes("@radix-ui")) {
              return "radix";
            }
            if (id.includes("lucide-react")) {
              return "icons";
            }
          },
        },
      },
    },
    server: {
      port: 5173,
      proxy: {
        "/api": {
          target: proxyTarget,
          changeOrigin: true,
        },
        "/git": {
          target: proxyTarget,
          changeOrigin: true,
        },
      },
    },
  };
});
