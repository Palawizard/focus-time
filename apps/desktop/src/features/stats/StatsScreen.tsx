import { useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { CheckCircle2, Flame, Laptop2, TimerReset } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../components/ui/Card";
import { Tabs, TabsList, TabsTrigger } from "../../components/ui/Tabs";
import { Button } from "../../components/ui/Button";
import { cn } from "../../lib/cn";
import { getStatsDashboard } from "../../lib/storage";
import type {
  StatsAppDistribution,
  StatsComparison,
  StatsDashboard,
  StatsPeriod,
  StatsSeriesBucket,
  StatsWeekdayBucket,
} from "../../types/storage";

const periodLabels: Record<StatsPeriod, string> = {
  day: "Today",
  week: "This week",
  month: "This month",
};

const periodDescriptions: Record<StatsPeriod, string> = {
  day: "Hourly view of your active focus blocks and breaks.",
  week: "See how your current week is pacing against the previous one.",
  month: "Read the shape of your month without digging through sessions.",
};

const categoryLabels = {
  development: "Development",
  browser: "Browser",
  communication: "Communication",
  writing: "Writing",
  design: "Design",
  meeting: "Meeting",
  research: "Research",
  utilities: "Utilities",
  unknown: "Unknown",
} as const;

export function StatsScreen() {
  const [period, setPeriod] = useState<StatsPeriod>("week");

  const statsQuery = useQuery({
    queryKey: ["stats-dashboard", period],
    queryFn: () => getStatsDashboard(period),
    placeholderData: (previousData) => previousData,
  });

  return (
    <div className="grid gap-5">
      <Card className="ft-panel-strong overflow-hidden p-6">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div className="space-y-3">
            <div className="ft-kicker text-xs">Insights</div>
            <div className="space-y-2">
              <CardTitle className="text-3xl sm:text-4xl">
                Read your current focus rhythm.
              </CardTitle>
              <CardDescription className="max-w-2xl text-sm sm:text-base">
                {periodDescriptions[period]}
              </CardDescription>
            </div>
          </div>

          <Tabs
            onValueChange={(value) => setPeriod(value as StatsPeriod)}
            value={period}
          >
            <TabsList>
              <TabsTrigger value="day">Day</TabsTrigger>
              <TabsTrigger value="week">Week</TabsTrigger>
              <TabsTrigger value="month">Month</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </Card>

      {statsQuery.isLoading && !statsQuery.data ? <StatsLoadingState /> : null}

      {statsQuery.isError ? (
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Stats unavailable</CardDescription>
            <CardTitle>Focus data could not be loaded.</CardTitle>
          </CardHeader>
          <CardContent className="flex items-center justify-between gap-4">
            <p className="ft-text-muted text-sm">
              The dashboard command did not return a valid payload. Try again
              once the desktop runtime is ready.
            </p>
            <Button onClick={() => statsQuery.refetch()} variant="secondary">
              Retry
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {statsQuery.data ? (
        <StatsDashboardView dashboard={statsQuery.data} />
      ) : null}
    </div>
  );
}

function StatsDashboardView({ dashboard }: { dashboard: StatsDashboard }) {
  const totalAppFocus = dashboard.appDistribution.reduce(
    (total, app) => total + app.focusSeconds,
    0,
  );

  if (dashboard.isEmpty) {
    return (
      <Card className="ft-panel p-8">
        <CardHeader>
          <CardDescription>{periodLabels[dashboard.period]}</CardDescription>
          <CardTitle>No focus history yet.</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="ft-text-muted text-sm">
            Start a session from the Focus screen and the dashboard will begin
            to surface your completion rate, streak and top apps automatically.
          </p>
          <div className="ft-panel-muted inline-flex w-fit rounded-full px-3 py-2 text-sm">
            Range:{" "}
            {formatRangeLabel(
              dashboard.range.startDate,
              dashboard.range.endDate,
            )}
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
      <Card className="ft-panel-strong p-6">
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)]">
          <div className="space-y-3">
            <div className="ft-kicker text-xs">
              {periodLabels[dashboard.period]}
            </div>
            <div className="space-y-2">
              <h2 className="ft-font-display text-4xl font-semibold tracking-tight sm:text-5xl">
                {formatDuration(dashboard.summary.focusSeconds)}
              </h2>
              <p className="ft-text-muted text-sm sm:text-base">
                {formatRangeLabel(
                  dashboard.range.startDate,
                  dashboard.range.endDate,
                )}
              </p>
            </div>
            <div className="flex flex-wrap gap-2 text-sm">
              <DeltaChip
                label={formatComparisonLabel(dashboard)}
                value={formatDurationDelta(
                  dashboard.comparison.focusSecondsDelta,
                  dashboard.comparison.focusSecondsRatio,
                )}
              />
              <DeltaChip
                label="Average active day"
                value={formatDuration(
                  dashboard.summary.averageFocusSecondsPerActiveDay,
                )}
              />
            </div>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <MetricPanel
              icon={<CheckCircle2 className="h-4 w-4" />}
              label="Completion rate"
              value={formatPercent(dashboard.summary.completionRate)}
              detail={formatCompletionDelta(dashboard.comparison)}
            />
            <MetricPanel
              icon={<Flame className="h-4 w-4" />}
              label="Current streak"
              value={`${dashboard.summary.streakDays} day${dashboard.summary.streakDays === 1 ? "" : "s"}`}
              detail={`Best: ${dashboard.summary.bestStreakDays} days`}
            />
            <MetricPanel
              icon={<Laptop2 className="h-4 w-4" />}
              label="Top app"
              value={dashboard.topApp?.name ?? "No app tracked"}
              detail={
                dashboard.topApp
                  ? `${formatDuration(dashboard.topApp.focusSeconds)} of tracked focus`
                  : "Tracking data is not available yet."
              }
            />
            <MetricPanel
              icon={<TimerReset className="h-4 w-4" />}
              label="Sessions"
              value={`${dashboard.summary.totalSessions}`}
              detail={`${dashboard.summary.activeDays} active day${dashboard.summary.activeDays === 1 ? "" : "s"}`}
            />
          </div>
        </div>
      </Card>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.55fr)_minmax(320px,0.9fr)]">
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Evolution</CardDescription>
            <CardTitle>Your focus trend in the selected range.</CardTitle>
          </CardHeader>
          <CardContent>
            <TrendChart
              period={dashboard.period}
              series={dashboard.focusSeries}
            />
          </CardContent>
        </Card>

        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>App distribution</CardDescription>
            <CardTitle>Where your focus time went.</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {dashboard.appDistribution.length > 0 ? (
              dashboard.appDistribution.map((app) => (
                <AppDistributionRow
                  app={app}
                  key={`${app.trackedAppId ?? "other"}-${app.name}`}
                  totalFocus={totalAppFocus}
                />
              ))
            ) : (
              <EmptyChartState label="No tracked app data in this range." />
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-5 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)]">
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Weekday distribution</CardDescription>
            <CardTitle>Which days carried the load.</CardTitle>
          </CardHeader>
          <CardContent>
            <WeekdayChart buckets={dashboard.weekdayDistribution} />
          </CardContent>
        </Card>

        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>Overview</CardDescription>
            <CardTitle>Quick read on this period.</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-3">
            <SummaryRow
              label="Focus time"
              value={formatDuration(dashboard.summary.focusSeconds)}
            />
            <SummaryRow
              label="Break time"
              value={formatDuration(dashboard.summary.breakSeconds)}
            />
            <SummaryRow
              label="Completed sessions"
              value={`${dashboard.summary.completedSessions}`}
            />
            <SummaryRow
              label="Interrupted sessions"
              value={`${dashboard.summary.interruptedSessions}`}
            />
            <SummaryRow
              label="Comparison window"
              value={formatRangeLabel(
                dashboard.range.comparisonStartDate,
                dashboard.range.comparisonEndDate,
              )}
            />
          </CardContent>
        </Card>
      </div>
    </>
  );
}

