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
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_#1f2937,_#0f172a_55%,_#020617)] text-slate-100">
      <section className="mx-auto flex min-h-screen w-full max-w-6xl flex-col gap-10 px-6 py-10 sm:px-10 lg:px-12">
        <header className="flex flex-col gap-4 rounded-[28px] border border-white/10 bg-white/5 p-6 shadow-2xl shadow-slate-950/40 backdrop-blur md:p-8">
          <span className="text-xs font-semibold uppercase tracking-[0.3em] text-cyan-300/80">
            Focus Time
          </span>
          <div className="grid gap-6 lg:grid-cols-[minmax(0,1.5fr)_minmax(22rem,1fr)]">
            <div className="space-y-4">
              <h1 className="max-w-3xl text-4xl font-semibold tracking-tight text-white sm:text-5xl">
                Reste concentre. Une session a la fois.
              </h1>
              <p className="max-w-2xl text-sm leading-7 text-slate-300 sm:text-base">
                Suis ton temps, garde ton rythme et retrouve tes sessions en un
                coup d'oeil.
              </p>
            </div>

            <div className="rounded-[24px] border border-cyan-400/20 bg-slate-950/50 p-5">
              <div className="mb-3 flex items-center justify-between">
                <h2 className="text-sm font-medium text-slate-200">
                  Etat
                </h2>
              </div>

              {runtimeHealth.isLoading ? (
                <p className="text-sm text-slate-400">
                  Preparation en cours...
                </p>
              ) : runtimeHealth.isError ? (
                <p className="text-sm text-rose-300">
                  Impossible de charger l'application.
                </p>
              ) : (
                <dl className="space-y-3 text-sm">
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Application</dt>
                    <dd className="text-slate-100">
                      {runtimeHealth.data.productName}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Suivi</dt>
                    <dd className="text-slate-100">
                      Pret
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Plateforme</dt>
                    <dd className="text-slate-100">
                      {runtimeHealth.data.platform}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Stockage</dt>
                    <dd className="text-slate-100">
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
              className="rounded-[24px] border border-white/10 bg-white/5 p-5"
            >
              <p className="text-sm text-slate-400">{item.label}</p>
              <h2 className="mt-3 text-2xl font-medium text-white">
                {item.value}
              </h2>
            </article>
          ))}

          <article className="rounded-[24px] border border-white/10 bg-white/5 p-5">
            <p className="text-sm text-slate-400">Top app</p>
            <h2 className="mt-3 text-2xl font-medium text-white">Aucune</h2>
          </article>
        </section>

        <section className="grid gap-4 lg:grid-cols-[minmax(0,1.3fr)_minmax(18rem,1fr)]">
          <article className="rounded-[24px] border border-white/10 bg-slate-950/40 p-6">
            <h2 className="text-lg font-medium text-white">Session</h2>
            <p className="mt-3 text-sm leading-7 text-slate-300">
              Aucune session en cours.
            </p>
          </article>

          <article className="rounded-[24px] border border-white/10 bg-white/5 p-6">
            <h2 className="text-lg font-medium text-white">Applications</h2>
            <ul className="mt-3 space-y-2 text-sm text-slate-300">
              <li>Le suivi apparaitra ici pendant les sessions.</li>
            </ul>
          </article>
        </section>
      </section>
    </main>
  );
}
