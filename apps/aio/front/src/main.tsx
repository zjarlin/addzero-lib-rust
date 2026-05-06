import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, useRoutes } from "react-router-dom";
import "./index.css";
import RootLayout from "./pages/_layout";
import DashboardPage from "./pages";
import ConsolePage from "./pages/console";
import EnvPage from "./pages/env";
import KnowledgePage from "./pages/knowledge";
import LoginPage from "./pages/login";
import MarketPage from "./pages/market";
import SkillsPage from "./pages/skills";
import StoragePage from "./pages/storage";
import SystemPage from "./pages/system";

const routes = [
    {
        element: <RootLayout />,
        children: [
            { path: "/login", element: <LoginPage /> },
            { index: true, element: <DashboardPage /> },
            { path: "/console", element: <ConsolePage /> },
            { path: "/env", element: <EnvPage /> },
            { path: "/knowledge", element: <KnowledgePage /> },
            { path: "/market", element: <MarketPage /> },
            { path: "/skills", element: <SkillsPage /> },
            { path: "/storage", element: <StoragePage /> },
            { path: "/system", element: <SystemPage /> },
        ],
    },
];

function App() {
    return useRoutes(routes);
}

createRoot(document.getElementById("root")!).render(
    <StrictMode>
        <BrowserRouter>
            <App />
        </BrowserRouter>
    </StrictMode>,
);
