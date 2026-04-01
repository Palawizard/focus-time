import { useEffect, useState, type ReactNode } from "react";
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/Dialog";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "../../components/ui/Tabs";
import {
  deleteSession,
  exportHistory,
  getHistorySessionDetail,
  listHistorySessions,
  listTrackedApps,
  replaceSession,
} from "../../lib/storage";
import { cn } from "../../lib/cn";
import type {
  HistoryExportFormat,
  HistorySessionDetail,
  HistorySessionSummary,
  ReplaceSessionRequest,
  SessionHistoryFilters,
  SessionSegmentDetail,
  SessionSegmentKind,
  SessionStatus,
  TrackedWindowEvent,
  TrackingCategory,
} from "../../types/storage";

const PAGE_SIZE = 12;

const statusOptions: Array<{ value: SessionStatus; label: string }> = [
  { value: "planned", label: "Planned" },
  { value: "in_progress", label: "In progress" },
  { value: "completed", label: "Completed" },
  { value: "cancelled", label: "Cancelled" },
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

type FilterDraft = {
  dateFrom: string;
  dateTo: string;
  minDurationMinutes: string;
  maxDurationMinutes: string;
  presetLabel: string;
  status: "" | SessionStatus;
  trackedAppId: string;
};

type EditDraft = {
  startedAt: string;
  endedAt: string;
  plannedFocusMinutes: string;
  actualFocusMinutes: string;
  breakMinutes: string;
  status: SessionStatus;
  presetLabel: string;
  note: string;
};

const emptyFilters: FilterDraft = {
  dateFrom: "",
  dateTo: "",
  minDurationMinutes: "",
  maxDurationMinutes: "",
  presetLabel: "",
  status: "",
  trackedAppId: "",
};

const inputClassName =
  "ft-panel-muted h-11 rounded-[1rem] border border-[var(--color-border)] bg-transparent px-4 text-sm outline-none";

export function HistoryScreen() {
  const queryClient = useQueryClient();
  const [filtersDraft, setFiltersDraft] = useState<FilterDraft>(emptyFilters);
  const [appliedFilters, setAppliedFilters] = useState<SessionHistoryFilters>(
    {},
  );
  const [offset, setOffset] = useState(0);
  const [items, setItems] = useState<HistorySessionSummary[]>([]);
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [selectedSessionId, setSelectedSessionId] = useState<number | null>(
    null,
  );
  const [detailOpen, setDetailOpen] = useState(false);
  const [editDraft, setEditDraft] = useState<EditDraft | null>(null);
  const [historyFeedback, setHistoryFeedback] = useState<string | null>(null);

  const trackedAppsQuery = useQuery({
    queryKey: ["tracked-apps"],
    queryFn: listTrackedApps,
  });

  const sessionsQuery = useQuery({
    queryKey: ["history-sessions", appliedFilters, offset],
    queryFn: () => listHistorySessions(PAGE_SIZE, offset, appliedFilters),
  });

  const detailQuery = useQuery({
    queryKey: ["history-session-detail", selectedSessionId],
    queryFn: () => getHistorySessionDetail(selectedSessionId ?? 0),
    enabled: selectedSessionId !== null,
  });

  useEffect(() => {
    if (!sessionsQuery.data) {
      return;
    }

    setNextOffset(sessionsQuery.data.nextOffset);
    setItems((current) => {
      if (offset === 0) {
        return sessionsQuery.data.items;
      }

      const merged = [...current];
      sessionsQuery.data.items.forEach((item) => {
        if (
          !merged.some((existing) => existing.session.id === item.session.id)
        ) {
          merged.push(item);
        }
      });

      return merged;
    });
  }, [offset, sessionsQuery.data]);

  useEffect(() => {
    if (!detailQuery.data) {
      return;
    }

    setEditDraft(buildEditDraft(detailQuery.data));
  }, [detailQuery.data]);

  const invalidateHistory = async (sessionId?: number) => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["history-sessions"] }),
      queryClient.invalidateQueries({ queryKey: ["recent-sessions"] }),
      queryClient.invalidateQueries({ queryKey: ["tracked-window-events"] }),
      ...(sessionId !== undefined
        ? [
            queryClient.invalidateQueries({
              queryKey: ["history-session-detail", sessionId],
            }),
          ]
        : []),
    ]);
  };

  const replaceSessionMutation = useMutation({
    mutationFn: (request: ReplaceSessionRequest) => replaceSession(request),
    onSuccess: async (_, variables) => {
      setHistoryFeedback("Session updated.");
      await invalidateHistory(variables.sessionId);
      await detailQuery.refetch();
    },
  });

  const deleteSessionMutation = useMutation({
    mutationFn: deleteSession,
    onSuccess: async (_, sessionId) => {
      setHistoryFeedback(`Session #${sessionId} deleted.`);
      setDetailOpen(false);
      setSelectedSessionId(null);
      setEditDraft(null);
      setOffset(0);
      setItems([]);
      await invalidateHistory(sessionId);
    },
  });

  const exportMutation = useMutation({
    mutationFn: (format: HistoryExportFormat) =>
      exportHistory({
        format,
        filters: hasFilters(appliedFilters) ? appliedFilters : undefined,
      }),
    onSuccess: (result) => {
      setHistoryFeedback(
        `${result.sessionsExported} session${result.sessionsExported === 1 ? "" : "s"} exported to ${result.path}.`,
      );
    },
  });

  const summaries = buildSummaryMetrics(items);
  const selectedDetail = detailQuery.data;
  const selectedSession = selectedDetail?.session;

  const applyFilters = () => {
    setHistoryFeedback(null);
    setItems([]);
    setOffset(0);
    setAppliedFilters(normalizeFilters(filtersDraft));
  };

  const clearFilters = () => {
    setFiltersDraft(emptyFilters);
    setItems([]);
    setOffset(0);
    setAppliedFilters({});
    setHistoryFeedback(null);
  };

  const openDetail = (sessionId: number) => {
    setSelectedSessionId(sessionId);
    setDetailOpen(true);
    setHistoryFeedback(null);
  };

  const saveEdit = async () => {
    if (!selectedSession || !editDraft) {
      return;
    }

    await replaceSessionMutation.mutateAsync({
      sessionId: selectedSession.id,
      startedAt: toIsoString(editDraft.startedAt),
      endedAt: editDraft.endedAt ? toIsoString(editDraft.endedAt) : null,
      plannedFocusMinutes: parseWholeNumber(editDraft.plannedFocusMinutes),
      actualFocusSeconds: parseMinutesToSeconds(editDraft.actualFocusMinutes),
      breakSeconds: parseMinutesToSeconds(editDraft.breakMinutes),
      status: editDraft.status,
      presetLabel: editDraft.presetLabel.trim() || null,
      note: editDraft.note.trim() || null,
    });
  };

  const requestDelete = async () => {
    if (!selectedSession) {
      return;
    }

    await deleteSessionMutation.mutateAsync(selectedSession.id);
  };

  return (
    <>
      <div className="grid gap-5">
        <Card className="ft-panel p-6">
          <CardHeader>
            <CardDescription>History</CardDescription>
            <CardTitle>
              Review, correct and export your past sessions.
            </CardTitle>
          </CardHeader>

          <CardContent className="grid gap-4">
            <div className="grid gap-3 lg:grid-cols-4">
              <MetricCard
                label="Loaded sessions"
                value={String(items.length)}
              />
              <MetricCard
                label="Focus time"
                value={formatDurationWords(summaries.focusSeconds)}
              />
              <MetricCard
                label="Interruptions"
                value={`${summaries.interruptionCount} event${summaries.interruptionCount === 1 ? "" : "s"}`}
              />
              <MetricCard
                label="Filtered presets"
                value={appliedFilters.presetLabel ?? "All presets"}
              />
            </div>

            <div className="grid gap-4 xl:grid-cols-[minmax(0,1.3fr)_minmax(20rem,0.7fr)]">
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                <Field label="From">
                  <input
                    className={inputClassName}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        dateFrom: event.target.value,
                      }))
                    }
                    type="date"
                    value={filtersDraft.dateFrom}
                  />
                </Field>

                <Field label="To">
                  <input
                    className={inputClassName}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        dateTo: event.target.value,
                      }))
                    }
                    type="date"
                    value={filtersDraft.dateTo}
                  />
                </Field>

                <Field label="Status">
                  <select
                    className={inputClassName}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        status: event.target.value as FilterDraft["status"],
                      }))
                    }
                    value={filtersDraft.status}
                  >
                    <option value="">All statuses</option>
                    {statusOptions.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </Field>

                <Field label="Min duration (min)">
                  <input
                    className={inputClassName}
                    min={0}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        minDurationMinutes: event.target.value,
                      }))
                    }
                    placeholder="15"
                    step={1}
                    type="number"
                    value={filtersDraft.minDurationMinutes}
                  />
                </Field>

                <Field label="Max duration (min)">
                  <input
                    className={inputClassName}
                    min={0}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        maxDurationMinutes: event.target.value,
                      }))
                    }
                    placeholder="90"
                    step={1}
                    type="number"
                    value={filtersDraft.maxDurationMinutes}
                  />
                </Field>

                <Field label="App">
                  <select
                    className={inputClassName}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        trackedAppId: event.target.value,
                      }))
                    }
                    value={filtersDraft.trackedAppId}
                  >
                    <option value="">All tracked apps</option>
                    {trackedAppsQuery.data?.map((trackedApp) => (
                      <option key={trackedApp.id} value={trackedApp.id}>
                        {trackedApp.name}
                      </option>
                    ))}
                  </select>
                </Field>

                <Field className="md:col-span-2 xl:col-span-3" label="Preset">
                  <input
                    className={inputClassName}
                    onChange={(event) =>
                      setFiltersDraft((current) => ({
                        ...current,
                        presetLabel: event.target.value,
                      }))
                    }
                    placeholder="Classic"
                    value={filtersDraft.presetLabel}
                  />
                </Field>
              </div>

              <Card className="p-5">
                <CardDescription>Local export</CardDescription>
                <CardTitle className="mt-2 text-xl">
                  Save this filtered history as CSV or JSON.
                </CardTitle>
                <p className="ft-text-muted mt-3 text-sm">
                  Export uses the active filters and writes a file to your local
                  documents or downloads folder.
                </p>

                <div className="mt-5 flex flex-wrap gap-3">
                  <Button
                    disabled={exportMutation.isPending}
                    onClick={() => exportMutation.mutate("csv")}
                    variant="secondary"
                  >
                    Export CSV
                  </Button>
                  <Button
                    disabled={exportMutation.isPending}
                    onClick={() => exportMutation.mutate("json")}
                    variant="secondary"
                  >
                    Export JSON
                  </Button>
                </div>
              </Card>
            </div>

            <div className="flex flex-wrap gap-3">
              <Button onClick={applyFilters}>Apply filters</Button>
              <Button onClick={clearFilters} variant="ghost">
                Clear filters
              </Button>
            </div>

            {historyFeedback ? (
              <p className="text-sm text-[var(--color-text)]">
                {historyFeedback}
              </p>
            ) : null}
            {sessionsQuery.isError ? (
              <p className="text-sm text-[var(--color-danger)]">
                {(sessionsQuery.error as Error).message}
              </p>
            ) : null}
          </CardContent>
        </Card>

        <div className="grid gap-5 xl:grid-cols-[minmax(0,1.4fr)_minmax(20rem,0.6fr)]">
          <Card className="ft-panel p-6">
            <CardHeader>
              <CardDescription>Sessions</CardDescription>
              <CardTitle>Browse recent focus runs and their context.</CardTitle>
            </CardHeader>

            <CardContent className="space-y-3">
              {sessionsQuery.isLoading && items.length === 0 ? (
                <p className="ft-text-muted text-sm">
                  Loading session history...
                </p>
              ) : null}

              {!sessionsQuery.isLoading && items.length === 0 ? (
                <EmptyState />
              ) : null}

              {items.map((item) => (
                <SessionRow
                  item={item}
                  key={item.session.id}
                  onOpen={() => openDetail(item.session.id)}
                />
              ))}

              {nextOffset !== null ? (
                <Button
                  disabled={sessionsQuery.isFetching}
                  onClick={() => setOffset(nextOffset)}
                  variant="secondary"
                >
                  {sessionsQuery.isFetching ? "Loading..." : "Load more"}
                </Button>
              ) : null}
            </CardContent>
          </Card>

          <Card className="ft-panel p-6">
            <CardHeader>
              <CardDescription>Filters recap</CardDescription>
              <CardTitle>Keep your review focused.</CardTitle>
            </CardHeader>

            <CardContent className="space-y-3">
              <FilterRecapItem
                label="Dates"
                value={
                  appliedFilters.dateFrom || appliedFilters.dateTo
                    ? `${appliedFilters.dateFrom ?? "Any"} -> ${appliedFilters.dateTo ?? "Any"}`
                    : "Any day"
                }
              />
              <FilterRecapItem
                label="Duration"
                value={
                  appliedFilters.minDurationSeconds ||
                  appliedFilters.maxDurationSeconds
                    ? `${formatOptionalMinutes(appliedFilters.minDurationSeconds)} to ${formatOptionalMinutes(appliedFilters.maxDurationSeconds)}`
                    : "Any duration"
                }
              />
              <FilterRecapItem
                label="Status"
                value={
                  appliedFilters.status
                    ? formatStatus(appliedFilters.status)
                    : "Any status"
                }
              />
              <FilterRecapItem
                label="Preset"
                value={appliedFilters.presetLabel ?? "Any preset"}
              />
              <FilterRecapItem
                label="Tracked app"
                value={
                  appliedFilters.trackedAppId
                    ? (trackedAppsQuery.data?.find(
                        (trackedApp) =>
                          trackedApp.id === appliedFilters.trackedAppId,
                      )?.name ?? `#${appliedFilters.trackedAppId}`)
                    : "Any app"
                }
              />
            </CardContent>
          </Card>
        </div>
      </div>

      <Dialog onOpenChange={setDetailOpen} open={detailOpen}>
        <DialogContent className="max-h-[88vh] w-[min(95vw,72rem)] overflow-y-auto">
          <DialogHeader>
            <DialogDescription>Session detail</DialogDescription>
            <DialogTitle className="ft-font-display text-3xl font-medium">
              {selectedDetail
                ? `${formatStatus(selectedDetail.session.status)} session on ${formatDate(selectedDetail.session.startedAt)}`
                : "Loading session"}
            </DialogTitle>
          </DialogHeader>

          {detailQuery.isLoading ? (
            <p className="ft-text-muted mt-5 text-sm">
              Loading session detail...
            </p>
          ) : null}

          {detailQuery.isError ? (
            <p className="mt-5 text-sm text-[var(--color-danger)]">
              {(detailQuery.error as Error).message}
            </p>
          ) : null}

          {selectedDetail ? (
            <Tabs className="mt-5" defaultValue="overview">
              <TabsList>
                <TabsTrigger value="overview">Overview</TabsTrigger>
                <TabsTrigger value="segments">Segments</TabsTrigger>
                <TabsTrigger value="activity">Activity</TabsTrigger>
              </TabsList>

              <TabsContent className="mt-5 space-y-5" value="overview">
                <div className="grid gap-3 md:grid-cols-4">
                  <MetricCard
                    label="Total duration"
                    value={formatDurationWords(
                      selectedDetail.totalDurationSeconds,
                    )}
                  />
                  <MetricCard
                    label="Focus"
                    value={formatDurationWords(
                      selectedDetail.session.actualFocusSeconds,
                    )}
                  />
                  <MetricCard
                    label="Break"
                    value={formatDurationWords(
                      selectedDetail.session.breakSeconds,
                    )}
                  />
                  <MetricCard
                    label="Interruptions"
                    value={`${selectedDetail.interruptionCount}`}
                  />
                </div>

                <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(22rem,0.9fr)]">
                  <Card className="p-5">
                    <CardDescription>Session summary</CardDescription>
                    <CardTitle className="mt-2 text-xl">
                      {selectedDetail.session.presetLabel ?? "Custom session"}
                    </CardTitle>
                    <div className="mt-4 grid gap-2 text-sm">
                      <SummaryLine
                        label="Started"
                        value={formatDateTime(selectedDetail.session.startedAt)}
                      />
                      <SummaryLine
                        label="Ended"
                        value={
                          selectedDetail.session.endedAt
                            ? formatDateTime(selectedDetail.session.endedAt)
                            : "Still open"
                        }
                      />
                      <SummaryLine
                        label="Planned"
                        value={`${selectedDetail.session.plannedFocusMinutes} min`}
                      />
                      <SummaryLine
                        label="Status"
                        value={formatStatus(selectedDetail.session.status)}
                      />
                    </div>

                    {selectedDetail.session.note ? (
                      <div className="ft-panel-muted mt-4 rounded-[1rem] px-4 py-3 text-sm">
                        {selectedDetail.session.note}
                      </div>
                    ) : (
                      <p className="ft-text-muted mt-4 text-sm">
                        No note saved for this session.
                      </p>
                    )}
                  </Card>

                  <Card className="p-5">
                    <CardDescription>Apps used</CardDescription>
                    <CardTitle className="mt-2 text-xl">
                      Which apps filled this session
                    </CardTitle>
                    <div className="mt-4 space-y-3">
                      {selectedDetail.trackedApps.length ? (
                        selectedDetail.trackedApps.map((trackedApp) => (
                          <div
                            className="ft-panel-muted flex items-center justify-between gap-4 rounded-[1rem] px-4 py-3"
                            key={trackedApp.trackedAppId}
                          >
                            <div className="flex items-center gap-3">
                              <span
                                className="h-3 w-3 rounded-full"
                                style={{
                                  backgroundColor:
                                    trackedApp.colorHex ?? "var(--color-brand)",
                                }}
                              />
                              <div>
                                <p className="text-sm font-medium">
                                  {trackedApp.name}
                                </p>
                                <p className="ft-text-muted text-xs">
                                  {categoryLabels[trackedApp.category]} ·{" "}
                                  {trackedApp.executable}
                                </p>
                              </div>
                            </div>
                            <span className="text-sm font-medium">
                              {formatDurationWords(trackedApp.durationSeconds)}
                            </span>
                          </div>
                        ))
                      ) : (
                        <p className="ft-text-muted text-sm">
                          No tracked apps were attached to this session.
                        </p>
                      )}
                    </div>
                  </Card>
                </div>

                {editDraft ? (
                  <Card className="p-5">
                    <CardDescription>Edit session</CardDescription>
                    <CardTitle className="mt-2 text-xl">
                      Correct timings, preset or note.
                    </CardTitle>

                    <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                      <Field label="Started at">
                        <input
                          className={inputClassName}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? { ...current, startedAt: event.target.value }
                                : current,
                            )
                          }
                          type="datetime-local"
                          value={editDraft.startedAt}
                        />
                      </Field>

                      <Field label="Ended at">
                        <input
                          className={inputClassName}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? { ...current, endedAt: event.target.value }
                                : current,
                            )
                          }
                          type="datetime-local"
                          value={editDraft.endedAt}
                        />
                      </Field>

                      <Field label="Planned (min)">
                        <input
                          className={inputClassName}
                          min={0}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? {
                                    ...current,
                                    plannedFocusMinutes: event.target.value,
                                  }
                                : current,
                            )
                          }
                          type="number"
                          value={editDraft.plannedFocusMinutes}
                        />
                      </Field>

                      <Field label="Status">
                        <select
                          className={inputClassName}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? {
                                    ...current,
                                    status: event.target.value as SessionStatus,
                                  }
                                : current,
                            )
                          }
                          value={editDraft.status}
                        >
                          {statusOptions.map((option) => (
                            <option key={option.value} value={option.value}>
                              {option.label}
                            </option>
                          ))}
                        </select>
                      </Field>

                      <Field label="Focus (min)">
                        <input
                          className={inputClassName}
                          min={0}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? {
                                    ...current,
                                    actualFocusMinutes: event.target.value,
                                  }
                                : current,
                            )
                          }
                          type="number"
                          value={editDraft.actualFocusMinutes}
                        />
                      </Field>

                      <Field label="Break (min)">
                        <input
                          className={inputClassName}
                          min={0}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? {
                                    ...current,
                                    breakMinutes: event.target.value,
                                  }
                                : current,
                            )
                          }
                          type="number"
                          value={editDraft.breakMinutes}
                        />
                      </Field>

                      <Field className="md:col-span-2" label="Preset">
                        <input
                          className={inputClassName}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? {
                                    ...current,
                                    presetLabel: event.target.value,
                                  }
                                : current,
                            )
                          }
                          placeholder="Classic"
                          value={editDraft.presetLabel}
                        />
                      </Field>

                      <Field
                        className="md:col-span-2 xl:col-span-4"
                        label="Note"
                      >
                        <textarea
                          className={`${inputClassName} min-h-28 resize-y py-3`}
                          onChange={(event) =>
                            setEditDraft((current) =>
                              current
                                ? { ...current, note: event.target.value }
                                : current,
                            )
                          }
                          placeholder="Optional note"
                          value={editDraft.note}
                        />
                      </Field>
                    </div>

                    <div className="mt-5 flex flex-wrap gap-3">
                      <Button
                        disabled={replaceSessionMutation.isPending}
                        onClick={() => {
                          void saveEdit();
                        }}
                      >
                        Save changes
                      </Button>
                      <Button
                        className="text-[var(--color-danger)]"
                        disabled={deleteSessionMutation.isPending}
                        onClick={() => {
                          void requestDelete();
                        }}
                        variant="ghost"
                      >
                        Delete session
                      </Button>
                    </div>
                  </Card>
                ) : null}
              </TabsContent>

              <TabsContent className="mt-5 space-y-3" value="segments">
                {selectedDetail.segments.length ? (
                  selectedDetail.segments.map((segmentDetail) => (
                    <SegmentRow
                      key={segmentDetail.segment.id}
                      segmentDetail={segmentDetail}
                    />
                  ))
                ) : (
                  <p className="ft-text-muted text-sm">
                    No segments were stored for this session.
                  </p>
                )}
              </TabsContent>

              <TabsContent className="mt-5 space-y-3" value="activity">
                {selectedDetail.trackedWindowEvents.length ? (
                  selectedDetail.trackedWindowEvents.map((event) => (
                    <TrackedEventRow event={event} key={event.id} />
                  ))
                ) : (
                  <p className="ft-text-muted text-sm">
                    No tracked window activity was stored for this session.
                  </p>
                )}
              </TabsContent>
            </Tabs>
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="ft-panel-muted rounded-[1rem] px-4 py-3">
      <p className="ft-text-muted text-sm">{label}</p>
      <p className="mt-2 text-sm font-medium">{value}</p>
    </div>
  );
}

