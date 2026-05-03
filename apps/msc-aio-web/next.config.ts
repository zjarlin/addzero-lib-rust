import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  images: {
    unoptimized: true,
  },
  transpilePackages: ["@addzero/admin-shell", "@addzero/api-client"],
};

export default nextConfig;
