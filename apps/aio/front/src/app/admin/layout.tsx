'use client';

import React, { useState, useEffect, useCallback } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { LayoutDashboard, Brain, FolderOpen, Settings, Menu, X } from 'lucide-react';

interface MenuNode {
  id: string;
  route_path: string;
  title: string;
  icon?: string;
  children: MenuNode[];
}

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  LayoutDashboard,
  Brain,
  FolderOpen,
  Settings,
};

export default function AdminLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [menuTree, setMenuTree] = useState<MenuNode[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchMenuTree = useCallback(async () => {
    try {
      const res = await fetch('/api/admin/menus/tree');
      if (res.ok) {
        const data = await res.json();
        setMenuTree(data);
      }
    } catch (error) {
      console.error('Failed to fetch menu tree:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchMenuTree();
  }, [fetchMenuTree]);

  const getIcon = (iconName?: string) => {
    if (!iconName) return LayoutDashboard;
    return iconMap[iconName] || LayoutDashboard;
  };

  const renderMenuItem = (item: MenuNode, level: number = 0) => {
    const Icon = getIcon(item.icon);
    const isActive = pathname === item.route_path;
    const hasChildren = item.children && item.children.length > 0;

    return (
      <div key={item.id} className="w-full">
        <Link
          href={item.route_path}
          className={`flex items-center gap-3 px-4 py-2.5 rounded-lg transition-colors ${
            isActive
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
          style={{ paddingLeft: `${level * 16 + 16}px` }}
        >
          <Icon className="w-5 h-5" />
          <span className="font-medium">{item.title}</span>
        </Link>
        {hasChildren && (
          <div className="mt-1">
            {item.children.map((child) => renderMenuItem(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Mobile menu button */}
      <div className="lg:hidden fixed top-0 left-0 right-0 z-50 bg-background border-b p-4">
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          className="p-2 rounded-lg hover:bg-muted"
        >
          {sidebarOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
        </button>
      </div>

      {/* Sidebar */}
      <aside
        className={`fixed top-0 left-0 z-40 h-screen bg-card border-r transition-transform duration-300 ease-in-out ${
          sidebarOpen ? 'translate-x-0' : '-translate-x-full'
        } lg:translate-x-0 w-64`}
      >
        <div className="flex flex-col h-full">
          {/* Logo */}
          <div className="flex items-center gap-3 p-6 border-b">
            <div className="w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
              <LayoutDashboard className="w-5 h-5 text-primary-foreground" />
            </div>
            <span className="text-xl font-bold">MSC AIO</span>
          </div>

          {/* Navigation */}
          <nav className="flex-1 overflow-y-auto p-4 space-y-1">
            {loading ? (
              <div className="text-sm text-muted-foreground px-4">Loading menu...</div>
            ) : (
              menuTree.map((item) => renderMenuItem(item))
            )}
          </nav>

          {/* Footer */}
          <div className="p-4 border-t">
            <Link
              href="/"
              className="flex items-center gap-3 px-4 py-2.5 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            >
              <Settings className="w-5 h-5" />
              <span className="font-medium">Back to Home</span>
            </Link>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className={`lg:ml-64 min-h-screen ${sidebarOpen ? 'lg:ml-64' : ''}`}>
        <div className="p-6 lg:p-8">
          {children}
        </div>
      </main>
    </div>
  );
}
