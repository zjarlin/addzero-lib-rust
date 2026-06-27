#![forbid(unsafe_code)]

use az_aio_platform::plugin::api::AdminMenuTree;
use dioxus::prelude::*;

mod components;

use components::AppLayout;

/// Render full HTML page.
pub fn render_app_html(
    snapshot: &az_aio_platform::plugin::host::HostSnapshot,
    route: &str,
    query: &str,
) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        AppLayout {
            renderers: snapshot.native_renderers.clone(),
            admin_menu_tree: snapshot_admin_menu_tree(snapshot),
            pages: snapshot.pages.clone(),
            route: route.to_string(),
            query: query.to_string(),
        }
    });

    format!(
        concat!(
            "<!DOCTYPE html>\n",
            "<html lang=\"zh-CN\" data-theme=\"light\">\n",
            "<head>\n",
            "    <meta charset=\"utf-8\">\n",
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
            "    <title>AIO</title>\n",
            "    <link rel=\"stylesheet\" href=\"/assets/app.css\">\n",
            "</head>\n",
            "<body>\n",
            "    {body}\n",
            "    <script>{script}</script>\n",
            "</body>\n",
            "</html>",
        ),
        body = body,
        script = shell_bootstrap_script(),
    )
}

fn snapshot_admin_menu_tree(
    snapshot: &az_aio_platform::plugin::host::HostSnapshot,
) -> AdminMenuTree {
    snapshot.admin_menu_tree.clone()
}

fn shell_bootstrap_script() -> &'static str {
    r#"
(function(){
  var root = document.documentElement;
  var savedTheme = localStorage.getItem('az-theme');
  if (savedTheme) {
    root.setAttribute('data-theme', savedTheme);
  }

  var themeButton = document.getElementById('theme-toggle');
  if (themeButton) {
    themeButton.onclick = function() {
      var nextTheme = root.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
      root.setAttribute('data-theme', nextTheme);
      localStorage.setItem('az-theme', nextTheme);
      return false;
    };
  }

  var shell = document.querySelector('.shell');
  var sidebar = document.getElementById('sidebar-toggle');
  function setCollapsed(collapsed) {
    if (!shell) {
      return;
    }
    shell.classList.toggle('shell--collapsed', collapsed);
    if (sidebar) {
      sidebar.setAttribute('aria-expanded', String(!collapsed));
    }
  }
  setCollapsed(localStorage.getItem('az-sidebar-collapsed') === 'true');
  if (sidebar) {
    sidebar.onclick = function() {
      var next = !shell.classList.contains('shell--collapsed');
      setCollapsed(next);
      localStorage.setItem('az-sidebar-collapsed', String(next));
      return false;
    };
  }

  var search = document.getElementById('admin-menu-search');
  if (!search) {
    return;
  }
  function normalize(value) {
    return String(value || '').toLowerCase().trim();
  }
  function itemMatches(item, query) {
    if (!query) {
      return true;
    }
    return normalize(item.getAttribute('data-menu-text')).indexOf(query) >= 0;
  }
  function parentMenuNode(item) {
    var parent = item.parentElement;
    while (parent) {
      if (parent.matches && parent.matches('[data-menu-node]')) {
        return parent;
      }
      parent = parent.parentElement;
    }
    return null;
  }
  function hasMatchedAncestor(item) {
    var parent = parentMenuNode(item);
    while (parent) {
      if (parent.getAttribute('data-menu-match') === 'self') {
        return true;
      }
      parent = parentMenuNode(parent);
    }
    return false;
  }
  function refreshMenuSearch() {
    var query = normalize(search.value);
    var nodes = Array.prototype.slice.call(document.querySelectorAll('[data-menu-node]'));
    nodes.forEach(function(item) {
      item.hidden = false;
      item.setAttribute('data-menu-match', itemMatches(item, query) ? 'self' : '');
    });
    nodes.forEach(function(item) {
      if (!query) {
        item.removeAttribute('data-menu-match');
        return;
      }
      var ownMatched = item.getAttribute('data-menu-match') === 'self';
      var childMatched = !!item.querySelector('[data-menu-node][data-menu-match="self"]');
      var ancestorMatched = hasMatchedAncestor(item);
      var matched = ownMatched || childMatched || ancestorMatched;
      item.hidden = !matched;
      if ((ownMatched || childMatched) && item.tagName.toLowerCase() === 'details') {
        item.open = true;
      }
    });
    nodes.forEach(function(item) {
      item.removeAttribute('data-menu-match');
    });
    document.querySelectorAll('[data-menu-domain]').forEach(function(domain) {
      var domainMatched = itemMatches(domain, query);
      var panel = domain.nextElementSibling;
      var childMatched = panel && !!panel.querySelector('[data-menu-node]:not([hidden])');
      domain.hidden = !!query && !domainMatched && !childMatched;
      if (panel) {
        panel.hidden = !!query && !domainMatched && !childMatched;
      }
    });
  }
  search.addEventListener('input', refreshMenuSearch);
  refreshMenuSearch();
})();
"#
}
