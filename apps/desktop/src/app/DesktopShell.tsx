interface DesktopShellProps {
  children: React.ReactNode;
}

const shellItems = [
  "Focus",
  "History",
  "Stats",
  "Tracker",
  "Gamification",
  "Settings",
] as const;

export function DesktopShell({ children }: DesktopShellProps) {
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
                Garde un rythme simple et clair.
              </p>
            </div>
          </div>

          <nav aria-label="Primary" className="space-y-2">
            {shellItems.map((item, index) => (
              <div
                key={item}
                className={[
                  "flex items-center justify-between rounded-[1.15rem] px-4 py-3 text-sm transition-colors",
                  index === 0
                    ? "bg-[var(--color-brand-soft)] text-[var(--color-brand)]"
                    : "text-[var(--color-text-muted)]",
                ].join(" ")}
              >
                <span>{item}</span>
                {index === 0 ? (
                  <span className="rounded-full border border-[var(--color-border-strong)] px-2 py-0.5 text-[10px] uppercase tracking-[0.2em]">
                    Live
                  </span>
                ) : null}
              </div>
            ))}
          </nav>
        </div>

        <div className="ft-panel mt-8 p-4">
          <p className="ft-text-soft text-xs uppercase tracking-[0.22em]">
            Aujourd&apos;hui
          </p>
          <p className="ft-font-display mt-3 text-3xl font-semibold">00:00</p>
        </div>
      </aside>

      <div className="ft-panel-strong flex min-h-[calc(100vh-2rem)] flex-col overflow-hidden">
        {children}
      </div>
    </div>
  );
}
