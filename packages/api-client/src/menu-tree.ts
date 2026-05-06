// Types and client methods for the menu-tree endpoint (GET /api/admin/menus/tree).

/** Flattened menu tree node as returned by GET /api/admin/menus/tree */
export interface MenuTreeNodeDto {
    id: string;
    route_path: string;
    title: string;
    icon: string | null;
    parent_id: string | null;
    sort_order: number;
    visible: boolean;
    metadata: unknown;
    created_at: string;
    updated_at: string;
    children: MenuTreeNodeDto[];
}

async function request<T>(baseUrl: string, path: string): Promise<T> {
    const response = await fetch(`${baseUrl}${path}`, {
        credentials: "include",
        headers: { "Content-Type": "application/json" },
    });
    if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
    }
    return (await response.json()) as T;
}

export interface MenuTreeApi {
    getMenuTree(): Promise<MenuTreeNodeDto[]>;
}

export function createMenuTreeApi(baseUrl: string): MenuTreeApi {
    return {
        getMenuTree: () =>
            request<MenuTreeNodeDto[]>(baseUrl, "/api/admin/menus/tree"),
    };
}
