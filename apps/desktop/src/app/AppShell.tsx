import { useQuery } from "@tanstack/react-query";

import { DesktopShell } from "./DesktopShell";
import { getRuntimeHealth } from "../lib/tauri";

const focusStats = [
  { label: "Aujourd'hui", value: "0 min" },
  { label: "Sessions", value: "0" },
  { label: "Serie", value: "0 jour" },
] as const;

export function AppShell() {
  const runtimeHealth = useQuery({
    queryKey: ["runtime-health"],
    queryFn: getRuntimeHealth,
  });

  return (
    <DesktopShell>
      <section className="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4 sm:px-6">
        <div>
          <p className="ft-kicker text-[11px] font-semibold">Aujourd&apos;hui</p>
          <h2 className="ft-font-display mt-2 text-2xl font-semibold tracking-tight">
            Vue d&apos;ensemble
          </h2>
        </div>

        <div className="ft-panel-muted px-4 py-2 text-sm">
          Pret a demarrer
        </div>
      </section>

      <section className="flex flex-1 flex-col gap-5 overflow-auto p-5 sm:p-6">
        <header className="grid gap-6 xl:grid-cols-[minmax(0,1.45fr)_minmax(20rem,1fr)]">
          <div className="space-y-4">
            <h1 className="ft-font-display max-w-3xl text-4xl font-semibold tracking-tight sm:text-5xl">
              Reste concentre. Une session a la fois.
            </h1>
            <p className="ft-text-muted max-w-2xl text-sm leading-7 sm:text-base">
              Suis ton temps, garde ton rythme et retrouve tes sessions en un
              coup d&apos;oeil.
            </p>
          </div>

          <div className="ft-panel p-5">
            <div className="mb-3 flex items-center justify-between">
              <h2 className="text-sm font-medium">Etat</h2>
            </div>

            {runtimeHealth.isLoading ? (
              <p className="ft-text-muted text-sm">Preparation en cours...</p>
            ) : runtimeHealth.isError ? (
              <p className="text-sm text-[var(--color-danger)]">
                Impossible de charger l&apos;application.
              </p>
            ) : (
              <dl className="space-y-3 text-sm">
                <div className="flex justify-between gap-4">
                  <dt className="ft-text-soft">Application</dt>
                  <dd>{runtimeHealth.data.productName}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="ft-text-soft">Suivi</dt>
                  <dd className="ft-brand-badge rounded-full px-2.5 py-1 text-xs font-medium">
                    Pret
                  </dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="ft-text-soft">Plateforme</dt>
                  <dd>{runtimeHealth.data.platform}</dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="ft-text-soft">Stockage</dt>
                  <dd>Local</dd>
                </div>
              </dl>
            )}
          </div>
        </header>

        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          {focusStats.map((item) => (
            <article key={item.label} className="ft-panel-muted p-5">
              <p className="ft-text-muted text-sm">{item.label}</p>
              <h2 className="ft-font-display mt-3 text-2xl font-medium">
                {item.value}
              </h2>
            </article>
          ))}

          <article className="ft-panel-muted p-5">
            <p className="ft-text-muted text-sm">Top app</p>
            <h2 className="ft-font-display mt-3 text-2xl font-medium">
              Aucune
            </h2>
          </article>
        </section>

        <section className="grid gap-4 xl:grid-cols-[minmax(0,1.35fr)_minmax(20rem,1fr)]">
          <article className="ft-panel p-6">
            <h2 className="text-lg font-medium">Session</h2>
            <p className="ft-text-muted mt-3 text-sm leading-7">
              Aucune session en cours.
            </p>
          </article>

          <article className="ft-panel p-6">
            <h2 className="text-lg font-medium">Applications</h2>
            <ul className="ft-text-muted mt-3 space-y-2 text-sm">
              <li>Le suivi apparaitra ici pendant les sessions.</li>
            </ul>
          </article>
        </section>
      </section>
    </DesktopShell>
  );
}
