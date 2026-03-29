import { useQuery } from "@tanstack/react-query";

import { Button } from "../components/ui/Button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/Card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../components/ui/Dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../components/ui/Tabs";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../components/ui/Tooltip";
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
    <TooltipProvider delayDuration={100}>
      <DesktopShell>
      <section className="flex items-center justify-between border-b border-[var(--color-border)] px-5 py-4 sm:px-6">
        <div>
          <p className="ft-kicker text-[11px] font-semibold">Aujourd&apos;hui</p>
          <h2 className="ft-font-display mt-2 text-2xl font-semibold tracking-tight">
            Vue d&apos;ensemble
          </h2>
        </div>

        <div className="flex items-center gap-3">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="ft-panel-muted px-4 py-2 text-sm">
                Pret a demarrer
              </div>
            </TooltipTrigger>
            <TooltipContent>Tout est pret pour une nouvelle session.</TooltipContent>
          </Tooltip>

          <Dialog>
            <DialogTrigger asChild>
              <Button>Nouvelle session</Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle className="ft-font-display text-2xl font-semibold">
                  Nouvelle session
                </DialogTitle>
                <DialogDescription className="ft-text-muted text-sm">
                  Choisis une duree et lance ton prochain focus.
                </DialogDescription>
              </DialogHeader>

              <div className="mt-5 grid gap-3 sm:grid-cols-3">
                <Button variant="secondary">25 min</Button>
                <Button variant="secondary">45 min</Button>
                <Button variant="secondary">60 min</Button>
              </div>
            </DialogContent>
          </Dialog>
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

          <Card className="ft-panel p-5">
            <CardHeader>
              <CardDescription>Etat</CardDescription>
            </CardHeader>
            <CardContent>
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
            </CardContent>
          </Card>
        </header>

        <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          {focusStats.map((item) => (
            <Card key={item.label}>
              <CardDescription>{item.label}</CardDescription>
              <CardTitle className="mt-3">{item.value}</CardTitle>
            </Card>
          ))}

          <Card>
            <CardDescription>Top app</CardDescription>
            <CardTitle className="mt-3">Aucune</CardTitle>
          </Card>
        </section>

        <Tabs defaultValue="session" className="grid gap-4">
          <TabsList>
            <TabsTrigger value="session">Session</TabsTrigger>
            <TabsTrigger value="apps">Applications</TabsTrigger>
          </TabsList>

          <TabsContent
            value="session"
            className="grid gap-4 xl:grid-cols-[minmax(0,1.35fr)_minmax(20rem,1fr)]"
          >
            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Session</CardDescription>
                <CardTitle>Aucune session en cours.</CardTitle>
              </CardHeader>
            </Card>

            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Pause suivante</CardDescription>
                <CardTitle>25 min</CardTitle>
              </CardHeader>
            </Card>
          </TabsContent>

          <TabsContent value="apps">
            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Applications</CardDescription>
                <CardTitle>Le suivi apparaitra ici.</CardTitle>
              </CardHeader>
            </Card>
          </TabsContent>
        </Tabs>
      </section>
      </DesktopShell>
    </TooltipProvider>
  );
}
