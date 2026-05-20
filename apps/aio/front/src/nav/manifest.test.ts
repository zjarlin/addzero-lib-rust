import {describe, expect, it} from "vitest";
import {aioNavManifest} from "./manifest";

describe("aioNavManifest", () => {
  it("keeps the first migrated release routes registered", () => {
    const routes = aioNavManifest.pages.map(page => page.route).sort();

    expect(routes).toEqual(["/assets", "/config", "/drive", "/gateway", "/software"]);
  });

  it("keeps every page attached to a domain and branch", () => {
    const domains = new Set(aioNavManifest.domains.map(domain => domain.id));
    const branches = new Set(aioNavManifest.branches.map(branch => branch.id));

    for (const page of aioNavManifest.pages) {
      expect(domains.has(page.domainId)).toBe(true);
      expect(branches.has(page.branchId)).toBe(true);
      expect(page.toolbarActions.length).toBeGreaterThan(0);
    }
  });
});
