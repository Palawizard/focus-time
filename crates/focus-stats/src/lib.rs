use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{Datelike, Duration, NaiveDate, Timelike, Weekday};
use focus_domain::{
    Session, SessionSegment, SessionSegmentKind, SessionStatus, TrackedApp, TrackingCategory,
};
use serde::{Deserialize, Serialize};

pub const fn crate_name() -> &'static str {
    "focus-stats"
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatsPeriod {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatsRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub comparison_start_date: NaiveDate,
    pub comparison_end_date: NaiveDate,
    pub is_partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummary {
    pub focus_seconds: i64,
    pub break_seconds: i64,
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub interrupted_sessions: usize,
    pub active_days: usize,
    pub completion_rate: f64,
    pub average_focus_seconds_per_active_day: i64,
    pub streak_days: usize,
    pub best_streak_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsComparison {
    pub focus_seconds_delta: i64,
    pub focus_seconds_ratio: Option<f64>,
    pub completion_rate_delta: f64,
    pub completed_sessions_delta: i64,
    pub interrupted_sessions_delta: i64,
    pub active_days_delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsSeriesBucket {
    pub key: String,
    pub label: String,
    pub short_label: String,
    pub focus_seconds: i64,
    pub break_seconds: i64,
    pub completed_sessions: usize,
    pub interrupted_sessions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsWeekdayBucket {
    pub weekday: String,
    pub label: String,
    pub focus_seconds: i64,
    pub share_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatsAppDistribution {
    pub tracked_app_id: Option<i64>,
    pub name: String,
    pub executable: Option<String>,
    pub category: Option<TrackingCategory>,
    pub color_hex: Option<String>,
    pub focus_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsDashboard {
    pub period: StatsPeriod,
    pub range: StatsRange,
    pub summary: StatsSummary,
    pub comparison: StatsComparison,
    pub top_app: Option<StatsAppDistribution>,
    pub app_distribution: Vec<StatsAppDistribution>,
    pub focus_series: Vec<StatsSeriesBucket>,
    pub weekday_distribution: Vec<StatsWeekdayBucket>,
    pub is_empty: bool,
}

#[derive(Debug, Clone)]
pub struct BuildDashboardInput {
    pub period: StatsPeriod,
    pub anchor_date: NaiveDate,
    pub today: NaiveDate,
    pub sessions: Vec<Session>,
    pub current_segments: Vec<SessionSegment>,
    pub tracked_apps: Vec<TrackedApp>,
}

pub fn resolve_range(period: StatsPeriod, anchor_date: NaiveDate, today: NaiveDate) -> StatsRange {
    let (period_start, nominal_end) = match period {
        StatsPeriod::Day => (anchor_date, anchor_date),
        StatsPeriod::Week => {
            let offset = i64::from(anchor_date.weekday().num_days_from_monday());
            let start = anchor_date - Duration::days(offset);
            (start, start + Duration::days(6))
        }
        StatsPeriod::Month => {
            let start = anchor_date.with_day(1).expect("month start should exist");
            let end = month_end(anchor_date.year(), anchor_date.month());
            (start, end)
        }
    };

    let effective_end = nominal_end.min(anchor_date).min(today);
    let span_days = (effective_end - period_start).num_days();
    let comparison_end = period_start - Duration::days(1);
    let comparison_start = comparison_end - Duration::days(span_days);

    StatsRange {
        start_date: period_start,
        end_date: effective_end,
        comparison_start_date: comparison_start,
        comparison_end_date: comparison_end,
        is_partial: effective_end < nominal_end,
    }
}

pub fn build_dashboard(input: BuildDashboardInput) -> StatsDashboard {
    let range = resolve_range(input.period, input.anchor_date, input.today);
    let current_sessions = filter_sessions(&input.sessions, range.start_date, range.end_date);
    let previous_sessions = filter_sessions(
        &input.sessions,
        range.comparison_start_date,
        range.comparison_end_date,
    );

    let summary = summarize_sessions(&current_sessions, &input.sessions, range.end_date);
    let previous_summary = summarize_sessions(
        &previous_sessions,
        &input.sessions,
        range.comparison_end_date,
    );
    let app_distribution = build_app_distribution(&input.current_segments, &input.tracked_apps);
    let top_app = app_distribution
        .iter()
        .find(|app| app.tracked_app_id.is_some())
        .cloned();

    StatsDashboard {
        period: input.period,
        range: range.clone(),
        comparison: compare_summaries(&summary, &previous_summary),
        focus_series: build_focus_series(
            input.period,
            &current_sessions,
            &input.current_segments,
            range.start_date,
            range.end_date,
        ),
        weekday_distribution: build_weekday_distribution(&current_sessions),
        top_app,
        app_distribution,
        is_empty: summary.total_sessions == 0 && summary.focus_seconds == 0,
        summary,
    }
}

fn filter_sessions(sessions: &[Session], start: NaiveDate, end: NaiveDate) -> Vec<Session> {
    sessions
        .iter()
        .filter(|session| {
            let date = session.started_at.date_naive();
            date >= start && date <= end
        })
        .cloned()
        .collect()
}

fn summarize_sessions(
    current_sessions: &[Session],
    historical_sessions: &[Session],
    streak_end: NaiveDate,
) -> StatsSummary {
    let focus_seconds = current_sessions
        .iter()
        .map(|session| session.actual_focus_seconds.max(0))
        .sum();
    let break_seconds = current_sessions
        .iter()
        .map(|session| session.break_seconds.max(0))
        .sum();
    let completed_sessions = current_sessions
        .iter()
        .filter(|session| session.status == SessionStatus::Completed)
        .count();
    let interrupted_sessions = current_sessions
        .iter()
        .filter(|session| session.status == SessionStatus::Cancelled)
        .count();
    let active_days = current_sessions
        .iter()
        .filter(|session| session.actual_focus_seconds > 0)
        .map(|session| session.started_at.date_naive())
        .collect::<BTreeSet<_>>()
        .len();
    let denominator = completed_sessions + interrupted_sessions;
    let completion_rate = if denominator == 0 {
        0.0
    } else {
        completed_sessions as f64 / denominator as f64
    };
    let average_focus_seconds_per_active_day = if active_days == 0 {
        0
    } else {
        focus_seconds / active_days as i64
    };
    let streak_days = current_streak_days(historical_sessions, streak_end);
    let best_streak_days = best_streak_days(historical_sessions, streak_end);

    StatsSummary {
        focus_seconds,
        break_seconds,
        total_sessions: current_sessions.len(),
        completed_sessions,
        interrupted_sessions,
        active_days,
        completion_rate,
        average_focus_seconds_per_active_day,
        streak_days,
        best_streak_days,
    }
}

fn compare_summaries(current: &StatsSummary, previous: &StatsSummary) -> StatsComparison {
    StatsComparison {
        focus_seconds_delta: current.focus_seconds - previous.focus_seconds,
        focus_seconds_ratio: ratio_delta(previous.focus_seconds, current.focus_seconds),
        completion_rate_delta: current.completion_rate - previous.completion_rate,
        completed_sessions_delta: current.completed_sessions as i64
            - previous.completed_sessions as i64,
        interrupted_sessions_delta: current.interrupted_sessions as i64
            - previous.interrupted_sessions as i64,
        active_days_delta: current.active_days as i64 - previous.active_days as i64,
    }
}

fn ratio_delta(previous: i64, current: i64) -> Option<f64> {
    (previous != 0).then_some((current - previous) as f64 / previous as f64)
}

fn build_focus_series(
    period: StatsPeriod,
    current_sessions: &[Session],
    current_segments: &[SessionSegment],
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<StatsSeriesBucket> {
    match period {
        StatsPeriod::Day => build_hourly_series(current_sessions, current_segments, start),
        StatsPeriod::Week | StatsPeriod::Month => build_daily_series(current_sessions, start, end),
    }
}

fn build_hourly_series(
    current_sessions: &[Session],
    current_segments: &[SessionSegment],
    day: NaiveDate,
) -> Vec<StatsSeriesBucket> {
    let mut buckets = (0..24)
        .map(|hour| StatsSeriesBucket {
            key: format!("{hour:02}:00"),
            label: format!("{hour:02}:00"),
            short_label: format!("{hour:02}"),
            focus_seconds: 0,
            break_seconds: 0,
            completed_sessions: 0,
            interrupted_sessions: 0,
        })
        .collect::<Vec<_>>();

    for session in current_sessions {
        if session.started_at.date_naive() != day {
            continue;
        }

        let hour = session.started_at.hour() as usize;
        if let Some(bucket) = buckets.get_mut(hour) {
            match session.status {
                SessionStatus::Completed => bucket.completed_sessions += 1,
                SessionStatus::Cancelled => bucket.interrupted_sessions += 1,
                SessionStatus::Planned | SessionStatus::InProgress => {}
            }
        }
    }

    for segment in current_segments {
        distribute_segment_by_hour(segment, day, &mut buckets);
    }

    buckets
}

fn distribute_segment_by_hour(
    segment: &SessionSegment,
    day: NaiveDate,
    buckets: &mut [StatsSeriesBucket],
) {
    let day_start = day
        .and_hms_opt(0, 0, 0)
        .expect("valid day start should exist");
    let day_end = day_start + Duration::days(1);
    let start = segment.started_at.naive_utc().max(day_start);
    let end = segment.ended_at.naive_utc().min(day_end);

    if end <= start {
        return;
    }

    let mut cursor = start;
    while cursor < end {
        let hour_start = day
            .and_hms_opt(cursor.hour(), 0, 0)
            .expect("valid hour start should exist");
        let next_hour = hour_start + Duration::hours(1);
        let slice_end = end.min(next_hour);
        let seconds = (slice_end - cursor).num_seconds().max(0);

        if let Some(bucket) = buckets.get_mut(cursor.hour() as usize) {
            match segment.kind {
                SessionSegmentKind::Focus => bucket.focus_seconds += seconds,
                SessionSegmentKind::Break => bucket.break_seconds += seconds,
                SessionSegmentKind::Idle => {}
            }
        }

        cursor = slice_end;
    }
}

fn build_daily_series(
    current_sessions: &[Session],
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<StatsSeriesBucket> {
    let mut aggregates = BTreeMap::<NaiveDate, StatsSeriesBucket>::new();
    let mut cursor = start;
    while cursor <= end {
        aggregates.insert(
            cursor,
            StatsSeriesBucket {
                key: cursor.format("%Y-%m-%d").to_string(),
                label: weekday_short_label(cursor.weekday()).to_string(),
                short_label: cursor.format("%d").to_string(),
                focus_seconds: 0,
                break_seconds: 0,
                completed_sessions: 0,
                interrupted_sessions: 0,
            },
        );
        cursor += Duration::days(1);
    }

    for session in current_sessions {
        let date = session.started_at.date_naive();
        if let Some(bucket) = aggregates.get_mut(&date) {
            bucket.focus_seconds += session.actual_focus_seconds.max(0);
            bucket.break_seconds += session.break_seconds.max(0);

            match session.status {
                SessionStatus::Completed => bucket.completed_sessions += 1,
                SessionStatus::Cancelled => bucket.interrupted_sessions += 1,
                SessionStatus::Planned | SessionStatus::InProgress => {}
            }
        }
    }

    aggregates.into_values().collect()
}

fn build_weekday_distribution(current_sessions: &[Session]) -> Vec<StatsWeekdayBucket> {
    let mut totals = weekday_buckets();

    for session in current_sessions {
        let weekday = session.started_at.weekday();
        *totals.entry(weekday).or_default() += session.actual_focus_seconds.max(0);
    }

    let total_focus = totals.values().sum::<i64>();

    weekday_order()
        .into_iter()
        .map(|weekday| {
            let focus_seconds = totals.get(&weekday).copied().unwrap_or_default();
            StatsWeekdayBucket {
                weekday: weekday_key(weekday).to_string(),
                label: weekday_short_label(weekday).to_string(),
                focus_seconds,
                share_ratio: if total_focus == 0 {
                    0.0
                } else {
                    focus_seconds as f64 / total_focus as f64
                },
            }
        })
        .collect()
}

fn build_app_distribution(
    current_segments: &[SessionSegment],
    tracked_apps: &[TrackedApp],
) -> Vec<StatsAppDistribution> {
    let tracked_apps_by_id = tracked_apps
        .iter()
        .map(|tracked_app| (tracked_app.id, tracked_app))
        .collect::<HashMap<_, _>>();
    let mut usage_by_app = HashMap::<i64, i64>::new();

    for segment in current_segments {
        if segment.kind != SessionSegmentKind::Focus {
            continue;
        }

        let Some(tracked_app_id) = segment.tracked_app_id else {
            continue;
        };

        *usage_by_app.entry(tracked_app_id).or_default() += segment.duration_seconds.max(0);
    }

    let mut distribution = usage_by_app
        .into_iter()
        .filter_map(|(tracked_app_id, focus_seconds)| {
            let tracked_app = tracked_apps_by_id.get(&tracked_app_id)?;

            Some(StatsAppDistribution {
                tracked_app_id: Some(tracked_app_id),
                name: tracked_app.name.clone(),
                executable: Some(tracked_app.executable.clone()),
                category: Some(tracked_app.category),
                color_hex: tracked_app.color_hex.clone(),
                focus_seconds,
            })
        })
        .collect::<Vec<_>>();

    distribution.sort_by(|left, right| {
        right
            .focus_seconds
            .cmp(&left.focus_seconds)
            .then_with(|| left.name.cmp(&right.name))
    });

    if distribution.len() <= 5 {
        return distribution;
    }

    let remaining_focus = distribution[5..]
        .iter()
        .map(|app| app.focus_seconds)
        .sum::<i64>();
    distribution.truncate(5);

    if remaining_focus > 0 {
        distribution.push(StatsAppDistribution {
            tracked_app_id: None,
            name: "Other apps".to_string(),
            executable: None,
            category: None,
            color_hex: None,
            focus_seconds: remaining_focus,
        });
    }

    distribution
}

fn current_streak_days(sessions: &[Session], streak_end: NaiveDate) -> usize {
    let active_days = active_days_until(sessions, streak_end);
    let mut streak_days = 0usize;
    let mut cursor = streak_end;

    while active_days.contains(&cursor) {
        streak_days += 1;
        cursor -= Duration::days(1);
    }

    streak_days
}

fn best_streak_days(sessions: &[Session], streak_end: NaiveDate) -> usize {
    let active_days = active_days_until(sessions, streak_end);
    let mut best = 0usize;
    let mut current = 0usize;
    let mut previous_day: Option<NaiveDate> = None;

    for day in active_days {
        if previous_day == Some(day - Duration::days(1)) {
            current += 1;
        } else {
            current = 1;
        }

        best = best.max(current);
        previous_day = Some(day);
    }

    best
}

fn active_days_until(sessions: &[Session], streak_end: NaiveDate) -> BTreeSet<NaiveDate> {
    sessions
        .iter()
        .filter(|session| {
            session.actual_focus_seconds > 0 && session.started_at.date_naive() <= streak_end
        })
        .map(|session| session.started_at.date_naive())
        .collect()
}

fn weekday_buckets() -> HashMap<Weekday, i64> {
    weekday_order()
        .into_iter()
        .map(|weekday| (weekday, 0))
        .collect()
}

fn weekday_order() -> [Weekday; 7] {
    [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ]
}

fn weekday_short_label(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

fn weekday_key(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "monday",
        Weekday::Tue => "tuesday",
        Weekday::Wed => "wednesday",
        Weekday::Thu => "thursday",
        Weekday::Fri => "friday",
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
    }
}

fn month_end(year: i32, month: u32) -> NaiveDate {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("next january should exist")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("next month should exist")
    };

    next_month - Duration::days(1)
}

#[cfg(test)]
mod tests {
    use super::{
        build_dashboard, resolve_range, BuildDashboardInput, StatsPeriod, StatsSeriesBucket,
    };
    use chrono::{DateTime, NaiveDate, Utc};
    use focus_domain::{
        Session, SessionSegment, SessionSegmentKind, SessionStatus, TrackedApp, TrackingCategory,
    };

    #[test]
    fn exposes_current_crate_name() {
        assert_eq!(super::crate_name(), "focus-stats");
    }

    #[test]
    fn resolves_partial_week_range_against_previous_elapsed_period() {
        let range = resolve_range(
            StatsPeriod::Week,
            NaiveDate::from_ymd_opt(2026, 4, 1).expect("anchor should exist"),
            NaiveDate::from_ymd_opt(2026, 4, 1).expect("today should exist"),
        );

        assert_eq!(
            range.start_date,
            NaiveDate::from_ymd_opt(2026, 3, 30).expect("start should exist")
        );
        assert_eq!(
            range.end_date,
            NaiveDate::from_ymd_opt(2026, 4, 1).expect("end should exist")
        );
        assert_eq!(
            range.comparison_start_date,
            NaiveDate::from_ymd_opt(2026, 3, 27).expect("comparison start should exist")
        );
        assert_eq!(
            range.comparison_end_date,
            NaiveDate::from_ymd_opt(2026, 3, 29).expect("comparison end should exist")
        );
        assert!(range.is_partial);
    }

    #[test]
    fn builds_week_dashboard_with_comparisons_streak_and_top_apps() {
        let dashboard = build_dashboard(BuildDashboardInput {
            period: StatsPeriod::Week,
            anchor_date: NaiveDate::from_ymd_opt(2026, 4, 1).expect("anchor should exist"),
            today: NaiveDate::from_ymd_opt(2026, 4, 1).expect("today should exist"),
            sessions: vec![
                session(
                    1,
                    "2026-03-30T08:00:00Z",
                    1_800,
                    300,
                    SessionStatus::Completed,
                ),
                session(
                    2,
                    "2026-03-31T08:00:00Z",
                    2_100,
                    300,
                    SessionStatus::Completed,
                ),
                session(
                    3,
                    "2026-04-01T08:00:00Z",
                    900,
                    120,
                    SessionStatus::Cancelled,
                ),
                session(4, "2026-03-27T08:00:00Z", 900, 60, SessionStatus::Completed),
                session(5, "2026-03-28T08:00:00Z", 600, 60, SessionStatus::Cancelled),
                session(6, "2026-03-29T08:00:00Z", 600, 0, SessionStatus::Completed),
            ],
            current_segments: vec![
                segment(
                    1,
                    Some(10),
                    "2026-03-30T08:00:00Z",
                    "2026-03-30T08:30:00Z",
                    1_800,
                    SessionSegmentKind::Focus,
                ),
                segment(
                    2,
                    Some(10),
                    "2026-03-31T08:00:00Z",
                    "2026-03-31T08:20:00Z",
                    1_200,
                    SessionSegmentKind::Focus,
                ),
                segment(
                    2,
                    Some(11),
                    "2026-03-31T08:20:00Z",
                    "2026-03-31T08:35:00Z",
                    900,
                    SessionSegmentKind::Focus,
                ),
                segment(
                    3,
                    Some(11),
                    "2026-04-01T08:00:00Z",
                    "2026-04-01T08:15:00Z",
                    900,
                    SessionSegmentKind::Focus,
                ),
            ],
            tracked_apps: vec![
                tracked_app(10, "Code", "Code.exe", TrackingCategory::Development),
                tracked_app(11, "Arc", "Arc.exe", TrackingCategory::Browser),
            ],
        });

        assert_eq!(dashboard.summary.focus_seconds, 4_800);
        assert_eq!(dashboard.summary.completed_sessions, 2);
        assert_eq!(dashboard.summary.interrupted_sessions, 1);
        assert_eq!(dashboard.summary.streak_days, 6);
        assert_eq!(dashboard.summary.best_streak_days, 6);
        assert_eq!(dashboard.comparison.focus_seconds_delta, 2_700);
        assert_eq!(
            dashboard.top_app.as_ref().map(|app| app.name.as_str()),
            Some("Code")
        );
        assert_eq!(dashboard.focus_series.len(), 3);
        assert_eq!(dashboard.weekday_distribution.len(), 7);
    }

    #[test]
    fn builds_day_series_from_segments_split_across_hours() {
        let dashboard = build_dashboard(BuildDashboardInput {
            period: StatsPeriod::Day,
            anchor_date: NaiveDate::from_ymd_opt(2026, 4, 1).expect("anchor should exist"),
            today: NaiveDate::from_ymd_opt(2026, 4, 1).expect("today should exist"),
            sessions: vec![session(
                1,
                "2026-04-01T09:45:00Z",
                2_700,
                300,
                SessionStatus::Completed,
            )],
            current_segments: vec![
                segment(
                    1,
                    Some(10),
                    "2026-04-01T09:45:00Z",
                    "2026-04-01T10:30:00Z",
                    2_700,
                    SessionSegmentKind::Focus,
                ),
                segment(
                    1,
                    Some(10),
                    "2026-04-01T10:30:00Z",
                    "2026-04-01T10:35:00Z",
                    300,
                    SessionSegmentKind::Break,
                ),
            ],
            tracked_apps: vec![tracked_app(
                10,
                "Code",
                "Code.exe",
                TrackingCategory::Development,
            )],
        });

        let nine_am = bucket(&dashboard.focus_series, "09:00");
        let ten_am = bucket(&dashboard.focus_series, "10:00");

        assert_eq!(nine_am.focus_seconds, 900);
        assert_eq!(ten_am.focus_seconds, 1_800);
        assert_eq!(ten_am.break_seconds, 300);
        assert_eq!(ten_am.completed_sessions, 0);
        assert_eq!(nine_am.completed_sessions, 1);
    }

    fn session(
        id: i64,
        started_at: &str,
        actual_focus_seconds: i64,
        break_seconds: i64,
        status: SessionStatus,
    ) -> Session {
        let started_at = parse_datetime(started_at);

        Session {
            id,
            started_at,
            ended_at: Some(
                started_at + chrono::Duration::seconds(actual_focus_seconds + break_seconds),
            ),
            planned_focus_minutes: 25,
            actual_focus_seconds,
            break_seconds,
            status,
            preset_label: Some("Classic".to_string()),
            note: None,
            created_at: started_at,
            updated_at: started_at,
        }
    }

    fn segment(
        session_id: i64,
        tracked_app_id: Option<i64>,
        started_at: &str,
        ended_at: &str,
        duration_seconds: i64,
        kind: SessionSegmentKind,
    ) -> SessionSegment {
        SessionSegment {
            id: session_id,
            session_id,
            tracked_app_id,
            kind,
            window_title: None,
            started_at: parse_datetime(started_at),
            ended_at: parse_datetime(ended_at),
            duration_seconds,
            created_at: parse_datetime(started_at),
        }
    }

    fn tracked_app(
        id: i64,
        name: &str,
        executable: &str,
        category: TrackingCategory,
    ) -> TrackedApp {
        let timestamp = parse_datetime("2026-04-01T00:00:00Z");

        TrackedApp {
            id,
            name: name.to_string(),
            executable: executable.to_string(),
            category,
            color_hex: Some("#60b7ff".to_string()),
            is_excluded: false,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    fn parse_datetime(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("timestamp should parse")
            .with_timezone(&Utc)
    }

    fn bucket<'a>(series: &'a [StatsSeriesBucket], key: &str) -> &'a StatsSeriesBucket {
        series
            .iter()
            .find(|bucket| bucket.key == key)
            .expect("bucket should exist")
    }
}
