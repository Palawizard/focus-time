CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_session_segments_session_id ON session_segments(session_id);
CREATE INDEX IF NOT EXISTS idx_session_segments_tracked_app_id ON session_segments(tracked_app_id);
CREATE INDEX IF NOT EXISTS idx_tracked_window_events_session_id ON tracked_window_events(session_id);
CREATE INDEX IF NOT EXISTS idx_tracked_window_events_started_at ON tracked_window_events(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_tracked_apps_name ON tracked_apps(name);
CREATE INDEX IF NOT EXISTS idx_daily_stats_updated_at ON daily_stats(updated_at DESC);

INSERT INTO user_preferences (
  id,
  focus_minutes,
  short_break_minutes,
  long_break_minutes,
  sessions_until_long_break,
  auto_start_breaks,
  auto_start_focus,
  tracking_enabled,
  notifications_enabled,
  theme,
  updated_at
)
SELECT
  1,
  25,
  5,
  15,
  4,
  0,
  0,
  1,
  1,
  'system',
  '1970-01-01T00:00:00Z'
WHERE NOT EXISTS (
  SELECT 1
  FROM user_preferences
  WHERE id = 1
);
