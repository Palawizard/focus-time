import { useEffect, useMemo, useState } from "react";
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
  createLocalBackup,
  getUserPreferences,
  listLocalBackups,
  restoreLocalBackup,
  saveUserPreferences,
} from "../../lib/storage";
import { getRuntimeHealth } from "../../lib/tauri";
import { usePomodoroPreferencesStore } from "../../stores/pomodoro-preferences-store";
import { useThemeStore, type ThemeMode } from "../../stores/theme-store";
import type { BackupArchiveSummary, UserPreference } from "../../types/storage";

const inputClassName =
  "ft-panel-muted h-11 rounded-[1rem] border border-[var(--color-border)] bg-transparent px-4 text-sm outline-none";

type SettingsDraft = Omit<UserPreference, "updatedAt">;

export function SettingsScreen() {
  const queryClient = useQueryClient();
  const setThemeMode = useThemeStore((state) => state.setMode);
  const setSoundEnabled = usePomodoroPreferencesStore(
    (state) => state.setSoundEnabled,
  );
  const [draft, setDraft] = useState<SettingsDraft | null>(null);
  const [saveFeedback, setSaveFeedback] = useState<string | null>(null);
  const [backupFeedback, setBackupFeedback] = useState<string | null>(null);

  const preferencesQuery = useQuery({
    queryKey: ["user-preferences"],
    queryFn: getUserPreferences,
  });
  const runtimeQuery = useQuery({
    queryKey: ["runtime-health"],
    queryFn: getRuntimeHealth,
  });
  const backupsQuery = useQuery({
    queryKey: ["local-backups"],
    queryFn: listLocalBackups,
  });

  useEffect(() => {
    if (!preferencesQuery.data || draft !== null) {
      return;
    }

    setDraft(stripUpdatedAt(preferencesQuery.data));
  }, [draft, preferencesQuery.data]);

  const saveMutation = useMutation({
    mutationFn: (nextDraft: SettingsDraft) => saveUserPreferences(nextDraft),
    onSuccess: async (preferences) => {
      setDraft(stripUpdatedAt(preferences));
      setThemeMode(preferences.theme as ThemeMode);
      setSoundEnabled(preferences.soundEnabled);
      setSaveFeedback("Settings saved.");
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["user-preferences"] }),
        queryClient.invalidateQueries({ queryKey: ["runtime-health"] }),
        queryClient.invalidateQueries({ queryKey: ["tracking-status"] }),
        queryClient.invalidateQueries({ queryKey: ["gamification-overview"] }),
      ]);
    },
    onError: () => {
      setSaveFeedback("Settings could not be saved.");
    },
  });
  const createBackupMutation = useMutation({
    mutationFn: createLocalBackup,
    onSuccess: async (backup) => {
      setBackupFeedback(`Backup created at ${backup.path}.`);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["local-backups"] }),
        queryClient.invalidateQueries({ queryKey: ["runtime-health"] }),
      ]);
    },
    onError: () => {
      setBackupFeedback("Backup creation failed.");
    },
  });
  const restoreBackupMutation = useMutation({
    mutationFn: restoreLocalBackup,
    onSuccess: async (backup) => {
      const refreshedPreferences = await queryClient.fetchQuery({
        queryKey: ["user-preferences"],
        queryFn: getUserPreferences,
      });

      setDraft(stripUpdatedAt(refreshedPreferences));
      setThemeMode(refreshedPreferences.theme as ThemeMode);
      setSoundEnabled(refreshedPreferences.soundEnabled);
      setBackupFeedback(`Backup restored from ${backup.path}.`);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["local-backups"] }),
        queryClient.invalidateQueries({ queryKey: ["runtime-health"] }),
        queryClient.invalidateQueries({ queryKey: ["tracking-status"] }),
        queryClient.invalidateQueries({ queryKey: ["recent-sessions"] }),
        queryClient.invalidateQueries({ queryKey: ["history-sessions"] }),
        queryClient.invalidateQueries({ queryKey: ["gamification-overview"] }),
      ]);
    },
    onError: () => {
      setBackupFeedback("Backup restoration failed.");
    },
  });

  const isDirty =
    draft !== null &&
    preferencesQuery.data !== undefined &&
    !sameDraft(draft, stripUpdatedAt(preferencesQuery.data));
  const backupCount = backupsQuery.data?.length ?? 0;
  const saveDisabled = !draft || saveMutation.isPending || !isDirty;
  const runtime = runtimeQuery.data;

  const summaryItems = useMemo(
    () => [
      {
        label: "Focus cycle",
        value: draft
          ? `${draft.focusMinutes}/${draft.shortBreakMinutes}/${draft.longBreakMinutes} min`
          : "--",
      },
      {
        label: "Theme",
        value: draft ? capitalize(draft.theme) : "--",
      },
      {
        label: "Backups",
        value: `${backupCount} archive${backupCount === 1 ? "" : "s"}`,
      },
    ],
    [backupCount, draft],
  );

  return (
    <div className="grid gap-5">
      <Card className="ft-panel-strong overflow-hidden p-6">
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1.2fr)_minmax(20rem,0.8fr)] lg:items-end">
          <div className="space-y-3">
            <div className="ft-kicker text-xs">Preferences</div>
            <div className="space-y-2">
              <CardTitle className="text-3xl sm:text-4xl">
                Keep the desktop app predictable, local and easy to recover.
              </CardTitle>
              <CardDescription className="max-w-2xl text-sm sm:text-base">
                Tune your focus defaults, control tray and startup behavior,
                create local backups and inspect the runtime in one place.
              </CardDescription>
            </div>
          </div>

          <div className="grid gap-3 sm:grid-cols-3">
            {summaryItems.map((item) => (
              <div
                className="ft-panel-muted rounded-[1.25rem] px-4 py-4"
                key={item.label}
              >
                <p className="ft-text-muted text-sm">{item.label}</p>
                <p className="mt-2 text-sm font-medium">{item.value}</p>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {preferencesQuery.isLoading && !draft ? (
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Settings</CardDescription>
            <CardTitle>Loading your local preferences...</CardTitle>
          </CardHeader>
        </Card>
      ) : null}

      {preferencesQuery.isError ? (
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Settings unavailable</CardDescription>
            <CardTitle>Your preferences could not be loaded.</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-3">
            <Button
              onClick={() => {
                void preferencesQuery.refetch();
              }}
              variant="secondary"
            >
              Retry
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {draft ? (
        <>
          <div className="grid gap-5 xl:grid-cols-[minmax(0,1.2fr)_minmax(22rem,0.8fr)]">
            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Focus cycle</CardDescription>
                <CardTitle>
                  Adjust the default rhythm of each session.
                </CardTitle>
              </CardHeader>

              <CardContent className="grid gap-5">
                <div className="grid gap-3 md:grid-cols-2">
                  <NumberField
                    label="Focus minutes"
                    value={draft.focusMinutes}
                    onChange={(value) =>
                      setDraft((current) =>
                        current ? { ...current, focusMinutes: value } : current,
                      )
                    }
                  />
                  <NumberField
                    label="Short break"
                    value={draft.shortBreakMinutes}
                    onChange={(value) =>
                      setDraft((current) =>
                        current
                          ? { ...current, shortBreakMinutes: value }
                          : current,
                      )
                    }
                  />
                  <NumberField
                    label="Long break"
                    value={draft.longBreakMinutes}
                    onChange={(value) =>
                      setDraft((current) =>
                        current
                          ? { ...current, longBreakMinutes: value }
                          : current,
                      )
                    }
                  />
                  <NumberField
                    label="Sessions before long break"
                    value={draft.sessionsUntilLongBreak}
                    onChange={(value) =>
                      setDraft((current) =>
                        current
                          ? { ...current, sessionsUntilLongBreak: value }
                          : current,
                      )
                    }
                  />
                </div>

                <div className="grid gap-3 md:grid-cols-2">
                  <ToggleField
                    label="Auto-start breaks"
                    checked={draft.autoStartBreaks}
                    onChange={(checked) =>
                      setDraft((current) =>
                        current
                          ? { ...current, autoStartBreaks: checked }
                          : current,
                      )
                    }
                  />
                  <ToggleField
                    label="Auto-start focus"
                    checked={draft.autoStartFocus}
                    onChange={(checked) =>
                      setDraft((current) =>
                        current
                          ? { ...current, autoStartFocus: checked }
                          : current,
                      )
                    }
                  />
                </div>
              </CardContent>
            </Card>

            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Goals</CardDescription>
                <CardTitle>Keep the weekly targets realistic.</CardTitle>
              </CardHeader>

              <CardContent className="grid gap-5">
                <NumberField
                  label="Weekly focus goal"
                  suffix="minutes"
                  value={draft.weeklyFocusGoalMinutes}
                  onChange={(value) =>
                    setDraft((current) =>
                      current
                        ? { ...current, weeklyFocusGoalMinutes: value }
                        : current,
                    )
                  }
                />
                <NumberField
                  label="Weekly completed sessions"
                  value={draft.weeklyCompletedSessionsGoal}
                  onChange={(value) =>
                    setDraft((current) =>
                      current
                        ? { ...current, weeklyCompletedSessionsGoal: value }
                        : current,
                    )
                  }
                />
              </CardContent>
            </Card>
          </div>

          <div className="grid gap-5 xl:grid-cols-[minmax(0,1.15fr)_minmax(22rem,0.85fr)]">
            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Tracking and alerts</CardDescription>
                <CardTitle>
                  Choose what the desktop app is allowed to do.
                </CardTitle>
              </CardHeader>

              <CardContent className="grid gap-3">
                <ToggleField
                  label="Tracking enabled"
                  description="Watch active windows only while a focus session is running."
                  checked={draft.trackingEnabled}
                  onChange={(checked) =>
                    setDraft((current) =>
                      current
                        ? { ...current, trackingEnabled: checked }
                        : current,
                    )
                  }
                />
                <ToggleField
                  label="Tracking permission granted"
                  checked={draft.trackingPermissionGranted}
                  onChange={(checked) =>
                    setDraft((current) =>
                      current
                        ? { ...current, trackingPermissionGranted: checked }
                        : current,
                    )
                  }
                />
                <ToggleField
                  label="Tracking onboarding completed"
                  checked={draft.trackingOnboardingCompleted}
                  onChange={(checked) =>
                    setDraft((current) =>
                      current
                        ? { ...current, trackingOnboardingCompleted: checked }
                        : current,
                    )
                  }
                />
                <ToggleField
                  label="Desktop notifications"
                  checked={draft.notificationsEnabled}
                  onChange={(checked) =>
                    setDraft((current) =>
                      current
                        ? { ...current, notificationsEnabled: checked }
                        : current,
                    )
                  }
                />
                <ToggleField
                  label="Sound cues"
                  checked={draft.soundEnabled}
                  onChange={(checked) =>
                    setDraft((current) =>
                      current ? { ...current, soundEnabled: checked } : current,
                    )
                  }
                />
              </CardContent>
            </Card>

            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Desktop behavior</CardDescription>
                <CardTitle>
                  Control startup, tray visibility and close behavior.
                </CardTitle>
              </CardHeader>

              <CardContent className="grid gap-5">
                <label className="grid gap-2">
                  <span className="text-sm font-medium">Theme</span>
                  <select
                    className={inputClassName}
                    value={draft.theme}
                    onChange={(event) =>
                      setDraft((current) =>
                        current
                          ? {
                              ...current,
                              theme: event.target
                                .value as SettingsDraft["theme"],
                            }
                          : current,
                      )
                    }
                  >
                    <option value="system">System</option>
                    <option value="light">Light</option>
                    <option value="dark">Dark</option>
                  </select>
                </label>

                <div className="grid gap-3">
                  <ToggleField
                    label="Launch on startup"
                    checked={draft.launchOnStartup}
                    onChange={(checked) =>
                      setDraft((current) =>
                        current
                          ? { ...current, launchOnStartup: checked }
                          : current,
                      )
                    }
                  />
                  <ToggleField
                    label="Show tray icon"
                    checked={draft.trayEnabled}
                    onChange={(checked) =>
                      setDraft((current) =>
                        current
                          ? {
                              ...current,
                              trayEnabled: checked,
                              closeToTray: checked
                                ? current.closeToTray
                                : false,
                            }
                          : current,
                      )
                    }
                  />
                  <ToggleField
                    label="Close to tray"
                    description="Hide the window instead of exiting when the close button is used."
                    checked={draft.closeToTray}
                    disabled={!draft.trayEnabled}
                    onChange={(checked) =>
                      setDraft((current) =>
                        current
                          ? { ...current, closeToTray: checked }
                          : current,
                      )
                    }
                  />
                </div>

                {runtime ? (
                  <div className="ft-panel-muted grid gap-2 rounded-[1rem] p-4 text-sm">
                    <InfoRow
                      label="Autostart plugin"
                      value={
                        runtime.launchOnStartupEnabled ? "Enabled" : "Disabled"
                      }
                    />
                    <InfoRow
                      label="Tray behavior"
                      value={runtime.trayEnabled ? "Visible" : "Hidden"}
                    />
                    <InfoRow
                      label="Close behavior"
                      value={runtime.closeToTray ? "Close to tray" : "Exit app"}
                    />
                  </div>
                ) : null}
              </CardContent>
            </Card>
          </div>

          <Card className="ft-panel p-6">
            <CardHeader>
              <CardDescription>Backup and restore</CardDescription>
              <CardTitle>Save a local archive before you experiment.</CardTitle>
            </CardHeader>

            <CardContent className="grid gap-5">
              <div className="flex flex-wrap items-center gap-3">
                <Button
                  disabled={createBackupMutation.isPending}
                  onClick={() => createBackupMutation.mutate()}
                >
                  Create backup
                </Button>
                <Button
                  onClick={() => {
                    void backupsQuery.refetch();
                  }}
                  variant="secondary"
                >
                  Refresh archives
                </Button>
                {backupFeedback ? (
                  <p className="ft-text-muted text-sm">{backupFeedback}</p>
                ) : null}
              </div>

              {backupsQuery.data?.length ? (
                <div className="grid gap-3">
                  {backupsQuery.data.map((backup) => (
                    <BackupRow
                      backup={backup}
                      key={backup.path}
                      restoring={restoreBackupMutation.isPending}
                      onRestore={() =>
                        restoreBackupMutation.mutate(backup.path)
                      }
                    />
                  ))}
                </div>
              ) : (
                <div className="ft-panel-muted rounded-[1rem] px-4 py-4 text-sm">
                  No local backup has been created yet.
                </div>
              )}
            </CardContent>
          </Card>

          <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(24rem,1fr)]">
            <Card className="ft-panel p-6">
              <CardHeader>
                <CardDescription>Runtime health</CardDescription>
                <CardTitle>
                  Inspect the local environment and storage paths.
                </CardTitle>
              </CardHeader>

              <CardContent className="grid gap-3">
                <InfoRow
                  label="Product"
                  value={runtime?.productName ?? "Loading"}
                />
                <InfoRow label="Version" value={runtime?.appVersion ?? "--"} />
                <InfoRow
                  label="Desktop shell"
                  value={runtime?.desktopShell ?? "--"}
                />
                <InfoRow label="Platform" value={runtime?.platform ?? "--"} />
                <InfoRow
                  label="Persistence mode"
                  value={runtime?.persistenceMode ?? "--"}
                />
                <InfoRow
                  label="Workspace crates"
                  value={runtime?.workspaceCrates.join(", ") ?? "--"}
                />
                <PathRow
                  label="App data dir"
                  value={runtime?.appDataDir ?? "--"}
                />
                <PathRow
                  label="Backup dir"
                  value={runtime?.backupDir ?? "--"}
                />
              </CardContent>
            </Card>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <Button
              disabled={saveDisabled}
              onClick={() => draft && saveMutation.mutate(draft)}
            >
              Save settings
            </Button>
            <Button
              disabled={
                !draft || !preferencesQuery.data || saveMutation.isPending
              }
              onClick={() =>
                preferencesQuery.data &&
                setDraft(stripUpdatedAt(preferencesQuery.data))
              }
              variant="secondary"
            >
              Reset changes
            </Button>
            {saveFeedback ? (
              <p className="ft-text-muted text-sm">{saveFeedback}</p>
            ) : null}
          </div>
        </>
      ) : null}
    </div>
  );
}

function NumberField({
  label,
  value,
  onChange,
  suffix,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  suffix?: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="text-sm font-medium">{label}</span>
      <div className="grid gap-2">
        <input
          className={inputClassName}
          min={1}
          step={1}
          type="number"
          value={String(value)}
          onChange={(event) =>
            onChange(parsePositiveInteger(event.target.value, value))
          }
        />
        {suffix ? (
          <span className="ft-text-muted text-xs">{suffix}</span>
        ) : null}
      </div>
    </label>
  );
}

function ToggleField({
  label,
  checked,
  onChange,
  description,
  disabled,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
  disabled?: boolean;
}) {
  return (
    <label className="ft-panel-muted flex items-start justify-between gap-4 rounded-[1rem] px-4 py-4">
      <div className="grid gap-1">
        <span className="text-sm font-medium">{label}</span>
        {description ? (
          <span className="ft-text-muted text-sm">{description}</span>
        ) : null}
      </div>
      <input
        checked={checked}
        className="mt-1 h-4 w-4 accent-[var(--color-brand)]"
        disabled={disabled}
        type="checkbox"
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}

function BackupRow({
  backup,
  restoring,
  onRestore,
}: {
  backup: BackupArchiveSummary;
  restoring: boolean;
  onRestore: () => void;
}) {
  return (
    <div className="ft-panel-muted grid gap-3 rounded-[1rem] p-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-center">
      <div className="grid gap-1 text-sm">
        <p className="font-medium">{backup.fileName}</p>
        <p className="ft-text-muted">
          {formatDateTime(backup.createdAt)} · {formatBytes(backup.sizeBytes)}
        </p>
        <p className="ft-text-muted break-all">{backup.path}</p>
      </div>
      <Button disabled={restoring} onClick={onRestore} variant="secondary">
        Restore
      </Button>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="ft-panel-muted flex items-center justify-between gap-4 rounded-[1rem] px-4 py-3 text-sm">
      <span className="ft-text-muted">{label}</span>
      <span className="text-right font-medium">{value}</span>
    </div>
  );
}

function PathRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="ft-panel-muted grid gap-2 rounded-[1rem] px-4 py-3 text-sm">
      <span className="ft-text-muted">{label}</span>
      <span className="break-all font-medium">{value}</span>
    </div>
  );
}

function stripUpdatedAt(preferences: UserPreference): SettingsDraft {
  const request = { ...preferences };
  Reflect.deleteProperty(request, "updatedAt");

  return request;
}

function sameDraft(left: SettingsDraft, right: SettingsDraft) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function parsePositiveInteger(value: string, fallback: number) {
  const parsed = Number.parseInt(value, 10);

  if (!Number.isFinite(parsed)) {
    return fallback;
  }

  return Math.max(1, parsed);
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function formatBytes(size: number) {
  if (size < 1024) {
    return `${size} B`;
  }

  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(1)} KB`;
  }

  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

function capitalize(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
