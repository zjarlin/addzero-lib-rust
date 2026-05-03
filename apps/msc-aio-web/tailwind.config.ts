import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{ts,tsx}",
    "./src/**/*.{ts,tsx}",
    "../../packages/admin-shell/src/**/*.{ts,tsx}",
    "../../packages/remote-ui/src/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        panel: "#0f1724",
        ink: "#edf2ff",
        muted: "#94a3b8",
      },
      boxShadow: {
        panel: "0 30px 80px rgba(0,0,0,0.35)",
      },
    },
  },
  plugins: [],
};

export default config;
