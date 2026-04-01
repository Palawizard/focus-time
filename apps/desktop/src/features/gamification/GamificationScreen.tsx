import { useEffect, useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Flame, Medal, Target, Trophy } from "lucide-react";

import { Button } from "../../components/ui/Button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../components/ui/Card";
import {
  getGamificationOverview,
  getUserPreferences,
  saveUserPreferences,
} from "../../lib/storage";
import { cn } from "../../lib/cn";
import type {
  AchievementProgress,
  GamificationOverview,
  ProgressBadge,
  UserPreference,
} from "../../types/storage";

const inputClassName =
  "ft-panel-muted h-11 rounded-[1rem] border border-[var(--color-border)] bg-transparent px-4 text-sm outline-none";

export function GamificationScreen() {
  const queryClient = useQueryClient();
  const gamificationQuery = useQuery({
    queryKey: ["gamification-overview"],
    queryFn: getGamificationOverview,
  });
  const preferencesQuery = useQuery({
    queryKey: ["user-preferences"],
    queryFn: getUserPreferences,
  });
  const preferences = preferencesQuery.data;
  const [focusGoalDraft, setFocusGoalDraft] = useState("240");
  const [sessionsGoalDraft, setSessionsGoalDraft] = useState("5");

  useEffect(() => {
    if (!preferences) {
      return;
    }

    setFocusGoalDraft(String(preferences.weeklyFocusGoalMinutes));
    setSessionsGoalDraft(String(preferences.weeklyCompletedSessionsGoal));
  }, [preferences]);

  const saveGoalsMutation = useMutation({
    mutationFn: async () => {
      const preferences = preferencesQuery.data;
      if (!preferences) {
        throw new Error("Preferences are not loaded yet.");
      }

      return saveUserPreferences({
        ...stripUpdatedAt(preferences),
        weeklyFocusGoalMinutes: sanitizeGoal(focusGoalDraft, 240),
        weeklyCompletedSessionsGoal: sanitizeGoal(sessionsGoalDraft, 5),
      });
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["user-preferences"] }),
        queryClient.invalidateQueries({ queryKey: ["gamification-overview"] }),
      ]);
    },
  });

  const isDirty =
    preferencesQuery.data !== undefined &&
    (sanitizeGoal(
      focusGoalDraft,
      preferencesQuery.data.weeklyFocusGoalMinutes,
    ) !== preferencesQuery.data.weeklyFocusGoalMinutes ||
      sanitizeGoal(
        sessionsGoalDraft,
        preferencesQuery.data.weeklyCompletedSessionsGoal,
      ) !== preferencesQuery.data.weeklyCompletedSessionsGoal);

  return (
    <div className="grid gap-5">
      <Card className="ft-panel-strong overflow-hidden p-6">
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1.2fr)_minmax(20rem,0.8fr)] lg:items-end">
          <div className="space-y-3">
            <div className="ft-kicker text-xs">Consistency</div>
            <div className="space-y-2">
              <CardTitle className="text-3xl sm:text-4xl">
                Build steady momentum without noise.
              </CardTitle>
              <CardDescription className="max-w-2xl text-sm sm:text-base">
                Track a simple daily streak, tune your weekly targets and let
                achievements unlock quietly in the background.
              </CardDescription>
            </div>
          </div>

          <div className="grid gap-3 sm:grid-cols-3">
            <MetricTile
              icon={<Flame className="h-4 w-4" />}
              label="Current streak"
              value={
                gamificationQuery.data
                  ? `${gamificationQuery.data.streak.currentDays} day${gamificationQuery.data.streak.currentDays === 1 ? "" : "s"}`
                  : "--"
              }
            />
            <MetricTile
              icon={<Target className="h-4 w-4" />}
              label="Goals reached"
              value={
                gamificationQuery.data
                  ? `${gamificationQuery.data.weeklyGoal.completedGoalCount}/2`
                  : "--"
              }
            />
            <MetricTile
              icon={<Trophy className="h-4 w-4" />}
              label="Achievements"
              value={
                gamificationQuery.data
                  ? `${gamificationQuery.data.achievements.filter((item) => item.unlockedAt).length}`
                  : "--"
              }
            />
          </div>
        </div>
      </Card>

      {gamificationQuery.isLoading && !gamificationQuery.data ? (
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Progress</CardDescription>
            <CardTitle>Loading your momentum...</CardTitle>
          </CardHeader>
        </Card>
      ) : null}

      {gamificationQuery.isError ? (
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Gamification unavailable</CardDescription>
            <CardTitle>Your progress could not be loaded.</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap items-center justify-between gap-3">
            <p className="ft-text-muted text-sm">
              The local runtime did not return a valid progress snapshot.
            </p>
            <Button
              onClick={() => {
                void gamificationQuery.refetch();
              }}
              variant="secondary"
            >
              Retry
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {gamificationQuery.data ? (
        <GamificationContent
          focusGoalDraft={focusGoalDraft}
          isSaving={saveGoalsMutation.isPending}
          overview={gamificationQuery.data}
          saveError={
            saveGoalsMutation.isError
              ? "Weekly goals could not be saved."
              : null
          }
          sessionsGoalDraft={sessionsGoalDraft}
          setFocusGoalDraft={setFocusGoalDraft}
          setSessionsGoalDraft={setSessionsGoalDraft}
          canSave={Boolean(preferencesQuery.data) && isDirty}
          onSave={() => saveGoalsMutation.mutate()}
        />
      ) : null}
    </div>
  );
}

