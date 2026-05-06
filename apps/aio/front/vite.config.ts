import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import pages from "vite-plugin-pages";

export default defineConfig({
    plugins: [
        react(),
        tailwindcss(),
        pages({
            dirs: "src/pages",
            routeStyle: "next",
        }),
    ],
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
