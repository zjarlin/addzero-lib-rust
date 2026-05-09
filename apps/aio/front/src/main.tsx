import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Navigate, useRoutes } from "react-router-dom";
import "./index.css";
import RootLayout from "./pages/_layout";
import GlobalAiWorkspace from "./components/GlobalAiWorkspace";
import AssetsPage from "./pages/assets";
import ConsolePage from "./pages/console";
import EnvPage from "./pages/env";
import LoginPage from "./pages/login";
import MarketPage from "./pages/market";
import { InstancePluginPage, SystemPluginPage } from "./pages/plugin-page";
import SetupPage from "./pages/setup";
import SkillsPage from "./pages/skills";
import SystemPage from "./pages/system";
import WasmStudioPage from "./pages/wasm-studio";

const routes = [
    { path: "/prototype/wasm-studio", element: <WasmStudioPage /> },
    {
        element: <RootLayout />,
        children: [
            { path: "/setup", element: <SetupPage /> },
            { path: "/login", element: <LoginPage /> },
            { index: true, element: <Navigate to="/assets/notes" replace /> },
            { path: "/assets", element: <Navigate to="/assets/notes" replace /> },
            { path: "/assets/files", element: <Navigate to="/assets/notes" replace /> },
            { path: "/assets/notes", element: <AssetsPage /> },
            { path: "/assets/packages", element: <AssetsPage /> },
            { path: "/assets/dotfiles", element: <AssetsPage /> },
            { path: "/assets/agents", element: <Navigate to="/assets/agents/skills" replace /> },
            { path: "/assets/agents/skills", element: <SkillsPage /> },
            { path: "/assets/agents/cli", element: <MarketPage forcedScene="cli" /> },
            { path: "/assets/agents/mcp", element: <Navigate to="/assets/notes" replace /> },
            { path: "/console", element: <ConsolePage /> },
            { path: "/env", element: <EnvPage /> },
            { path: "/knowledge", element: <Navigate to="/assets/notes" replace /> },
            { path: "/market", element: <MarketPage /> },
            { path: "/market/:scene", element: <MarketPage /> },
            { path: "/skills", element: <Navigate to="/assets/agents/skills" replace /> },
            { path: "/storage", element: <Navigate to="/assets/notes" replace /> },
            { path: "/system", element: <SystemPage /> },
            { path: "/system/:pluginId/:pageId", element: <SystemPluginPage /> },
            { path: "/apps/:instanceSlug/:pageId", element: <InstancePluginPage /> },
        ],
    },
];

function App() {
    const element = useRoutes(routes);
    return (
        <>
            {element}
            <GlobalAiWorkspace />
        </>
    );
}

createRoot(document.getElementById("root")!).render(
    <StrictMode>
        <BrowserRouter>
            <App />
        </BrowserRouter>
    </StrictMode>,
);