interface GamificationContentProps {
  overview: GamificationOverview;
  focusGoalDraft: string;
  sessionsGoalDraft: string;
  canSave: boolean;
  isSaving: boolean;
  saveError: string | null;
  setFocusGoalDraft: (value: string) => void;
  setSessionsGoalDraft: (value: string) => void;
  onSave: () => void;
}

function GamificationContent({
  overview,
  focusGoalDraft,
  sessionsGoalDraft,
  canSave,
  isSaving,
  saveError,
  setFocusGoalDraft,
  setSessionsGoalDraft,
  onSave,
}: GamificationContentProps) {
  return (
    <>
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.35fr)_minmax(20rem,0.85fr)]">
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Weekly goals</CardDescription>
            <CardTitle>Set targets that fit your real week.</CardTitle>
          </CardHeader>

          <CardContent className="grid gap-5">
            <div className="grid gap-3 md:grid-cols-2">
              <GoalMeter
                label="Focus minutes"
                value={`${overview.weeklyGoal.focusMinutesCompleted} / ${overview.weeklyGoal.focusGoalMinutes} min`}
                ratio={overview.weeklyGoal.focusCompletionRatio}
              />
              <GoalMeter
                label="Completed sessions"
                value={`${overview.weeklyGoal.completedSessions} / ${overview.weeklyGoal.completedSessionsGoal}`}
                ratio={overview.weeklyGoal.sessionsCompletionRatio}
              />
            </div>

            <div className="ft-panel-muted grid gap-4 p-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p className="text-sm font-medium">Current week</p>
                  <p className="ft-text-muted mt-1 text-sm">
                    {formatWeekRange(
                      overview.weeklyGoal.startDate,
                      overview.weeklyGoal.endDate,
                    )}
                  </p>
                </div>
                <span className="ft-brand-badge rounded-full px-3 py-1 text-xs font-medium">
                  {overview.weeklyGoal.completedGoalCount} of 2 goals reached
                </span>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <label className="grid gap-2">
                  <span className="ft-text-muted text-sm">Focus goal</span>
                  <input
                    className={inputClassName}
                    inputMode="numeric"
                    min={30}
                    onChange={(event) => setFocusGoalDraft(event.target.value)}
                    step={15}
                    type="number"
                    value={focusGoalDraft}
                  />
                </label>

                <label className="grid gap-2">
                  <span className="ft-text-muted text-sm">Session goal</span>
                  <input
                    className={inputClassName}
                    inputMode="numeric"
                    min={1}
                    onChange={(event) =>
                      setSessionsGoalDraft(event.target.value)
                    }
                    step={1}
                    type="number"
                    value={sessionsGoalDraft}
                  />
                </label>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-3">
                <p className="ft-text-muted text-sm">
                  Keep goals small enough to stay motivating and realistic.
                </p>
                <Button disabled={!canSave || isSaving} onClick={onSave}>
                  Save weekly goals
                </Button>
              </div>

              {saveError ? (
                <p className="text-sm text-[var(--color-danger)]">
                  {saveError}
                </p>
              ) : null}
            </div>
          </CardContent>
        </Card>

        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Status</CardDescription>
            <CardTitle>{describeStreak(overview)}</CardTitle>
          </CardHeader>

          <CardContent className="space-y-3">
            {overview.badges.map((badge) => (
              <BadgeRow badge={badge} key={badge.slug} />
            ))}
          </CardContent>
        </Card>
      </div>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Achievements</CardDescription>
          <CardTitle>Unlocks stay deterministic and low-noise.</CardTitle>
        </CardHeader>

        <CardContent className="grid gap-3 lg:grid-cols-2">
          {overview.achievements.map((achievement) => (
            <AchievementRow achievement={achievement} key={achievement.slug} />
          ))}
        </CardContent>
      </Card>
    </>
  );
}

function MetricTile({
  icon,
  label,
  value,
}: {
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="ft-panel-muted p-4">
      <div className="ft-text-muted flex items-center gap-2 text-sm">
        {icon}
        <span>{label}</span>
      </div>
      <p className="ft-font-display mt-3 text-2xl font-semibold tracking-tight">
        {value}
      </p>
    </div>
  );
}

