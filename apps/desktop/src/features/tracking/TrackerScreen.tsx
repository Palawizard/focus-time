import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Button } from "../../components/ui/Button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../components/ui/Card";
import {
  createTrackingExclusionRule,
  deleteTrackingExclusionRule,
  getTrackingStatus,
  getUserPreferences,
  listTrackedApps,
  listTrackedWindowEvents,
  listTrackingExclusionRules,
  saveUserPreferences,
  upsertTrackedApp,
} from "../../lib/storage";
import type {
  TrackingCategory,
  TrackingExclusionKind,
  TrackedApp,
  TrackedWindowEvent,
  UserPreference,
} from "../../types/storage";

const exclusionKindOptions: Array<{
  value: TrackingExclusionKind;
  label: string;
}> = [
  { value: "window_title", label: "Window title" },
  { value: "executable", label: "Executable" },
  { value: "category", label: "Category" },
];

const categoryLabels: Record<TrackingCategory, string> = {
  development: "Development",
  browser: "Browser",
  communication: "Communication",
  writing: "Writing",
  design: "Design",
  meeting: "Meeting",
  research: "Research",
  utilities: "Utilities",
  unknown: "Unknown",
};

export function TrackerScreen() {
  const queryClient = useQueryClient();
  const [ruleKind, setRuleKind] = useState<TrackingExclusionKind>("window_title");
  const [rulePattern, setRulePattern] = useState("");

  const trackingStatus = useQuery({
    queryKey: ["tracking-status"],
    queryFn: getTrackingStatus,
    refetchInterval: 3_000,
  });
  const preferences = useQuery({
    queryKey: ["user-preferences"],
    queryFn: getUserPreferences,
  });
  const trackedApps = useQuery({
    queryKey: ["tracked-apps"],
    queryFn: listTrackedApps,
  });
  const exclusionRules = useQuery({
    queryKey: ["tracking-exclusion-rules"],
    queryFn: listTrackingExclusionRules,
  });
  const trackedEvents = useQuery({
    queryKey: ["tracked-window-events"],
    queryFn: () => listTrackedWindowEvents(12),
    refetchInterval: 5_000,
  });

  const invalidateTracking = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["tracking-status"] }),
      queryClient.invalidateQueries({ queryKey: ["user-preferences"] }),
      queryClient.invalidateQueries({ queryKey: ["tracked-apps"] }),
      queryClient.invalidateQueries({ queryKey: ["tracking-exclusion-rules"] }),
      queryClient.invalidateQueries({ queryKey: ["tracked-window-events"] }),
    ]);
  };

  const savePreferencesMutation = useMutation({
    mutationFn: (request: Omit<UserPreference, "updatedAt">) =>
      saveUserPreferences(request),
    onSuccess: invalidateTracking,
  });
  const toggleAppMutation = useMutation({
    mutationFn: (app: TrackedApp) =>
      upsertTrackedApp({
        name: app.name,
        executable: app.executable,
        category: app.category,
        colorHex: app.colorHex,
        isExcluded: !app.isExcluded,
      }),
    onSuccess: invalidateTracking,
  });
  const createRuleMutation = useMutation({
    mutationFn: (request: { kind: TrackingExclusionKind; pattern: string }) =>
      createTrackingExclusionRule(request),
    onSuccess: async () => {
      setRulePattern("");
      await invalidateTracking();
    },
  });
  const deleteRuleMutation = useMutation({
    mutationFn: deleteTrackingExclusionRule,
    onSuccess: invalidateTracking,
  });

  const currentPreferences = preferences.data;
  const runtime = trackingStatus.data;
  const topApp = pickTopApp(trackedEvents.data ?? []);
  const excludedAppsCount =
    trackedApps.data?.filter((app) => app.isExcluded).length ?? 0;
  const onboardingBlocked =
    !runtime?.permissionGranted || !runtime?.onboardingCompleted;
  const canCreateRule = rulePattern.trim().length > 0;

  const updatePreferences = (mutate: (draft: Omit<UserPreference, "updatedAt">) => void) => {
    if (!currentPreferences) {
      return;
    }

    const next: Omit<UserPreference, "updatedAt"> = {
      focusMinutes: currentPreferences.focusMinutes,
      shortBreakMinutes: currentPreferences.shortBreakMinutes,
      longBreakMinutes: currentPreferences.longBreakMinutes,
      sessionsUntilLongBreak: currentPreferences.sessionsUntilLongBreak,
      autoStartBreaks: currentPreferences.autoStartBreaks,
      autoStartFocus: currentPreferences.autoStartFocus,
      trackingEnabled: currentPreferences.trackingEnabled,
      trackingPermissionGranted: currentPreferences.trackingPermissionGranted,
      trackingOnboardingCompleted: currentPreferences.trackingOnboardingCompleted,
      notificationsEnabled: currentPreferences.notificationsEnabled,
      theme: currentPreferences.theme,
    };

    mutate(next);
    savePreferencesMutation.mutate(next);
  };

  const handleEnableTracking = () =>
    updatePreferences((draft) => {
      draft.trackingEnabled = true;
      draft.trackingPermissionGranted = true;
      draft.trackingOnboardingCompleted = true;
    });

  const handleToggleTracking = () =>
    updatePreferences((draft) => {
      draft.trackingEnabled = !draft.trackingEnabled;
    });

  return (
    <div className="grid gap-5 xl:grid-cols-[minmax(0,1.2fr)_minmax(22rem,0.8fr)]">
      <div className="grid gap-5">
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Tracker</CardDescription>
            <CardTitle>Capture active apps during real focus time.</CardTitle>
          </CardHeader>

          <CardContent className="grid gap-4">
            <div className="grid gap-3 md:grid-cols-3">
              <MetricCard
                label="Mode"
                value={runtime ? formatMode(runtime.status.mode) : "Loading"}
              />
              <MetricCard
                label="Top app"
                value={topApp ?? "No activity yet"}
              />
              <MetricCard
                label="Exclusions"
                value={`${excludedAppsCount} app${excludedAppsCount === 1 ? "" : "s"}`}
              />
            </div>

            <div className="ft-panel-strong p-5">
              <p className="ft-kicker text-[11px] font-semibold">
                {runtime?.status.capability === "supported"
                  ? "Ready"
                  : runtime?.status.capability === "limited"
                    ? "Limited"
                    : "Unavailable"}
              </p>
              <p className="mt-3 text-lg font-medium">
                {runtime?.status.message ?? "Preparing tracker diagnostics..."}
              </p>
              {runtime?.status.dependencyHint ? (
                <p className="ft-text-muted mt-2 text-sm">
                  {runtime.status.dependencyHint}
                </p>
              ) : null}
              {runtime?.lastError ? (
                <p className="mt-2 text-sm text-[var(--color-danger)]">
                  {runtime.lastError}
                </p>
              ) : null}

              <div className="mt-5 flex flex-wrap gap-3">
                {onboardingBlocked ? (
                  <Button
                    disabled={!currentPreferences}
                    onClick={handleEnableTracking}
                  >
                    Enable tracking
                  </Button>
                ) : (
                  <Button
                    disabled={!currentPreferences}
                    onClick={handleToggleTracking}
                    variant={runtime?.trackingEnabled ? "secondary" : "default"}
                  >
                    {runtime?.trackingEnabled ? "Pause tracking" : "Resume tracking"}
                  </Button>
                )}
              </div>
            </div>

            {onboardingBlocked ? (
              <Card className="p-5">
                <CardDescription>Permission</CardDescription>
                <CardTitle className="mt-2 text-xl">
                  Tracking stays off until you explicitly enable it.
                </CardTitle>
                <p className="ft-text-muted mt-3 text-sm">
                  Focus Time keeps everything local. Enabling tracking lets the app
                  watch the active window only while a focus session is running.
                </p>
              </Card>
            ) : null}

            <div className="grid gap-3 md:grid-cols-2">
              <Card className="p-5">
                <CardDescription>Current window</CardDescription>
                <CardTitle className="mt-2 text-xl">
                  {runtime?.activeWindow?.appName ?? "No active tracked app"}
                </CardTitle>
                <p className="ft-text-muted mt-3 text-sm">
                  {runtime?.activeWindow?.windowTitle ??
                    "Start a focus session to see live app activity."}
                </p>
                {runtime?.activeWindow ? (
                  <p className="mt-3 text-sm">
                    {categoryLabels[runtime.activeWindow.category]} ·{" "}
                    {runtime.activeWindow.executable}
                  </p>
                ) : null}
              </Card>

              <Card className="p-5">
                <CardDescription>Live status</CardDescription>
                <CardTitle className="mt-2 text-xl">
                  {runtime?.isTrackingLive ? "Tracking in progress" : "Idle"}
                </CardTitle>
                <p className="ft-text-muted mt-3 text-sm">
                  {runtime?.activeSessionId
                    ? `Session #${runtime.activeSessionId} is eligible for tracking.`
                    : "Tracking activates only during running focus phases."}
                </p>
              </Card>
            </div>
          </CardContent>
        </Card>

        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Known apps</CardDescription>
            <CardTitle>Manage per-app exclusions.</CardTitle>
          </CardHeader>

          <CardContent className="space-y-3">
            {trackedApps.data?.length ? (
              trackedApps.data.map((app) => (
                <div
                  className="ft-panel-muted flex items-center justify-between gap-4 px-4 py-3"
                  key={app.id}
                >
                  <div>
                    <p className="text-sm font-medium">{app.name}</p>
                    <p className="ft-text-muted mt-1 text-xs">
                      {categoryLabels[app.category]} · {app.executable}
                    </p>
                  </div>
                  <Button
                    disabled={toggleAppMutation.isPending}
                    onClick={() => toggleAppMutation.mutate(app)}
                    size="sm"
                    variant={app.isExcluded ? "default" : "secondary"}
                  >
                    {app.isExcluded ? "Excluded" : "Track"}
                  </Button>
                </div>
              ))
            ) : (
              <p className="ft-text-muted text-sm">
                Known apps will appear after your first tracked sessions.
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-5">
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Rules</CardDescription>
            <CardTitle>Exclude titles, executables or categories.</CardTitle>
          </CardHeader>

          <CardContent className="space-y-4">
            <div className="grid gap-3">
              <select
                className="ft-panel-muted h-11 rounded-[1rem] border border-[var(--color-border)] bg-transparent px-4 text-sm"
                onChange={(event) =>
                  setRuleKind(event.target.value as TrackingExclusionKind)
                }
                value={ruleKind}
              >
                {exclusionKindOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>

              <input
                className="ft-panel-muted h-11 rounded-[1rem] border border-[var(--color-border)] bg-transparent px-4 text-sm outline-none"
                onChange={(event) => setRulePattern(event.target.value)}
                placeholder={
                  ruleKind === "category"
                    ? "browser"
                    : ruleKind === "executable"
                      ? "slack"
                      : "Private project"
                }
                value={rulePattern}
              />

              <Button
                disabled={!canCreateRule || createRuleMutation.isPending}
                onClick={() =>
                  createRuleMutation.mutate({
                    kind: ruleKind,
                    pattern: rulePattern.trim(),
                  })
                }
              >
                Add rule
              </Button>
            </div>

            <div className="space-y-3">
              {exclusionRules.data?.length ? (
                exclusionRules.data.map((rule) => (
                  <div
                    className="ft-panel-muted flex items-center justify-between gap-4 px-4 py-3"
                    key={rule.id}
                  >
                    <div>
                      <p className="text-sm font-medium">{rule.pattern}</p>
                      <p className="ft-text-muted mt-1 text-xs">
                        {rule.kind.replace("_", " ")}
                      </p>
                    </div>
                    <Button
                      disabled={deleteRuleMutation.isPending}
                      onClick={() => deleteRuleMutation.mutate(rule.id)}
                      size="sm"
                      variant="ghost"
                    >
                      Remove
                    </Button>
                  </div>
                ))
              ) : (
                <p className="ft-text-muted text-sm">
                  No rule yet. Add one to keep personal apps or window titles out
                  of your timeline.
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Recent activity</CardDescription>
            <CardTitle>Review the last tracked windows.</CardTitle>
          </CardHeader>

          <CardContent className="space-y-3">
            {trackedEvents.data?.length ? (
              trackedEvents.data.map((event) => (
                <RecentEventRow event={event} key={event.id} />
              ))
            ) : (
              <p className="ft-text-muted text-sm">
                Recent tracked windows will appear here after a focus session.
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="ft-panel-muted px-4 py-3">
      <p className="ft-text-muted text-sm">{label}</p>
      <p className="mt-2 text-sm font-medium">{value}</p>
    </div>
  );
}

function RecentEventRow({ event }: { event: TrackedWindowEvent }) {
  return (
    <div className="ft-panel-muted px-4 py-3">
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="text-sm font-medium">{event.appName ?? "Unknown app"}</p>
          <p className="ft-text-muted mt-1 text-xs">
            {event.windowTitle ?? "No title"} · {formatTimeRange(event.startedAt, event.endedAt)}
          </p>
        </div>
        <span className="ft-brand-badge rounded-full px-3 py-1 text-xs font-medium">
          {event.category ? categoryLabels[event.category] : "Unknown"}
        </span>
      </div>
    </div>
  );
}

function pickTopApp(events: TrackedWindowEvent[]): string | null {
  const durations = new Map<string, number>();

  events.forEach((event) => {
    if (!event.appName) {
      return;
    }

    const startedAt = Date.parse(event.startedAt);
    const endedAt = event.endedAt ? Date.parse(event.endedAt) : startedAt;
    const duration = Math.max(0, endedAt - startedAt);

    durations.set(event.appName, (durations.get(event.appName) ?? 0) + duration);
  });

  let winner: string | null = null;
  let maxDuration = -1;

  durations.forEach((duration, appName) => {
    if (duration > maxDuration) {
      maxDuration = duration;
      winner = appName;
    }
  });

  return winner;
}

function formatMode(mode: string) {
  switch (mode) {
    case "windows_native":
      return "Windows";
    case "linux_hyprland":
      return "Hyprland";
    case "linux_sway":
      return "Sway";
    case "linux_x11":
      return "Linux X11";
    case "linux_wayland":
      return "Linux Wayland";
    default:
      return "Unsupported";
  }
}

function formatTimeRange(startedAt: string, endedAt: string | null) {
  const formatter = new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });

  const start = formatter.format(new Date(startedAt));
  const end = formatter.format(new Date(endedAt ?? startedAt));

  return `${start} - ${end}`;
}