function Field({
  children,
  className,
  label,
}: {
  children: ReactNode;
  className?: string;
  label: string;
}) {
  return (
    <label className={cn("grid gap-2", className)}>
      <span className="ft-text-muted text-xs font-medium uppercase tracking-[0.2em]">
        {label}
      </span>
      {children}
    </label>
  );
}

function SessionRow({
  item,
  onOpen,
}: {
  item: HistorySessionSummary;
  onOpen: () => void;
}) {
  return (
    <button
      className="ft-panel-muted w-full rounded-[1.25rem] px-4 py-4 text-left transition-transform hover:-translate-y-[1px]"
      onClick={onOpen}
      type="button"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge status={item.session.status} />
            <span className="ft-text-muted text-xs">
              {formatDateTime(item.session.startedAt)}
            </span>
          </div>
          <p className="text-base font-medium">
            {item.session.presetLabel ?? "Custom session"}
          </p>
          <p className="ft-text-muted text-sm">
            {item.session.note
              ? shorten(item.session.note, 88)
              : "No note saved."}
          </p>
        </div>

        <div className="grid gap-1 text-right text-sm">
          <span className="font-medium">
            {formatDurationWords(item.totalDurationSeconds)}
          </span>
          <span className="ft-text-muted">
            Focus {formatDurationWords(item.session.actualFocusSeconds)}
          </span>
          <span className="ft-text-muted">
            Interruptions {item.interruptionCount}
          </span>
        </div>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        {item.trackedApps.length ? (
          item.trackedApps.slice(0, 4).map((trackedApp) => (
            <span
              className="rounded-full border border-[var(--color-border)] px-3 py-1 text-xs"
              key={trackedApp.trackedAppId}
            >
              {trackedApp.name} ·{" "}
              {formatDurationCompact(trackedApp.durationSeconds)}
            </span>
          ))
        ) : (
          <span className="ft-text-muted text-xs">No tracked app context</span>
        )}
      </div>
    </button>
  );
}