function GoalMeter({
  label,
  value,
  ratio,
}: {
  label: string;
  value: string;
  ratio: number;
}) {
  return (
    <div className="ft-panel-muted p-4">
      <div className="flex items-center justify-between gap-3">
        <p className="text-sm font-medium">{label}</p>
        <p className="ft-text-muted text-sm">{Math.round(ratio * 100)}%</p>
      </div>
      <p className="mt-3 text-sm">{value}</p>
      <div className="mt-3 h-2 overflow-hidden rounded-full bg-[var(--color-surface-muted)]">
        <div
          className="h-full rounded-full bg-[var(--color-brand)] transition-[width] duration-300"
          style={{ width: `${progressWidth(ratio)}%` }}
        />
      </div>
    </div>
  );
}

function BadgeRow({ badge }: { badge: ProgressBadge }) {
  return (
    <div className="ft-panel-muted rounded-[1rem] p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <p className="text-sm font-medium">{badge.title}</p>
            <span
              className={cn(
                "rounded-full px-2.5 py-1 text-[11px] font-medium",
                badge.isUnlocked
                  ? "ft-brand-badge"
                  : "bg-[var(--color-surface-muted)] text-[var(--color-text-soft)]",
              )}
            >
              {badge.isUnlocked ? "Unlocked" : "In progress"}
            </span>
          </div>
          <p className="ft-text-muted mt-2 text-sm">{badge.description}</p>
        </div>
        <div className="ft-brand-badge inline-flex h-10 w-10 items-center justify-center rounded-full">
          <Medal className="h-4 w-4" />
        </div>
      </div>

      <div className="mt-4 space-y-2">
        <div className="flex items-center justify-between gap-3 text-sm">
          <span className="ft-text-muted">{badge.progressLabel}</span>
          <span>{Math.round(badge.progressRatio * 100)}%</span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-[var(--color-surface-muted)]">
          <div
            className="h-full rounded-full bg-[var(--color-brand)] transition-[width] duration-300"
            style={{ width: `${progressWidth(badge.progressRatio)}%` }}
          />
        </div>
      </div>
    </div>
  );
}

function AchievementRow({ achievement }: { achievement: AchievementProgress }) {
  const isUnlocked = Boolean(achievement.unlockedAt);

  return (
    <div className="ft-panel-muted rounded-[1rem] p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <p className="text-sm font-medium">{achievement.title}</p>
            <span
              className={cn(
                "rounded-full px-2.5 py-1 text-[11px] font-medium",
                isUnlocked
                  ? "ft-brand-badge"
                  : "bg-[var(--color-surface-muted)] text-[var(--color-text-soft)]",
              )}
            >
              {isUnlocked ? "Unlocked" : "Locked"}
            </span>
          </div>
          <p className="ft-text-muted mt-2 text-sm">
            {achievement.description}
          </p>
        </div>
        <div className="ft-brand-badge inline-flex h-10 w-10 items-center justify-center rounded-full">
          <Trophy className="h-4 w-4" />
        </div>
      </div>

      <div className="mt-4 space-y-2">
        <div className="flex items-center justify-between gap-3 text-sm">
          <span className="ft-text-muted">
            {achievement.progressCurrent} / {achievement.progressTarget}
          </span>
          <span>{Math.round(achievement.progressRatio * 100)}%</span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-[var(--color-surface-muted)]">
          <div
            className="h-full rounded-full bg-[var(--color-brand)] transition-[width] duration-300"
            style={{ width: `${progressWidth(achievement.progressRatio)}%` }}
          />
        </div>
        <p className="ft-text-muted text-xs">
          {achievement.unlockedAt
            ? `Unlocked on ${formatDateLabel(achievement.unlockedAt)}`
            : "Still in progress."}
        </p>
      </div>
    </div>
  );
}

function describeStreak(overview: GamificationOverview) {
  if (overview.streak.todayCompleted) {
    return "Today is already counted in your streak.";
  }

  if (overview.streak.isAtRisk) {
    return "Your streak is alive, but today still needs a focus block.";
  }

  return "The next focused day will start a new streak.";
}

function formatWeekRange(startDate: string, endDate: string) {
  return `${formatDateLabel(startDate)} to ${formatDateLabel(endDate)}`;
}

function formatDateLabel(value: string) {
  const normalized = value.includes("T") ? value : `${value}T00:00:00`;

  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
  }).format(new Date(normalized));
}

function sanitizeGoal(value: string, fallback: number) {
  const parsed = Number(value);

  if (!Number.isFinite(parsed)) {
    return fallback;
  }

  return Math.max(1, Math.round(parsed));
}

function stripUpdatedAt(preferences: UserPreference) {
  const request = { ...preferences };
  Reflect.deleteProperty(request, "updatedAt");

  return request;
}

function progressWidth(ratio: number) {
  if (ratio <= 0) {
    return 0;
  }

  return Math.max(6, Math.round(ratio * 100));
}
