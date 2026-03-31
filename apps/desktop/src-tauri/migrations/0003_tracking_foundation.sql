ALTER TABLE tracked_apps
ADD COLUMN category TEXT NOT NULL DEFAULT 'unknown'
CHECK(category IN (
  'development',
  'browser',
  'communication',
  'writing',
  'design',
  'meeting',
  'research',
  'utilities',
  'unknown'
));

ALTER TABLE user_preferences
ADD COLUMN tracking_permission_granted INTEGER NOT NULL DEFAULT 0
CHECK(tracking_permission_granted IN (0, 1));

ALTER TABLE user_preferences
ADD COLUMN tracking_onboarding_completed INTEGER NOT NULL DEFAULT 0
CHECK(tracking_onboarding_completed IN (0, 1));

CREATE TABLE IF NOT EXISTS tracking_exclusion_rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  kind TEXT NOT NULL CHECK(kind IN ('executable', 'window_title', 'category')),
  pattern TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(kind, pattern)
);

CREATE INDEX IF NOT EXISTS idx_tracking_exclusion_rules_kind
ON tracking_exclusion_rules(kind);

UPDATE tracked_apps
SET category = CASE
  WHEN lower(executable) LIKE '%code%'
    OR lower(executable) LIKE '%idea%'
    OR lower(executable) LIKE '%studio%'
    OR lower(executable) LIKE '%nvim%'
    OR lower(executable) LIKE '%vim%'
    OR lower(executable) LIKE '%zed%'
  THEN 'development'
  WHEN lower(executable) LIKE '%chrome%'
    OR lower(executable) LIKE '%firefox%'
    OR lower(executable) LIKE '%brave%'
    OR lower(executable) LIKE '%arc%'
    OR lower(executable) LIKE '%browser%'
  THEN 'browser'
  WHEN lower(executable) LIKE '%slack%'
    OR lower(executable) LIKE '%discord%'
    OR lower(executable) LIKE '%teams%'
    OR lower(executable) LIKE '%telegram%'
  THEN 'communication'
  WHEN lower(executable) LIKE '%word%'
    OR lower(executable) LIKE '%writer%'
    OR lower(executable) LIKE '%notion%'
    OR lower(executable) LIKE '%obsidian%'
  THEN 'writing'
  WHEN lower(executable) LIKE '%figma%'
    OR lower(executable) LIKE '%sketch%'
    OR lower(executable) LIKE '%xd%'
  THEN 'design'
  WHEN lower(executable) LIKE '%zoom%'
    OR lower(executable) LIKE '%meet%'
  THEN 'meeting'
  ELSE 'unknown'
END
WHERE category = 'unknown';