function EmptyState() {
  return (
    <div className="ft-panel-muted rounded-[1.25rem] px-5 py-8 text-center">
      <p className="text-base font-medium">
        No session matches the current filters.
      </p>
      <p className="ft-text-muted mt-2 text-sm">
        Try widening the date range, clearing the app filter, or reviewing a
        different preset.
      </p>
    </div>
  );
}

function FilterRecapItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="ft-panel-muted rounded-[1rem] px-4 py-3">
      <p className="ft-text-muted text-xs uppercase tracking-[0.18em]">
        {label}
      </p>
      <p className="mt-2 text-sm">{value}</p>
    </div>
  );
}

function SummaryLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="ft-text-muted">{label}</span>
      <span className="text-right">{value}</span>
    </div>
  );
}

function SegmentRow({
  segmentDetail,
}: {
  segmentDetail: SessionSegmentDetail;
}) {
  return (
    <div className="ft-panel-muted rounded-[1rem] px-4 py-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-sm font-medium">
            {formatSegmentKind(segmentDetail.segment.kind)}
          </p>
          <p className="ft-text-muted mt-1 text-xs">
            {formatDateTime(segmentDetail.segment.startedAt)}
            {" -> "}
            {formatTime(segmentDetail.segment.endedAt)}
          </p>
        </div>
        <span className="text-sm font-medium">
          {formatDurationWords(segmentDetail.segment.durationSeconds)}
        </span>
      </div>

      <div className="mt-3 grid gap-2 text-sm">
        <SummaryLine
          label="App"
          value={segmentDetail.trackedApp?.name ?? "No tracked app"}
        />
        <SummaryLine
          label="Window"
          value={segmentDetail.segment.windowTitle ?? "No title"}
        />
      </div>
    </div>
  );
}

