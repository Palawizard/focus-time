import { useQuery } from "@tanstack/react-query";
import { AppWindow, ChartColumnBig, History, SlidersHorizontal, TimerReset, Trophy } from "lucide-react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";

import { Button } from "../components/ui/Button";
import { ThemeSwitch } from "../components/ThemeSwitch";
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
    title: "Overview",
    eyebrow: "Today",
    description: "Stay on track and start a session in seconds.",
    icon: TimerReset,
    element: <FocusScreen />,
  },
  {
    path: "/history",
    label: "History",
    title: "History",
    eyebrow: "Sessions",
    description: "Review past sessions and their duration.",
    icon: History,
    element: <HistoryScreen />,
  },
  {
    path: "/stats",
    label: "Stats",
    title: "Stats",
    eyebrow: "Insights",
    description: "See your trends and time distribution at a glance.",
    icon: ChartColumnBig,
    element: <StatsScreen />,
  },
  {
    path: "/tracker",
    label: "Tracker",
    title: "Tracker",
    eyebrow: "Apps",
    description: "See which apps were active during your sessions.",
    icon: AppWindow,
    element: <TrackerScreen />,
  },
  {
    path: "/gamification",
    label: "Gamification",
    title: "Progress",
    eyebrow: "Consistency",
    description: "Track your streak and weekly goals.",
    icon: Trophy,
    element: <GamificationScreen />,
  },
  {
    path: "/settings",
    label: "Settings",
    title: "Settings",
    eyebrow: "Preferences",
    description: "Adjust presets and app behavior.",
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
            <ThemeSwitch />

            <Tooltip>
              <TooltipTrigger asChild>
                <div className="ft-panel-muted px-4 py-2 text-sm">
                  {runtimeHealth.isError ? "Unavailable" : "Local"}
                </div>
              </TooltipTrigger>
              <TooltipContent>Your data stays on this device.</TooltipContent>
            </Tooltip>

            <Dialog>
              <DialogTrigger asChild>
                <Button>New session</Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle className="ft-font-display text-2xl font-semibold">
                    New session
                  </DialogTitle>
                  <DialogDescription className="ft-text-muted text-sm">
                    Pick a duration and start your next focus block.
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
