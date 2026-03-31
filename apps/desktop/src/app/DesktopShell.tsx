import type { LucideIcon } from "lucide-react";
import { NavLink } from "react-router-dom";

import { cn } from "../lib/cn";

export interface ShellRouteItem {
  icon: LucideIcon;
  label: string;
  path: string;
}

interface DesktopShellProps {
  actions?: React.ReactNode;
  description: string;
  eyebrow: string;
  children: React.ReactNode;
  routes: ShellRouteItem[];
  title: string;
}

export function DesktopShell({
  actions,
  children,
  description,
  eyebrow,
  routes,
  title,
}: DesktopShellProps) {
  return (
    <div className="mx-auto grid min-h-screen w-full max-w-[1480px] gap-5 px-4 py-4 lg:grid-cols-[280px_minmax(0,1fr)] lg:px-6 lg:py-6">
      <aside className="ft-panel-strong flex flex-col justify-between p-5 lg:p-6">
        <div className="space-y-8">
          <div className="space-y-3">
            <p className="ft-kicker text-xs font-semibold">Focus Time</p>
            <div>
              <h1 className="ft-font-display text-2xl font-semibold tracking-tight">
                Focus
              </h1>
              <p className="ft-text-muted mt-2 text-sm leading-6">
                Keep a steady rhythm with less friction.
              </p>
            </div>
          </div>

          <nav aria-label="Primary" className="space-y-2">
            {routes.map((item) => (
              <NavLink
                key={item.path}
                end={item.path === "/"}
                to={item.path}
                className={({ isActive }) =>
                  cn(
                    "flex items-center justify-between rounded-[1.15rem] px-4 py-3 text-sm transition-colors",
                    isActive
                      ? "bg-[var(--color-brand-soft)] text-[var(--color-brand)]"
                      : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-text)]",
                  )
                }
              >
                <span className="flex items-center gap-3">
                  <item.icon className="h-4 w-4" />
                  {item.label}
                </span>
              </NavLink>
            ))}
          </nav>
        </div>

        <div className="ft-panel mt-8 p-4">
          <p className="ft-text-soft text-xs uppercase tracking-[0.22em]">
            Today
          </p>
          <p className="ft-font-display mt-3 text-3xl font-semibold">00:00</p>
        </div>
      </aside>

      <div className="ft-panel-strong flex min-h-[calc(100vh-2rem)] flex-col overflow-hidden">
        <section className="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4 sm:px-6">
          <div>
            <p className="ft-kicker text-[11px] font-semibold">{eyebrow}</p>
            <h2 className="ft-font-display mt-2 text-2xl font-semibold tracking-tight">
              {title}
            </h2>
            <p className="ft-text-muted mt-2 max-w-2xl text-sm leading-6">
              {description}
            </p>
          </div>

          <div className="flex items-center gap-3">{actions}</div>
        </section>
        {children}
      </div>
    </div>
  );
}