function TrackedEventRow({ event }: { event: TrackedWindowEvent }) {
  return (
    <div className="ft-panel-muted rounded-[1rem] px-4 py-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-sm font-medium">
            {event.appName ?? "Unknown app"}
          </p>
          <p className="ft-text-muted mt-1 text-xs">
            {event.windowTitle ?? "No title"}
          </p>
        </div>
        <span className="text-sm font-medium">
          {formatRange(event.startedAt, event.endedAt)}
        </span>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: SessionStatus }) {
  return (
    <span
      className={cn(
        "rounded-full px-3 py-1 text-xs font-medium",
        status === "completed" && "bg-emerald-500/15 text-emerald-200",
        status === "in_progress" && "bg-sky-500/15 text-sky-200",
        status === "planned" && "bg-amber-500/15 text-amber-100",
        status === "cancelled" && "bg-rose-500/15 text-rose-200",
      )}
    >
      {formatStatus(status)}
    </span>
  );
}

function buildSummaryMetrics(items: HistorySessionSummary[]) {
  return items.reduce(
    (summary, item) => ({
      focusSeconds: summary.focusSeconds + item.session.actualFocusSeconds,
      interruptionCount: summary.interruptionCount + item.interruptionCount,
    }),
    { focusSeconds: 0, interruptionCount: 0 },
  );
}

