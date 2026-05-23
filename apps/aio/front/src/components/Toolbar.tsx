import {Icon} from "./Icon";
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
        <button
          key={action.id}
          className={action.primary ? "toolbarButton primary" : "toolbarButton"}
          title={action.tooltip}
          disabled={Boolean(busy)}
          onClick={() => onAction(action.id)}
          type="button"
        >
          {busy === action.id ? <span className="buttonSpinner" /> : <Icon name={action.icon} />}
          {action.label}
        </button>
      ))}
    </div>
  );
}
