import {useCallback, useEffect, useMemo, useState} from "react";
import {Toolbar} from "./components/Toolbar";
import {Icon} from "./components/Icon";
import {AssetsPage} from "./features/assets/AssetsPage";
import {ConfigPage} from "./features/config/ConfigPage";
import {DrivePage} from "./features/drive/DrivePage";
import {GatewayPage} from "./features/gateway/GatewayPage";
import {SoftwarePage} from "./features/software/SoftwarePage";
import {aioNavManifest, pageForRoute, sortedPages} from "./nav/manifest";
import type {AioPage} from "./types";

interface ShellPreferences {
  sidebarCollapsed: boolean;
  compactDensity: boolean;
  darkMode: boolean;
}

const defaultPreferences: ShellPreferences = {
  sidebarCollapsed: false,
  compactDensity: true,
  darkMode: false
};

export default function App() {
  const [route, setRoute] = useState(() => normalizeRoute(window.location.pathname));
  const [busyAction, setBusyAction] = useState<string>();
  const [preferences, setPreferences] = useState<ShellPreferences>(defaultPreferences);
  const [preferencesOpen, setPreferencesOpen] = useState(false);

  useEffect(() => {
    const handlePopState = () => setRoute(normalizeRoute(window.location.pathname));
    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, []);

  const currentPage = pageForRoute(route);
  const currentDomain = aioNavManifest.domains.find(domain => domain.id === currentPage.domainId) ?? aioNavManifest.domains[0];
  const openPages = useMemo(() => {
    const pinnedPages = sortedPages.filter(page => page.pinned);
    const domainPages = sortedPages.filter(page => page.domainId === currentDomain.id);
    return uniquePages([...pinnedPages, ...domainPages, currentPage]);
  }, [currentDomain.id, currentPage]);

  const branchGroups = useMemo(() => {
    const pages = sortedPages.filter(page => page.domainId === currentDomain.id);
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

  const navigate = useCallback((nextRoute: string) => {
    const normalizedRoute = normalizeRoute(nextRoute);
    if (normalizedRoute === route) {
      return;
    }
    window.history.pushState({}, "", normalizedRoute);
    setRoute(normalizedRoute);
  }, [route]);

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

  const shellClassName = [
    "appShell",
    preferences.sidebarCollapsed ? "sidebarCollapsed" : "",
    preferences.compactDensity ? "compactDensity" : "",
    preferences.darkMode ? "darkMode" : "lightMode"
  ].filter(Boolean).join(" ");

  return (
    <div className={shellClassName}>
      <aside className="sidebarNav">
        <div className="brandBlock">
          <div className="brandMark">A</div>
          <div className="brandCopy">
            <span>AIO</span>
            <small>Desktop Admin</small>
          </div>
        </div>
        <nav className="domainNav" aria-label="Primary context">
          {[...aioNavManifest.domains].sort((left, right) => left.order - right.order).map(domain => (
            <button
              key={domain.id}
              className={domain.id === currentDomain.id ? "domainItem active" : "domainItem"}
              onClick={() => navigate(domain.defaultRoute)}
              title={domain.label}
              type="button"
            >
              <Icon name={domain.icon} />
              <span>{domain.label}</span>
            </button>
          ))}
        </nav>
        <nav className="contextTree" aria-label="Side context tree">
          {branchGroups.map(branch => (
            <section key={branch.id} className="branchGroup">
              <div className="branchLabel">
                <Icon name={branch.icon} size={14} />
                <span>{branch.label}</span>
              </div>
              {branch.pages.map(page => (
                <button
                  key={page.id}
                  className={page.route === currentPage.route ? "navItem active" : "navItem"}
                  onClick={() => navigate(page.route)}
                  title={`${page.title}: ${page.subtitle}`}
                  type="button"
                >
                  <Icon name={page.icon} />
                  <span>{page.title}</span>
                  <Icon className="navItemChevron" name="ChevronRight" size={14} />
                </button>
              ))}
            </section>
          ))}
        </nav>
      </aside>
      <div className="workspaceShell">
        <header className="topbar">
          <div className="topbarLeading">
            <button
              className="iconButton"
              onClick={() => setPreferences(previous => ({...previous, sidebarCollapsed: !previous.sidebarCollapsed}))}
              title={preferences.sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
              type="button"
            >
              <Icon name={preferences.sidebarCollapsed ? "PanelLeftOpen" : "PanelLeftClose"} />
            </button>
            <div>
              <div className="breadcrumbLine">
                <span>{currentDomain.label}</span>
                <Icon name="ChevronRight" size={13} />
                <span>{currentPage.title}</span>
              </div>
              <h1>{currentPage.title}</h1>
            </div>
          </div>
          <div className="topbarActions">
            <button className="searchBox" type="button" title="Search current workspace">
              <Icon name="Search" />
              <span>Search</span>
              <kbd>/</kbd>
            </button>
            <button
              className="iconButton"
              onClick={() => setPreferences(previous => ({...previous, darkMode: !previous.darkMode}))}
              title={preferences.darkMode ? "Switch to light mode" : "Switch to dark mode"}
              type="button"
            >
              <Icon name={preferences.darkMode ? "Sun" : "Moon"} />
            </button>
            <button className="iconButton" onClick={() => setPreferencesOpen(true)} title="Open preferences" type="button">
              <Icon name="Settings" />
            </button>
          </div>
        </header>
        <div className="tabbar" role="tablist" aria-label="Open pages">
          {openPages.map(page => (
            <button
              key={page.id}
              aria-selected={page.route === currentPage.route}
              className={page.route === currentPage.route ? "pageTab active" : "pageTab"}
              onClick={() => navigate(page.route)}
              role="tab"
              type="button"
            >
              <Icon name={page.icon} size={14} />
              <span>{page.title}</span>
            </button>
          ))}
        </div>
        <div className="operationBar">
          <p>{currentPage.subtitle}</p>
          <Toolbar actions={currentPage.toolbarActions} busy={busyAction} onAction={handleToolbar} />
        </div>
        <div className="workbench">
          <main className="contentPane">{renderPage(currentPage)}</main>
          <aside className="summaryRail">
            <div className="summaryHeader">
              <Icon name="SlidersHorizontal" />
              <span>Context</span>
            </div>
            {busyAction ? (
              <div className="busyLine">
                <span className="buttonSpinner" />
                <span>Running action</span>
              </div>
            ) : null}
            {cards.map(card => (
              <button key={card.id} className="summaryCard" onClick={() => navigate(card.route)} type="button">
                <span className="summaryTitle">{card.title}</span>
                <span>{card.summary}</span>
              </button>
            ))}
          </aside>
        </div>
      </div>
      {preferencesOpen ? (
        <div className="drawerBackdrop" onClick={() => setPreferencesOpen(false)}>
          <aside className="preferencesDrawer" onClick={event => event.stopPropagation()}>
            <div className="drawerHeader">
              <div>
                <span>Preferences</span>
                <small>Shell layout</small>
              </div>
              <button className="iconButton" onClick={() => setPreferencesOpen(false)} title="Close preferences" type="button">
                <Icon name="X" />
              </button>
            </div>
            <PreferenceSwitch
              checked={preferences.sidebarCollapsed}
              label="Collapsed sidebar"
              onChange={() => setPreferences(previous => ({...previous, sidebarCollapsed: !previous.sidebarCollapsed}))}
            />
            <PreferenceSwitch
              checked={preferences.compactDensity}
              label="Compact density"
              onChange={() => setPreferences(previous => ({...previous, compactDensity: !previous.compactDensity}))}
            />
            <PreferenceSwitch
              checked={preferences.darkMode}
              label="Dark mode"
              onChange={() => setPreferences(previous => ({...previous, darkMode: !previous.darkMode}))}
            />
          </aside>
        </div>
      ) : null}
    </div>
  );
}

function PreferenceSwitch({checked, label, onChange}: {checked: boolean; label: string; onChange: () => void}) {
  return (
    <label className="preferenceSwitch">
      <span>{label}</span>
      <button aria-pressed={checked} className={checked ? "switchControl active" : "switchControl"} onClick={onChange} type="button">
        <span />
      </button>
    </label>
  );
}

function renderPage(page: AioPage) {
  switch (page.route) {
    case "/drive":
      return <DrivePage />;
    case "/gateway":
      return <GatewayPage />;
    case "/assets":
      return <AssetsPage />;
    case "/software":
      return <SoftwarePage />;
    case "/config":
      return <ConfigPage />;
    default:
      return <DrivePage />;
  }
}

function normalizeRoute(route: string) {
  if (route === "/") {
    return "/drive";
  }
  return pageForRoute(route).route;
}

function uniquePages(pages: AioPage[]) {
  const seen = new Set<string>();
  return pages.filter(page => {
    if (seen.has(page.id)) {
      return false;
    }
    seen.add(page.id);
    return true;
  });
}
