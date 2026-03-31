CREATE TABLE IF NOT EXISTS sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  planned_focus_minutes INTEGER NOT NULL CHECK(planned_focus_minutes > 0),
  actual_focus_seconds INTEGER NOT NULL DEFAULT 0 CHECK(actual_focus_seconds >= 0),
  break_seconds INTEGER NOT NULL DEFAULT 0 CHECK(break_seconds >= 0),
  status TEXT NOT NULL CHECK(status IN ('planned', 'in_progress', 'completed', 'cancelled')),
  preset_label TEXT,
  note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tracked_apps (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  executable TEXT NOT NULL UNIQUE,
  color_hex TEXT,
  is_excluded INTEGER NOT NULL DEFAULT 0 CHECK(is_excluded IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS session_segments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id INTEGER NOT NULL,
  tracked_app_id INTEGER,
  kind TEXT NOT NULL CHECK(kind IN ('focus', 'break', 'idle')),
  window_title TEXT,
  started_at TEXT NOT NULL,
  ended_at TEXT NOT NULL,
  duration_seconds INTEGER NOT NULL DEFAULT 0 CHECK(duration_seconds >= 0),
  created_at TEXT NOT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
  FOREIGN KEY(tracked_app_id) REFERENCES tracked_apps(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS tracked_window_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id INTEGER,
  tracked_app_id INTEGER,
  window_title TEXT,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE SET NULL,
  FOREIGN KEY(tracked_app_id) REFERENCES tracked_apps(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS daily_stats (
  stat_date TEXT PRIMARY KEY,
  focus_seconds INTEGER NOT NULL DEFAULT 0 CHECK(focus_seconds >= 0),
  break_seconds INTEGER NOT NULL DEFAULT 0 CHECK(break_seconds >= 0),
  completed_sessions INTEGER NOT NULL DEFAULT 0 CHECK(completed_sessions >= 0),
  interrupted_sessions INTEGER NOT NULL DEFAULT 0 CHECK(interrupted_sessions >= 0),
  top_app_id INTEGER,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(top_app_id) REFERENCES tracked_apps(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS achievements (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  slug TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  unlocked_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_preferences (
  id INTEGER PRIMARY KEY CHECK(id = 1),
  focus_minutes INTEGER NOT NULL CHECK(focus_minutes > 0),
  short_break_minutes INTEGER NOT NULL CHECK(short_break_minutes > 0),
  long_break_minutes INTEGER NOT NULL CHECK(long_break_minutes > 0),
  sessions_until_long_break INTEGER NOT NULL CHECK(sessions_until_long_break > 0),
  auto_start_breaks INTEGER NOT NULL DEFAULT 0 CHECK(auto_start_breaks IN (0, 1)),
  auto_start_focus INTEGER NOT NULL DEFAULT 0 CHECK(auto_start_focus IN (0, 1)),
  tracking_enabled INTEGER NOT NULL DEFAULT 1 CHECK(tracking_enabled IN (0, 1)),
  notifications_enabled INTEGER NOT NULL DEFAULT 1 CHECK(notifications_enabled IN (0, 1)),
  theme TEXT NOT NULL CHECK(theme IN ('system', 'light', 'dark')),
  updated_at TEXT NOT NULL
);
