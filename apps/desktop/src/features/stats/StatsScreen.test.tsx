import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { StatsScreen } from "./StatsScreen";

const getStatsDashboard = vi.fn();

vi.mock("../../lib/storage", () => ({
  getStatsDashboard: (...args: unknown[]) => getStatsDashboard(...args),
}));

describe("StatsScreen", () => {
  it("renders the dashboard metrics and app distribution", async () => {
    getStatsDashboard.mockResolvedValue({
      period: "week",
      range: {
        startDate: "2026-03-30",
        endDate: "2026-04-01",
        comparisonStartDate: "2026-03-27",
        comparisonEndDate: "2026-03-29",
        isPartial: true,
      },
      summary: {
        focusSeconds: 11_700,
        breakSeconds: 1_200,
        totalSessions: 4,
        completedSessions: 3,
        interruptedSessions: 1,
        activeDays: 3,
        completionRate: 0.75,
        averageFocusSecondsPerActiveDay: 3_900,
        streakDays: 4,
        bestStreakDays: 6,
      },
      comparison: {
        focusSecondsDelta: 3_600,
        focusSecondsRatio: 0.44,
        completionRateDelta: 0.15,
        completedSessionsDelta: 1,
        interruptedSessionsDelta: 0,
        activeDaysDelta: 1,
      },
      topApp: {
        trackedAppId: 1,
        name: "Code",
        executable: "Code.exe",
        category: "development",
        colorHex: "#0078d4",
        focusSeconds: 6_600,
      },
      appDistribution: [
        {
          trackedAppId: 1,
          name: "Code",
          executable: "Code.exe",
          category: "development",
          colorHex: "#0078d4",
          focusSeconds: 6_600,
        },
        {
          trackedAppId: 2,
          name: "Arc",
          executable: "Arc.exe",
          category: "browser",
          colorHex: "#60b7ff",
          focusSeconds: 5_100,
        },
      ],
      focusSeries: [
        {
          key: "2026-03-30",
          label: "Mon",
          shortLabel: "30",
          focusSeconds: 3_600,
          breakSeconds: 300,
          completedSessions: 1,
          interruptedSessions: 0,
        },
        {
          key: "2026-03-31",
          label: "Tue",
          shortLabel: "31",
          focusSeconds: 4_200,
          breakSeconds: 600,
          completedSessions: 1,
          interruptedSessions: 0,
        },
        {
          key: "2026-04-01",
          label: "Wed",
          shortLabel: "01",
          focusSeconds: 3_900,
          breakSeconds: 300,
          completedSessions: 1,
          interruptedSessions: 1,
        },
      ],
      weekdayDistribution: [
        {
          weekday: "monday",
          label: "Mon",
          focusSeconds: 3_600,
          shareRatio: 0.31,
        },
        {
          weekday: "tuesday",
          label: "Tue",
          focusSeconds: 4_200,
          shareRatio: 0.36,
        },
        {
          weekday: "wednesday",
          label: "Wed",
          focusSeconds: 3_900,
          shareRatio: 0.33,
        },
        { weekday: "thursday", label: "Thu", focusSeconds: 0, shareRatio: 0 },
        { weekday: "friday", label: "Fri", focusSeconds: 0, shareRatio: 0 },
        { weekday: "saturday", label: "Sat", focusSeconds: 0, shareRatio: 0 },
        { weekday: "sunday", label: "Sun", focusSeconds: 0, shareRatio: 0 },
      ],
      isEmpty: false,
    });

    render(
      <QueryClientProvider client={new QueryClient()}>
        <StatsScreen />
      </QueryClientProvider>,
    );

    expect(
      await screen.findByRole("heading", { level: 2, name: "3h 15m" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Your focus trend in the selected range."),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Code")).toHaveLength(2);
    expect(screen.getByText("75%")).toBeInTheDocument();
  });
});
