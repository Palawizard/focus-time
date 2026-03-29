import { useQuery } from "@tanstack/react-query";

import { getRuntimeHealth } from "../lib/tauri";

const foundationItems = [
  "Workspace pnpm configure",
  "Shell desktop Tauri v2 initialise",
  "Workspace Rust connecte",
  "Lint, test et CI prets",
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
                Pomodoro, app tracking et statistiques dans un shell desktop
                propre.
              </h1>
              <p className="max-w-2xl text-sm leading-7 text-slate-300 sm:text-base">
                Cette base pose le monorepo, Tauri v2, React, le workspace Rust
                et la chaine qualite. Les epics produit peuvent maintenant etre
                construites sans repartir du scaffold brut.
              </p>
            </div>

            <div className="rounded-[24px] border border-cyan-400/20 bg-slate-950/50 p-5">
              <div className="mb-3 flex items-center justify-between">
                <h2 className="text-sm font-medium text-slate-200">
                  Runtime health
                </h2>
                <span className="rounded-full border border-emerald-400/20 bg-emerald-400/10 px-2.5 py-1 text-xs text-emerald-200">
                  bootstrap
                </span>
              </div>

              {runtimeHealth.isLoading ? (
                <p className="text-sm text-slate-400">
                  Initialisation du runtime natif...
                </p>
              ) : runtimeHealth.isError ? (
                <p className="text-sm text-rose-300">
                  Le shell Tauri n&apos;a pas encore repondu.
                </p>
              ) : (
                <dl className="space-y-3 text-sm">
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Produit</dt>
                    <dd className="text-slate-100">
                      {runtimeHealth.data.productName}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Shell</dt>
                    <dd className="text-slate-100">
                      {runtimeHealth.data.desktopShell}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Plateforme</dt>
                    <dd className="text-slate-100">
                      {runtimeHealth.data.platform}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-4">
                    <dt className="text-slate-400">Persistence</dt>
                    <dd className="text-slate-100">
                      {runtimeHealth.data.persistenceMode}
                    </dd>
                  </div>
                </dl>
              )}
            </div>
          </div>
        </header>

        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          {foundationItems.map((item) => (
            <article
              key={item}
              className="rounded-[24px] border border-white/10 bg-white/5 p-5"
            >
              <p className="text-xs uppercase tracking-[0.25em] text-slate-500">
                Epic 0
              </p>
              <h2 className="mt-3 text-lg font-medium text-white">{item}</h2>
            </article>
          ))}
        </section>

        <section className="grid gap-4 lg:grid-cols-[minmax(0,1.3fr)_minmax(18rem,1fr)]">
          <article className="rounded-[24px] border border-white/10 bg-slate-950/40 p-6">
            <h2 className="text-lg font-medium text-white">
              Prochaine etape logique
            </h2>
            <p className="mt-3 text-sm leading-7 text-slate-300">
              L&apos;epic 1 peut maintenant se concentrer sur le design system,
              la navigation et les ecrans vides sans melanger ces sujets avec
              la plomberie desktop, la toolchain Rust et la CI.
            </p>
          </article>

          <article className="rounded-[24px] border border-white/10 bg-white/5 p-6">
            <h2 className="text-lg font-medium text-white">
              Crates connectees
            </h2>
            <ul className="mt-3 space-y-2 text-sm text-slate-300">
              {(runtimeHealth.data?.workspaceCrates ?? []).map((crateName) => (
                <li key={crateName}>{crateName}</li>
              ))}
            </ul>
          </article>
        </section>
      </section>
    </main>
  );
}
