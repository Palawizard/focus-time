import { useQuery } from "@tanstack/react-query";

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
    <main className="min-h-screen text-[var(--color-text)]">
      <section className="mx-auto flex min-h-screen w-full max-w-6xl flex-col gap-10 px-6 py-10 sm:px-10 lg:px-12">
        <header className="ft-panel-strong flex flex-col gap-4 p-6 md:p-8">
          <span className="ft-kicker text-xs font-semibold">
            Focus Time
          </span>
          <div className="grid gap-6 lg:grid-cols-[minmax(0,1.5fr)_minmax(22rem,1fr)]">
            <div className="space-y-4">
              <h1 className="ft-font-display max-w-3xl text-4xl font-semibold tracking-tight sm:text-5xl">
                Reste concentre. Une session a la fois.
              </h1>
              <p className="ft-text-muted max-w-2xl text-sm leading-7 sm:text-base">
                Suis ton temps, garde ton rythme et retrouve tes sessions en un
                coup d'oeil.
              </p>
            </div>

            <div className="ft-panel p-5">
              <div className="mb-3 flex items-center justify-between">
                <h2 className="text-sm font-medium">
                  Etat
                </h2>
              </div>

              {runtimeHealth.isLoading ? (
                <p className="ft-text-muted text-sm">
                  Preparation en cours...
                </p>
              ) : runtimeHealth.isError ? (
                <p className="text-sm text-[var(--color-danger)]">
                  Impossible de charger l'application.
                </p>
              ) : (
                <dl className="space-y-3 text-sm">
                  <div className="flex justify-between gap-4">
                    <dt className="ft-text-soft">Application</dt>
                    <dd>
                      {runtimeHealth.data.productName}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="ft-text-soft">Suivi</dt>
                    <dd className="ft-brand-badge rounded-full px-2.5 py-1 text-xs font-medium">
                      Pret
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="ft-text-soft">Plateforme</dt>
                    <dd>
                      {runtimeHealth.data.platform}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="ft-text-soft">Stockage</dt>
                    <dd>
                      Local
                    </dd>
                  </div>
                </dl>
              )}
            </div>
          </div>
        </header>

        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          {focusStats.map((item) => (
            <article
              key={item.label}
              className="ft-panel-muted p-5"
            >
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

        <section className="grid gap-4 lg:grid-cols-[minmax(0,1.3fr)_minmax(18rem,1fr)]">
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
    </main>
  );
}
