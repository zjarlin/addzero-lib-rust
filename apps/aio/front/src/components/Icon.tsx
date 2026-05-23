import {
  Archive,
  Boxes,
  ChevronRight,
  CircleDot,
  CloudUpload,
  Copy,
  Database,
  FolderSync,
  FolderTree,
  HardDrive,
  LayoutDashboard,
  Moon,
  Network,
  PackageSearch,
  PanelLeftClose,
  PanelLeftOpen,
  Play,
  RefreshCw,
  RotateCcw,
  Search,
  Settings,
  SlidersHorizontal,
  Sun,
  Upload,
  Wrench,
  X,
  Zap,
  type LucideIcon
} from "lucide-react";

const icons: Record<string, LucideIcon> = {
  Archive,
  Boxes,
  ChevronRight,
  CircleDot,
  CloudUpload,
  Copy,
  Database,
  FolderSync,
  FolderTree,
  HardDrive,
  LayoutDashboard,
  Moon,
  Network,
  PackageSearch,
  PanelLeftClose,
  PanelLeftOpen,
  Play,
  RefreshCw,
  RotateCcw,
  Search,
  Settings,
  SlidersHorizontal,
  Sun,
  Upload,
  Wrench,
  X,
  Zap
};

interface IconProps {
  name: string;
  size?: number;
  className?: string;
  "aria-hidden"?: boolean;
}

export function Icon({name, size = 16, className, "aria-hidden": ariaHidden = true}: IconProps) {
  const Component = icons[name] ?? CircleDot;
  return <Component aria-hidden={ariaHidden} className={className} size={size} strokeWidth={1.8} />;
}
