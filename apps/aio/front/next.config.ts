import type { NextConfig } from "next";

const API_BASE = process.env.AIO_API_BASE ?? "http://127.0.0.1:8787";

const nextConfig: NextConfig = {
    output: "export",
    images: {
        unoptimized: true,
    },
    // Dev-only: proxy /api/* to the Rust backend.
    // In production (Tauri), the Rust server runs on localhost and the
    // api-client calls it directly.
    async rewrites() {
        return [
            {
                source: "/api/:path*",
                destination: `${API_BASE}/api/:path*`,
            },
        ];
    },
};

export default nextConfig;
