import type {ReactNode} from "react";

interface PanelProps {
  title: string;
  metric?: string | number;
  children: ReactNode;
}

export function Panel({title, metric, children}: PanelProps) {
  return (
    <section className="panel">
      <div className="panelHeader">
        <h3>{title}</h3>
        {metric !== undefined ? <span className="metric">{metric}</span> : null}
      </div>
      {children}
    </section>
  );
}

export function JsonBlock({value}: {value: unknown}) {
  return <pre className="jsonBlock">{JSON.stringify(value, null, 2)}</pre>;
}
