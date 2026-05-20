import Button from "@jetbrains/ring-ui-built/components/button/button";
import type {AioToolbarAction} from "../types";

interface ToolbarProps {
  actions: AioToolbarAction[];
  busy?: string;
  onAction: (actionId: string) => void;
}

export function Toolbar({actions, busy, onAction}: ToolbarProps) {
  return (
    <div className="toolbar">
      {[...actions].sort((left, right) => left.order - right.order).map(action => (
        <Button
          key={action.id}
          title={action.tooltip}
          primary={action.primary}
          loader={busy === action.id}
          disabled={Boolean(busy)}
          onClick={() => onAction(action.id)}
        >
          {action.label}
        </Button>
      ))}
    </div>
  );
}
