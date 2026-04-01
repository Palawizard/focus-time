use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{Datelike, Duration, NaiveDate, Utc};

use crate::{
    Achievement, AchievementProgress, GamificationOverview, ProgressBadge, Session, SessionStatus,
    Streak, UserPreference, WeeklyGoalProgress,
};

const STREAK_MILESTONES: [usize; 6] = [3, 7, 14, 30, 60, 100];

#[derive(Debug, Clone)]
pub struct BuildGamificationOverviewInput {
    pub today: NaiveDate,
    pub sessions: Vec<Session>,
    pub preferences: UserPreference,
    pub unlocked_achievements: Vec<Achievement>,
}

pub fn build_gamification_overview(input: BuildGamificationOverviewInput) -> GamificationOverview {
    let streak = build_streak(&input.sessions, input.today);
    let weekly_goal = build_weekly_goal(&input.sessions, input.today, &input.preferences);
    let achievements = build_achievements(
        &input.sessions,
        &streak,
        &weekly_goal,
        input.unlocked_achievements,
    );
    let badges = build_badges(&streak, &weekly_goal);

    GamificationOverview {
        streak,
        weekly_goal,
        badges,
        achievements,
    }
}

fn build_streak(sessions: &[Session], today: NaiveDate) -> Streak {
    let active_days = sessions
        .iter()
        .filter(|session| session.actual_focus_seconds > 0)
        .map(|session| session.started_at.date_naive())
        .collect::<BTreeSet<_>>();
    let last_active_date = active_days.iter().next_back().copied();
    let today_completed = active_days.contains(&today);
    let streak_end = match last_active_date {
        Some(_) if today_completed => Some(today),
        Some(last_active_date) if last_active_date == today - Duration::days(1) => {
            Some(last_active_date)
        }
        _ => None,
    };
    let current_days = streak_end
        .map(|date| consecutive_days_until(&active_days, date))
        .unwrap_or_default();
    let best_days = best_streak_days(&active_days);
    let next_milestone_days = STREAK_MILESTONES
        .iter()
        .copied()
        .find(|milestone| *milestone > current_days)
        .unwrap_or_else(|| current_days.saturating_add(10).max(10));
    let is_at_risk =
        !today_completed && current_days > 0 && last_active_date == Some(today - Duration::days(1));

    Streak {
        current_days,
        best_days,
        today_completed,
        last_active_date,
        next_milestone_days,
        is_at_risk,
    }
}

fn build_weekly_goal(
    sessions: &[Session],
    today: NaiveDate,
    preferences: &UserPreference,
) -> WeeklyGoalProgress {
    let week_start = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    let week_end = week_start + Duration::days(6);
    let weekly_sessions = sessions
        .iter()
        .filter(|session| {
            let date = session.started_at.date_naive();
            date >= week_start && date <= week_end
        })
        .collect::<Vec<_>>();
    let focus_minutes_completed = weekly_sessions
        .iter()
        .map(|session| session.actual_focus_seconds.max(0))
        .sum::<i64>()
        / 60;
    let completed_sessions = weekly_sessions
        .iter()
        .filter(|session| session.status == SessionStatus::Completed)
        .count();
    let focus_completion_ratio = ratio_i64(
        focus_minutes_completed,
        i64::from(preferences.weekly_focus_goal_minutes),
    );
    let sessions_completion_ratio = ratio_i64(
        completed_sessions as i64,
        i64::from(preferences.weekly_completed_sessions_goal),
    );
    let completed_goal_count =
        usize::from(focus_completion_ratio >= 1.0) + usize::from(sessions_completion_ratio >= 1.0);
    let is_completed = completed_goal_count == 2;

    WeeklyGoalProgress {
        start_date: week_start,
        end_date: week_end,
        focus_goal_minutes: preferences.weekly_focus_goal_minutes,
        completed_sessions_goal: preferences.weekly_completed_sessions_goal,
        focus_minutes_completed,
        completed_sessions,
        focus_completion_ratio,
        sessions_completion_ratio,
        completed_goal_count,
        is_completed,
    }
}

