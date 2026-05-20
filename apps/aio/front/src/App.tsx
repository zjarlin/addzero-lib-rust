import Button from "@jetbrains/ring-ui-built/components/button/button";
import LoaderInline from "@jetbrains/ring-ui-built/components/loader-inline/loader-inline";
import {useMemo, useState} from "react";
import {Navigate, Route, Routes, useLocation, useNavigate} from "react-router-dom";
import {Toolbar} from "./components/Toolbar";
import {AssetsPage} from "./features/assets/AssetsPage";
import {ConfigPage} from "./features/config/ConfigPage";
import {DrivePage} from "./features/drive/DrivePage";
import {GatewayPage} from "./features/gateway/GatewayPage";
import {SoftwarePage} from "./features/software/SoftwarePage";
import {aioNavManifest, pageForRoute} from "./nav/manifest";

export default function App() {
  const location = useLocation();
  const navigate = useNavigate();
  const currentPage = pageForRoute(location.pathname);
  const [busyAction, setBusyAction] = useState<string>();

  const currentDomain = aioNavManifest.domains.find(domain => domain.id === currentPage.domainId) ?? aioNavManifest.domains[0];
  const branchGroups = useMemo(() => {
    const pages = aioNavManifest.pages.filter(page => page.domainId === currentDomain.id).sort((left, right) => left.order - right.order);
    return aioNavManifest.branches
      .filter(branch => branch.domainId === currentDomain.id)
      .sort((left, right) => left.order - right.order)
      .map(branch => ({
        ...branch,
        pages: pages.filter(page => page.branchId === branch.id)
      }))
      .filter(branch => branch.pages.length > 0);
  }, [currentDomain.id]);

  const cards = aioNavManifest.summaryCards
    .filter(card => aioNavManifest.pages.find(page => page.route === card.route)?.domainId === currentDomain.id)
    .sort((left, right) => left.order - right.order);

  const handleToolbar = async (actionId: string) => {
    const handler = (window as Window & {__AIO_PAGE_ACTION__?: (actionId: string) => Promise<void>}).__AIO_PAGE_ACTION__;
    if (!handler) {
      return;
    }
    setBusyAction(actionId);
    try {
      await handler(actionId);
    } finally {
      setBusyAction(undefined);
    }
  };

  return (
    <div className="appShell">
      <header className="topbar">
        <div>
          <div className="eyebrow">AIO Desktop</div>
          <h1>{currentPage.title}</h1>
          <p>{currentPage.subtitle}</p>
        </div>
        <div className="domainTabs">
          {aioNavManifest.domains.sort((left, right) => left.order - right.order).map(domain => (
            <Button
              key={domain.id}
              primary={domain.id === currentDomain.id}
              onClick={() => navigate(domain.defaultRoute)}
            >
              {domain.label}
            </Button>
          ))}
        </div>
      </header>
      <Toolbar actions={currentPage.toolbarActions} busy={busyAction} onAction={handleToolbar} />
      <div className="workbench">
        <aside className="navRail">
          {branchGroups.map(branch => (
            <section key={branch.id} className="branchGroup">
              <div className="branchLabel">{branch.label}</div>
              {branch.pages.map(page => (
                <button
                  key={page.id}
                  className={page.route === currentPage.route ? "navItem active" : "navItem"}
                  onClick={() => navigate(page.route)}
                  type="button"
                >
                  <span>{page.title}</span>
                  <small>{page.subtitle}</small>
                </button>
              ))}
            </section>
          ))}
        </aside>
        <main className="contentPane">
          <Routes>
            <Route path="/" element={<Navigate to="/drive" replace />} />
            <Route path="/drive" element={<DrivePage />} />
            <Route path="/gateway" element={<GatewayPage />} />
            <Route path="/assets" element={<AssetsPage />} />
            <Route path="/software" element={<SoftwarePage />} />
            <Route path="/config" element={<ConfigPage />} />
          </Routes>
        </main>
        <aside className="summaryRail">
          {busyAction ? <LoaderInline /> : null}
          {cards.map(card => (
            <section key={card.id} className="summaryCard">
              <div className="summaryTitle">{card.title}</div>
              <p>{card.summary}</p>
              <Button onClick={() => navigate(card.route)}>Open</Button>
            </section>
          ))}
        </aside>
      </div>
    </div>
  );
}
