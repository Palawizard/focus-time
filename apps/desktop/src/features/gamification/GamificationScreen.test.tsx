import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { GamificationScreen } from "./GamificationScreen";

const getGamificationOverview = vi.fn();
const getUserPreferences = vi.fn();
const saveUserPreferences = vi.fn();

vi.mock("../../lib/storage", () => ({
  getGamificationOverview: (...args: unknown[]) =>
    getGamificationOverview(...args),
  getUserPreferences: (...args: unknown[]) => getUserPreferences(...args),
  saveUserPreferences: (...args: unknown[]) => saveUserPreferences(...args),
}));

describe("GamificationScreen", () => {
  it("renders streak, weekly goals and achievements", async () => {
    getUserPreferences.mockResolvedValue({
      focusMinutes: 25,
      shortBreakMinutes: 5,
      longBreakMinutes: 15,
      sessionsUntilLongBreak: 4,
      autoStartBreaks: false,
      autoStartFocus: false,
      trackingEnabled: true,
      trackingPermissionGranted: true,
      trackingOnboardingCompleted: true,
      notificationsEnabled: true,
      weeklyFocusGoalMinutes: 240,
      weeklyCompletedSessionsGoal: 5,
      theme: "system",
      updatedAt: "2026-04-01T00:00:00Z",
    });
    getGamificationOverview.mockResolvedValue({
      streak: {
        currentDays: 4,
        bestDays: 7,
        todayCompleted: true,
        lastActiveDate: "2026-04-01",
        nextMilestoneDays: 7,
        isAtRisk: false,
      },
      weeklyGoal: {
        startDate: "2026-03-30",
        endDate: "2026-04-05",
        focusGoalMinutes: 240,
        completedSessionsGoal: 5,
        focusMinutesCompleted: 195,
        completedSessions: 4,
        focusCompletionRatio: 0.81,
        sessionsCompletionRatio: 0.8,
        completedGoalCount: 0,
        isCompleted: false,
      },
      badges: [
        {
          slug: "daily-streak",
          title: "Daily streak",
          description: "Keep one focused day after another.",
          progressLabel: "4 / 7 days",
          progressRatio: 0.57,
          isUnlocked: true,
        },
      ],
      achievements: [
        {
          slug: "first-session",
          title: "First focused block",
          description: "Complete your first focus session.",
          progressCurrent: 1,
          progressTarget: 1,
          progressRatio: 1,
          unlockedAt: "2026-04-01T09:00:00Z",
        },
      ],
    });

    render(
      <QueryClientProvider client={new QueryClient()}>
        <GamificationScreen />
      </QueryClientProvider>,
    );

    expect(
      await screen.findByRole("heading", {
        level: 3,
        name: "Set targets that fit your real week.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByDisplayValue("240")).toBeInTheDocument();
    expect(screen.getByText("4 / 7 days")).toBeInTheDocument();
    expect(screen.getByText("Unlocked on Apr 1")).toBeInTheDocument();
  });
});