fn build_badges(streak: &Streak, weekly_goal: &WeeklyGoalProgress) -> Vec<ProgressBadge> {
    vec![
        ProgressBadge {
            slug: "daily-streak".to_string(),
            title: "Daily streak".to_string(),
            description: "Keep one focused day after another.".to_string(),
            progress_label: format!(
                "{} / {} days",
                streak.current_days, streak.next_milestone_days
            ),
            progress_ratio: ratio_i64(
                streak.current_days as i64,
                streak.next_milestone_days as i64,
            ),
            is_unlocked: streak.current_days >= 3,
        },
        ProgressBadge {
            slug: "weekly-focus-goal".to_string(),
            title: "Weekly focus goal".to_string(),
            description: "Move your planned focus minutes forward this week.".to_string(),
            progress_label: format!(
                "{} / {} min",
                weekly_goal.focus_minutes_completed, weekly_goal.focus_goal_minutes
            ),
            progress_ratio: weekly_goal.focus_completion_ratio,
            is_unlocked: weekly_goal.focus_completion_ratio >= 1.0,
        },
        ProgressBadge {
            slug: "weekly-session-goal".to_string(),
            title: "Weekly session goal".to_string(),
            description: "Build a steady rhythm of completed sessions.".to_string(),
            progress_label: format!(
                "{} / {} sessions",
                weekly_goal.completed_sessions, weekly_goal.completed_sessions_goal
            ),
            progress_ratio: weekly_goal.sessions_completion_ratio,
            is_unlocked: weekly_goal.sessions_completion_ratio >= 1.0,
        },
    ]
}

fn build_achievements(
    sessions: &[Session],
    streak: &Streak,
    weekly_goal: &WeeklyGoalProgress,
    unlocked_achievements: Vec<Achievement>,
) -> Vec<AchievementProgress> {
    let unlocked_by_slug = unlocked_achievements
        .into_iter()
        .map(|achievement| (achievement.slug, achievement.unlocked_at))
        .collect::<HashMap<_, _>>();
    let completed_sessions = sessions
        .iter()
        .filter(|session| session.status == SessionStatus::Completed)
        .count() as i64;
    let daily_focus_by_date = sessions
        .iter()
        .filter(|session| session.actual_focus_seconds > 0)
        .fold(BTreeMap::<NaiveDate, i64>::new(), |mut buckets, session| {
            *buckets.entry(session.started_at.date_naive()).or_default() +=
                session.actual_focus_seconds.max(0);
            buckets
        });
    let max_daily_focus_seconds = daily_focus_by_date.into_values().max().unwrap_or_default();
    let weekly_goals_completed = weekly_goal.completed_goal_count as i64;

    achievement_definitions()
        .into_iter()
        .map(|definition| {
            let progress_current = match definition.metric {
                AchievementMetric::CompletedSessions => completed_sessions,
                AchievementMetric::BestStreakDays => streak.best_days as i64,
                AchievementMetric::MaxDailyFocusSeconds => max_daily_focus_seconds,
                AchievementMetric::WeeklyGoalsCompleted => weekly_goals_completed,
            };
            let unlocked_at = unlocked_by_slug
                .get(definition.slug)
                .copied()
                .flatten()
                .or_else(|| (progress_current >= definition.target).then_some(Utc::now()));

            AchievementProgress {
                slug: definition.slug.to_string(),
                title: definition.title.to_string(),
                description: definition.description.to_string(),
                progress_current: progress_current.min(definition.target),
                progress_target: definition.target,
                progress_ratio: ratio_i64(progress_current, definition.target),
                unlocked_at,
            }
        })
        .collect()
}

fn achievement_definitions() -> [AchievementDefinition; 6] {
    [
        AchievementDefinition {
            slug: "first-session",
            title: "First focused block",
            description: "Complete your first focus session.",
            target: 1,
            metric: AchievementMetric::CompletedSessions,
        },
        AchievementDefinition {
            slug: "five-sessions",
            title: "Steady cadence",
            description: "Complete five focus sessions.",
            target: 5,
            metric: AchievementMetric::CompletedSessions,
        },
        AchievementDefinition {
            slug: "three-day-streak",
            title: "Three-day run",
            description: "Reach a three-day focus streak.",
            target: 3,
            metric: AchievementMetric::BestStreakDays,
        },
        AchievementDefinition {
            slug: "seven-day-streak",
            title: "Full-week streak",
            description: "Reach a seven-day focus streak.",
            target: 7,
            metric: AchievementMetric::BestStreakDays,
        },
        AchievementDefinition {
            slug: "deep-work-day",
            title: "Deep work day",
            description: "Accumulate two hours of focus in a single day.",
            target: 7_200,
            metric: AchievementMetric::MaxDailyFocusSeconds,
        },
        AchievementDefinition {
            slug: "two-goals-week",
            title: "On target",
            description: "Complete both weekly goals in the same week.",
            target: 2,
            metric: AchievementMetric::WeeklyGoalsCompleted,
        },
    ]
}

fn consecutive_days_until(active_days: &BTreeSet<NaiveDate>, end_date: NaiveDate) -> usize {
    let mut cursor = end_date;
    let mut streak_days = 0usize;

    while active_days.contains(&cursor) {
        streak_days += 1;
        cursor -= Duration::days(1);
    }

    streak_days
}