function MetricPanel({
  detail,
  icon,
  label,
  value,
}: {
  detail: string;
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="ft-panel-muted flex min-h-28 flex-col justify-between rounded-[1.35rem] p-4">
      <div className="flex items-center gap-2 text-sm text-[var(--color-text-muted)]">
        <span className="ft-brand-badge inline-flex h-8 w-8 items-center justify-center rounded-full">
          {icon}
        </span>
        <span>{label}</span>
      </div>

      <div className="space-y-1">
        <div className="ft-font-display text-2xl font-semibold tracking-tight">
          {value}
        </div>
        <div className="ft-text-muted text-sm">{detail}</div>
      </div>
    </div>
  );
}

function TrendChart({
  period,
  series,
}: {
  period: StatsPeriod;
  series: StatsSeriesBucket[];
}) {
  const maxValue = Math.max(
    1,
    ...series.flatMap((bucket) => [bucket.focusSeconds, bucket.breakSeconds]),
  );

  if (
    series.every(
      (bucket) => bucket.focusSeconds === 0 && bucket.breakSeconds === 0,
    )
  ) {
    return (
      <EmptyChartState label="No focus or break segments were recorded in this range." />
    );
  }

  return (
    <div className="space-y-4">
      <div className="grid gap-2 sm:grid-cols-3">
        <LegendPill accent="bg-[var(--color-brand)]" label="Focus" />
        <LegendPill accent="bg-white/35" label="Break" />
        <LegendPill
          accent="bg-[var(--color-brand-soft)]"
          label={period === "day" ? "Hour by hour" : "Day by day"}
        />
      </div>

      <div className="overflow-x-auto pb-2">
        <div className="flex min-w-max items-end gap-3">
          {series.map((bucket) => {
            const focusHeight = `${Math.max((bucket.focusSeconds / maxValue) * 180, bucket.focusSeconds > 0 ? 12 : 2)}px`;
            const breakHeight = `${Math.max((bucket.breakSeconds / maxValue) * 180, bucket.breakSeconds > 0 ? 8 : 2)}px`;

            return (
              <div
                className="flex min-w-[1.8rem] flex-col items-center gap-2"
                key={bucket.key}
              >
                <div className="flex h-[200px] items-end gap-1">
                  <div
                    className="w-4 rounded-t-full bg-[var(--color-brand)]"
                    style={{ height: focusHeight }}
                    title={`${bucket.label}: ${formatDuration(bucket.focusSeconds)} focus`}
                  />
                  <div
                    className="w-1.5 rounded-t-full bg-white/35"
                    style={{ height: breakHeight }}
                    title={`${bucket.label}: ${formatDuration(bucket.breakSeconds)} break`}
                  />
                </div>
                <div className="ft-text-soft text-xs">{bucket.shortLabel}</div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function WeekdayChart({ buckets }: { buckets: StatsWeekdayBucket[] }) {
  const maxValue = Math.max(1, ...buckets.map((bucket) => bucket.focusSeconds));

  return (
    <div className="grid gap-3">
      {buckets.map((bucket) => (
        <div
          className="grid grid-cols-[3.5rem_minmax(0,1fr)_auto] items-center gap-3"
          key={bucket.weekday}
        >
          <div className="text-sm">{bucket.label}</div>
          <div className="ft-panel-muted h-3 overflow-hidden rounded-full">
            <div
              className="h-full rounded-full bg-[var(--color-brand)]"
              style={{ width: `${(bucket.focusSeconds / maxValue) * 100}%` }}
            />
          </div>
          <div className="ft-text-muted text-sm">
            {formatDurationCompact(bucket.focusSeconds)}
          </div>
        </div>
      ))}
    </div>
  );
}

function AppDistributionRow({
  app,
  totalFocus,
}: {
  app: StatsAppDistribution;
  totalFocus: number;
}) {
  const share = totalFocus === 0 ? 0 : app.focusSeconds / totalFocus;
  const width = `${Math.max(share * 100, app.focusSeconds > 0 ? 8 : 0)}%`;

  return (
    <div className="ft-panel-muted rounded-[1.2rem] p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <div className="flex items-center gap-2">
            <span
              className="h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: app.colorHex ?? "var(--color-brand)" }}
            />
            <span className="truncate text-sm font-medium">{app.name}</span>
          </div>
          <div className="ft-text-muted truncate text-xs">
            {app.category ? categoryLabels[app.category] : "Grouped remainder"}
          </div>
        </div>
        <div className="text-right">
          <div className="text-sm font-medium">
            {formatDurationCompact(app.focusSeconds)}
          </div>
          <div className="ft-text-muted text-xs">{formatPercent(share)}</div>
        </div>
      </div>

      <div className="ft-panel mt-3 h-2 overflow-hidden rounded-full">
        <div
          className="h-full rounded-full"
          style={{
            backgroundColor: app.colorHex ?? "var(--color-brand)",
            width,
          }}
        />
      </div>
    </div>
  );
}

function SummaryRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 border-b border-[var(--color-border)] pb-3 last:border-b-0 last:pb-0">
      <span className="ft-text-muted text-sm">{label}</span>
      <span className="text-right text-sm font-medium">{value}</span>
    </div>
  );
}

function DeltaChip({ label, value }: { label: string; value: string }) {
  return (
    <div className="ft-panel-muted inline-flex items-center gap-2 rounded-full px-3 py-2">
      <span className="ft-text-muted text-xs uppercase tracking-[0.2em]">
        {label}
      </span>
      <span className="text-sm font-medium">{value}</span>
    </div>
  );
}

function LegendPill({ accent, label }: { accent: string; label: string }) {
  return (
    <div className="ft-panel-muted inline-flex items-center gap-2 rounded-full px-3 py-2 text-sm">
      <span className={cn("h-2.5 w-2.5 rounded-full", accent)} />
      <span>{label}</span>
    </div>
  );
}

function EmptyChartState({ label }: { label: string }) {
  return (
    <div className="ft-panel-muted flex min-h-40 items-center justify-center rounded-[1.3rem] px-6 text-center">
      <p className="ft-text-muted max-w-sm text-sm">{label}</p>
    </div>
  );
}

function StatsLoadingState() {
  return (
    <div className="grid gap-5">
      <Card className="ft-panel p-6">
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,0.85fr)]">
          <LoadingBlock className="h-28" />
          <LoadingBlock className="h-28" />
        </div>
      </Card>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.55fr)_minmax(320px,0.9fr)]">
        <LoadingBlock className="h-[26rem]" />
        <LoadingBlock className="h-[26rem]" />
      </div>
    </div>
  );
}

