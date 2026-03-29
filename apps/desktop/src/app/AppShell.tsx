import { useQuery } from "@tanstack/react-query";
import { AppWindow, ChartColumnBig, History, SlidersHorizontal, TimerReset, Trophy } from "lucide-react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";

import { Button } from "../components/ui/Button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../components/ui/Dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../components/ui/Tooltip";
import { GamificationScreen } from "../features/gamification/GamificationScreen";
import { HistoryScreen } from "../features/history/HistoryScreen";
import { FocusScreen } from "../features/pomodoro/FocusScreen";
import { SettingsScreen } from "../features/settings/SettingsScreen";
import { StatsScreen } from "../features/stats/StatsScreen";
import { TrackerScreen } from "../features/tracking/TrackerScreen";
import { getRuntimeHealth } from "../lib/tauri";
import { DesktopShell, type ShellRouteItem } from "./DesktopShell";

const routes: Array<
  ShellRouteItem & {
    description: string;
    element: React.ReactNode;
    eyebrow: string;
    title: string;
  }
> = [
  {
    path: "/",
    label: "Focus",
    title: "Vue d'ensemble",
    eyebrow: "Aujourd'hui",
    description: "Reste sur l'essentiel et demarre rapidement.",
    icon: TimerReset,
    element: <FocusScreen />,
  },
  {
    path: "/history",
    label: "History",
    title: "Historique",
    eyebrow: "Sessions",
    description: "Retrouve tes sessions et leur duree.",
    icon: History,
    element: <HistoryScreen />,
  },
  {
    path: "/stats",
    label: "Stats",
    title: "Statistiques",
    eyebrow: "Analyse",
    description: "Observe les tendances et le temps passe.",
    icon: ChartColumnBig,
    element: <StatsScreen />,
  },
  {
    path: "/tracker",
    label: "Tracker",
    title: "Tracker",
    eyebrow: "Applications",
    description: "Visualise les apps suivies pendant tes sessions.",
    icon: AppWindow,
    element: <TrackerScreen />,
  },
  {
    path: "/gamification",
    label: "Gamification",
    title: "Progression",
    eyebrow: "Regularite",
    description: "Suis ta serie et tes objectifs.",
    icon: Trophy,
    element: <GamificationScreen />,
  },
  {
    path: "/settings",
    label: "Settings",
    title: "Preferences",
    eyebrow: "Configuration",
    description: "Ajuste les presets et les comportements de l'app.",
    icon: SlidersHorizontal,
    element: <SettingsScreen />,
  },
];

function AppShellFrame() {
  const location = useLocation();
  const activeRoute =
    routes.find((route) =>
      route.path === "/"
        ? location.pathname === "/"
        : location.pathname.startsWith(route.path),
    ) ?? routes[0];

  const runtimeHealth = useQuery({
    queryKey: ["runtime-health"],
    queryFn: getRuntimeHealth,
  });

  return (
    <TooltipProvider delayDuration={100}>
      <DesktopShell
        actions={
          <>
            <Tooltip>
              <TooltipTrigger asChild>
                <div className="ft-panel-muted px-4 py-2 text-sm">
                  {runtimeHealth.isError ? "Indisponible" : "Local"}
                </div>
              </TooltipTrigger>
              <TooltipContent>Les donnees restent sur cette machine.</TooltipContent>
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
          </>
        }
        description={activeRoute.description}
        eyebrow={activeRoute.eyebrow}
        routes={routes}
        title={activeRoute.title}
      >
        <section className="flex flex-1 flex-col gap-5 overflow-auto p-5 sm:p-6">
          <Routes>
            {routes.map((route) => (
              <Route key={route.path} element={route.element} path={route.path} />
            ))}
            <Route element={<Navigate replace to="/" />} path="*" />
          </Routes>
        </section>
      </DesktopShell>
    </TooltipProvider>
  );
}

export function AppShell() {
  return <AppShellFrame />;
}