fn best_streak_days(active_days: &BTreeSet<NaiveDate>) -> usize {
    let mut best = 0usize;
    let mut current = 0usize;
    let mut previous = None;

    for date in active_days {
        if previous == Some(*date - Duration::days(1)) {
            current += 1;
        } else {
            current = 1;
        }

        best = best.max(current);
        previous = Some(*date);
    }

    best
}

fn ratio_i64(current: i64, target: i64) -> f64 {
    if target <= 0 {
        return 0.0;
    }

    (current.max(0) as f64 / target as f64).min(1.0)
}

#[derive(Debug, Clone, Copy)]
enum AchievementMetric {
    CompletedSessions,
    BestStreakDays,
    MaxDailyFocusSeconds,
    WeeklyGoalsCompleted,
}

#[derive(Debug, Clone, Copy)]
struct AchievementDefinition {
    slug: &'static str,
    title: &'static str,
    description: &'static str,
    target: i64,
    metric: AchievementMetric,
}

#[cfg(test)]
mod tests {
    use super::{build_gamification_overview, BuildGamificationOverviewInput};
    use crate::{Achievement, Session, SessionStatus, ThemePreference, UserPreference};
    use chrono::{DateTime, NaiveDate, Utc};

    #[test]
    fn builds_gamification_progress_and_unlocks() {
        let overview = build_gamification_overview(BuildGamificationOverviewInput {
            today: NaiveDate::from_ymd_opt(2026, 4, 1).expect("today should exist"),
            sessions: vec![
                session(1, "2026-03-29T08:00:00Z", 1_500, SessionStatus::Completed),
                session(2, "2026-03-30T08:00:00Z", 1_800, SessionStatus::Completed),
                session(3, "2026-03-31T08:00:00Z", 2_400, SessionStatus::Completed),
                session(4, "2026-04-01T08:00:00Z", 3_000, SessionStatus::Completed),
            ],
            preferences: UserPreference {
                weekly_focus_goal_minutes: 120,
                weekly_completed_sessions_goal: 3,
                focus_minutes: 25,
                short_break_minutes: 5,
                long_break_minutes: 15,
                sessions_until_long_break: 4,
                auto_start_breaks: false,
                auto_start_focus: false,
                tracking_enabled: true,
                tracking_permission_granted: true,
                tracking_onboarding_completed: true,
                notifications_enabled: true,
                sound_enabled: false,
                launch_on_startup: false,
                tray_enabled: true,
                close_to_tray: true,
                theme: ThemePreference::System,
                updated_at: Utc::now(),
            },
            unlocked_achievements: vec![Achievement {
                id: 1,
                slug: "first-session".to_string(),
                title: "First focused block".to_string(),
                unlocked_at: Some(Utc::now()),
                created_at: Utc::now(),
            }],
        });

        assert_eq!(overview.streak.current_days, 4);
        assert!(overview.weekly_goal.is_completed);
        assert_eq!(overview.weekly_goal.completed_goal_count, 2);
        assert_eq!(overview.badges.len(), 3);
        assert!(overview
            .achievements
            .iter()
            .find(|achievement| achievement.slug == "first-session")
            .and_then(|achievement| achievement.unlocked_at)
            .is_some());
        assert_eq!(
            overview
                .achievements
                .iter()
                .find(|achievement| achievement.slug == "three-day-streak")
                .map(|achievement| achievement.progress_current),
            Some(3)
        );
    }

    #[test]
    fn keeps_a_streak_alive_until_the_previous_day() {
        let overview = build_gamification_overview(BuildGamificationOverviewInput {
            today: NaiveDate::from_ymd_opt(2026, 4, 2).expect("today should exist"),
            sessions: vec![
                session(1, "2026-03-31T08:00:00Z", 1_800, SessionStatus::Completed),
                session(2, "2026-04-01T08:00:00Z", 1_800, SessionStatus::Completed),
            ],
            preferences: UserPreference::default(),
            unlocked_achievements: Vec::new(),
        });

        assert_eq!(overview.streak.current_days, 2);
        assert!(overview.streak.is_at_risk);
        assert!(!overview.streak.today_completed);
    }

    fn session(
        id: i64,
        started_at: &str,
        actual_focus_seconds: i64,
        status: SessionStatus,
    ) -> Session {
        let started_at = DateTime::parse_from_rfc3339(started_at)
            .expect("timestamp should parse")
            .with_timezone(&Utc);

        Session {
            id,
            started_at,
            ended_at: Some(started_at + chrono::Duration::seconds(actual_focus_seconds)),
            planned_focus_minutes: 25,
            actual_focus_seconds,
            break_seconds: 0,
            status,
            preset_label: Some("Classic".to_string()),
            note: None,
            created_at: started_at,
            updated_at: started_at,
        }
    }
}
