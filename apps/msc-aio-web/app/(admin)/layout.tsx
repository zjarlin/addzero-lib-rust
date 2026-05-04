import type { ReactNode } from "react";

import { MscProtectedShell } from "@/components/msc-shell";

export default function AdminLayout({ children }: { children: ReactNode }) {
  return <MscProtectedShell>{children}</MscProtectedShell>;
}
