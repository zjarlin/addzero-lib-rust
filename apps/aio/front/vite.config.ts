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
        },
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