function buildEditDraft(detail: HistorySessionDetail): EditDraft {
  return {
    startedAt: toDateTimeLocalValue(detail.session.startedAt),
    endedAt: detail.session.endedAt
      ? toDateTimeLocalValue(detail.session.endedAt)
      : "",
    plannedFocusMinutes: String(detail.session.plannedFocusMinutes),
    actualFocusMinutes: String(
      Math.round(detail.session.actualFocusSeconds / 60),
    ),
    breakMinutes: String(Math.round(detail.session.breakSeconds / 60)),
    status: detail.session.status,
    presetLabel: detail.session.presetLabel ?? "",
    note: detail.session.note ?? "",
  };
}

function normalizeFilters(draft: FilterDraft): SessionHistoryFilters {
  const filters: SessionHistoryFilters = {};

  if (draft.dateFrom) {
    filters.dateFrom = draft.dateFrom;
  }
  if (draft.dateTo) {
    filters.dateTo = draft.dateTo;
  }
  if (draft.minDurationMinutes) {
    filters.minDurationSeconds = parseMinutesToSeconds(
      draft.minDurationMinutes,
    );
  }
  if (draft.maxDurationMinutes) {
    filters.maxDurationSeconds = parseMinutesToSeconds(
      draft.maxDurationMinutes,
    );
  }
  if (draft.presetLabel.trim()) {
    filters.presetLabel = draft.presetLabel.trim();
  }
  if (draft.status) {
    filters.status = draft.status;
  }
  if (draft.trackedAppId) {
    filters.trackedAppId = Number(draft.trackedAppId);
  }

  return filters;
}