function LoadingBlock({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "ft-panel-muted animate-pulse rounded-[1.35rem] bg-white/5",
        className,
      )}
    />
  );
}

function formatRangeLabel(startDate: string, endDate: string) {
  const formatter = new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
  });

  if (startDate === endDate) {
    return formatter.format(new Date(`${startDate}T12:00:00Z`));
  }

  return `${formatter.format(new Date(`${startDate}T12:00:00Z`))} - ${formatter.format(
    new Date(`${endDate}T12:00:00Z`),
  )}`;
}

function formatDuration(seconds: number) {
  const totalMinutes = Math.round(Math.max(0, seconds) / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours === 0) {
    return `${minutes}m`;
  }

  if (minutes === 0) {
    return `${hours}h`;
  }

  return `${hours}h ${minutes}m`;
}

function formatDurationCompact(seconds: number) {
  const safeSeconds = Math.max(0, seconds);
  const hours = Math.floor(safeSeconds / 3_600);
  const minutes = Math.floor((safeSeconds % 3_600) / 60);

  if (hours === 0) {
    return `${minutes}m`;
  }

  return `${hours}h ${minutes.toString().padStart(2, "0")}m`;
}

function formatDurationDelta(deltaSeconds: number, ratio: number | null) {
  const sign = deltaSeconds >= 0 ? "+" : "-";
  const duration = formatDuration(Math.abs(deltaSeconds));

  if (ratio === null) {
    return `${sign}${duration}`;
  }

  return `${sign}${duration} (${formatPercent(Math.abs(ratio))})`;
}

function formatPercent(value: number) {
  return `${Math.round(value * 100)}%`;
}

function formatCompletionDelta(comparison: StatsComparison) {
  const delta = comparison.completionRateDelta;
  const points = `${delta >= 0 ? "+" : "-"}${Math.abs(delta * 100).toFixed(0)} pts vs previous`;

  return points;
}

function formatComparisonLabel(dashboard: StatsDashboard) {
  switch (dashboard.period) {
    case "day":
      return "vs yesterday";
    case "week":
      return "vs previous week";
    case "month":
      return "vs previous month";
  }
}
