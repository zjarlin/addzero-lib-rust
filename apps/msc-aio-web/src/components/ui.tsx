"use client";

import clsx from "clsx";
import type { ComponentProps, ReactNode } from "react";

export function PageHeader({
  title,
  subtitle,
  actions,
}: {
  title: string;
  subtitle: string;
  actions?: ReactNode;
}) {
  return (
    <div className="mb-6 flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
      <div>
        <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">MSC_AIO</p>
        <h1 className="mt-2 text-3xl font-semibold text-white">{title}</h1>
        <p className="mt-2 max-w-3xl text-sm text-zinc-400">{subtitle}</p>
      </div>
      {actions ? <div className="flex flex-wrap gap-2">{actions}</div> : null}
    </div>
  );
}

export function Card({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  return (
    <section className={clsx("rounded-xl border border-white/10 bg-white/5 p-4 shadow-panel", className)}>
      {children}
    </section>
  );
}

export function SectionTitle({
  title,
  detail,
  actions,
}: {
  title: string;
  detail?: string;
  actions?: ReactNode;
}) {
  return (
    <header className="mb-4 flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
      <div>
        <h2 className="text-lg font-semibold text-white">{title}</h2>
        {detail ? <p className="mt-1 text-sm text-zinc-400">{detail}</p> : null}
      </div>
      {actions ? <div className="flex flex-wrap gap-2">{actions}</div> : null}
    </header>
  );
}

export function Button({
  className,
  tone = "default",
  ...props
}: ComponentProps<"button"> & { tone?: "default" | "accent" | "danger" }) {
  return (
    <button
      className={clsx(
        "rounded-md border px-3 py-2 text-sm transition disabled:cursor-not-allowed disabled:opacity-50",
        tone === "default" && "border-white/10 bg-white/5 text-zinc-100 hover:bg-white/10",
        tone === "accent" && "border-emerald-400/50 bg-emerald-500/10 text-emerald-100",
        tone === "danger" && "border-red-400/40 bg-red-500/10 text-red-100",
        className,
      )}
      {...props}
    />
  );
}

export function Input(props: ComponentProps<"input">) {
  return (
    <input
      {...props}
      className={clsx(
        "w-full rounded-md border border-white/10 bg-zinc-950/80 px-3 py-2 text-sm text-white outline-none placeholder:text-zinc-500 focus:border-emerald-400/50",
        props.className,
      )}
    />
  );
}

export function Textarea(props: ComponentProps<"textarea">) {
  return (
    <textarea
      {...props}
      className={clsx(
        "w-full rounded-md border border-white/10 bg-zinc-950/80 px-3 py-2 text-sm text-white outline-none placeholder:text-zinc-500 focus:border-emerald-400/50",
        props.className,
      )}
    />
  );
}

export function Select(props: ComponentProps<"select">) {
  return (
    <select
      {...props}
      className={clsx(
        "w-full rounded-md border border-white/10 bg-zinc-950/80 px-3 py-2 text-sm text-white outline-none focus:border-emerald-400/50",
        props.className,
      )}
    />
  );
}

export function Field({
  label,
  detail,
  children,
}: {
  label: string;
  detail?: string;
  children: ReactNode;
}) {
  return (
    <label className="flex flex-col gap-2">
      <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
        {label}
      </span>
      {children}
      {detail ? <span className="text-xs text-zinc-500">{detail}</span> : null}
    </label>
  );
}

export function Callout({
  children,
  tone = "info",
}: {
  children: ReactNode;
  tone?: "info" | "danger";
}) {
  return (
    <div
      className={clsx(
        "mb-4 rounded-lg border px-3 py-2 text-sm",
        tone === "info" && "border-sky-400/20 bg-sky-400/10 text-sky-100",
        tone === "danger" && "border-red-400/30 bg-red-500/10 text-red-100",
      )}
    >
      {children}
    </div>
  );
}

export function StatGrid({ items }: { items: Array<{ label: string; value: string; detail: string }> }) {
  return (
    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
      {items.map((item) => (
        <Card key={item.label} className="space-y-2">
          <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">{item.label}</div>
          <div className="text-2xl font-semibold text-white">{item.value}</div>
          <div className="text-sm text-zinc-400">{item.detail}</div>
        </Card>
      ))}
    </div>
  );
}

export function DataTable({
  columns,
  rows,
}: {
  columns: string[];
  rows: ReactNode[][];
}) {
  return (
    <div className="overflow-x-auto">
      <table className="min-w-full border-separate border-spacing-y-2 text-sm">
        <thead>
          <tr>
            {columns.map((column) => (
              <th
                key={column}
                className="px-3 py-2 text-left text-xs uppercase tracking-[0.18em] text-zinc-500"
              >
                {column}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, rowIndex) => (
            <tr key={rowIndex} className="rounded-lg bg-black/20">
              {row.map((cell, cellIndex) => (
                <td key={cellIndex} className="border-y border-white/5 px-3 py-3 text-zinc-200 first:rounded-l-lg first:border-l last:rounded-r-lg last:border-r">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