function hasFilters(filters: SessionHistoryFilters) {
  return Object.values(filters).some(
    (value) => value !== undefined && value !== null,
  );
}

function parseMinutesToSeconds(value: string) {
  return parseWholeNumber(value) * 60;
}

function parseWholeNumber(value: string) {
  const numeric = Number(value);

  if (!Number.isFinite(numeric) || numeric < 0) {
    return 0;
  }

  return Math.round(numeric);
}

function toIsoString(value: string) {
  return new Date(value).toISOString();
}

function toDateTimeLocalValue(value: string) {
  const date = new Date(value);
  const formatter = new Intl.DateTimeFormat("sv-SE", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });

  return formatter.format(date).replace(" ", "T");
}

function formatStatus(status: SessionStatus) {
  return status.replace("_", " ");
}

function formatSegmentKind(kind: SessionSegmentKind) {
  if (kind === "idle") {
    return "Idle interruption";
  }

  return kind.charAt(0).toUpperCase() + kind.slice(1);
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
  }).format(new Date(value));
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    timeStyle: "short",
  }).format(new Date(value));
}

function formatRange(startedAt: string, endedAt: string | null) {
  return `${formatTime(startedAt)} -> ${formatTime(endedAt ?? startedAt)}`;
}

function formatDurationWords(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes.toString().padStart(2, "0")}m`;
  }

  return `${minutes}m`;
}

function formatDurationCompact(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h${minutes.toString().padStart(2, "0")}`;
  }

  return `${minutes}m`;
}

function formatOptionalMinutes(totalSeconds?: number | null) {
  if (!totalSeconds) {
    return "Any";
  }

  return `${Math.round(totalSeconds / 60)} min`;
}

function shorten(value: string, maxLength: number) {
  if (value.length <= maxLength) {
    return value;
  }

  return `${value.slice(0, maxLength - 1)}…`;
}
