import { create } from "zustand";

type Theme = "dark" | "light";

interface ThemeState {
  theme: Theme;
  toggle: () => void;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: (localStorage.getItem("aio-theme") as Theme) || "dark",
  toggle: () => {
    const next = get().theme === "dark" ? "light" : "dark";
    localStorage.setItem("aio-theme", next);
    document.documentElement.classList.toggle("dark", next === "dark");
    set({ theme: next });
  },
}));

const saved = localStorage.getItem("aio-theme") as Theme | null;
document.documentElement.classList.toggle("dark", saved !== "light");
