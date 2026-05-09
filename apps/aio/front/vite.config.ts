import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

export default defineConfig({
    plugins: [
        react(),
        tailwindcss(),
    ],
    resolve: {
        dedupe: ["react", "react-dom"],
        alias: {
            "@": path.resolve(__dirname, "./src"),
            "@az/admin-shell": path.resolve(
                __dirname,
                "../../../packages/admin-shell/src/index.ts",
            ),
            "@az/api-client": path.resolve(
                __dirname,
                "../../../packages/api-client/src/index.ts",
            ),
        },
    },
    optimizeDeps: {
        exclude: [
            "@az/admin-shell",
            "@az/api-client",
            "@az/ui",
        ],
    },
    server: {
        port: 1430,
        proxy: {
            "/api": {
                target: "http://127.0.0.1:8787",
                changeOrigin: true,
            },
        },
    },
    build: {
        outDir: "dist",
    },
});
