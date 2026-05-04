'use client';

import { Brain, FolderOpen } from 'lucide-react';

export default function AdminHome() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">Welcome to MSC AIO Admin</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border bg-card text-card-foreground shadow-sm p-6">
          <div className="flex flex-row items-center justify-between space-y-0 pb-2">
            <h3 className="tracking-tight text-sm font-medium">Total Skills</h3>
            <Brain className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="text-2xl font-bold">0</div>
          <p className="text-xs text-muted-foreground">Skills loaded from system</p>
        </div>

        <div className="rounded-xl border bg-card text-card-foreground shadow-sm p-6">
          <div className="flex flex-row items-center justify-between space-y-0 pb-2">
            <h3 className="tracking-tight text-sm font-medium">Active Resources</h3>
            <FolderOpen className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="text-2xl font-bold">0</div>
          <p className="text-xs text-muted-foreground">Deployed resources</p>
        </div>
      </div>

      <div className="rounded-xl border bg-card text-card-foreground shadow-sm">
        <div className="p-6">
          <h3 className="text-lg font-semibold mb-4">Quick Actions</h3>
          <div className="grid gap-4 md:grid-cols-2">
            <button className="flex items-center gap-3 p-4 rounded-lg border hover:bg-muted transition-colors">
              <Brain className="w-5 h-5" />
              <div className="text-left">
                <div className="font-medium">Manage Skills</div>
                <div className="text-sm text-muted-foreground">View and manage skills</div>
              </div>
            </button>
            <button className="flex items-center gap-3 p-4 rounded-lg border hover:bg-muted transition-colors">
              <FolderOpen className="w-5 h-5" />
              <div className="text-left">
                <div className="font-medium">Manage Resources</div>
                <div className="text-sm text-muted-foreground">View deployment paths</div>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
