ALTER TABLE user_preferences
ADD COLUMN weekly_focus_goal_minutes INTEGER NOT NULL DEFAULT 240
CHECK(weekly_focus_goal_minutes > 0);

ALTER TABLE user_preferences
ADD COLUMN weekly_completed_sessions_goal INTEGER NOT NULL DEFAULT 5
CHECK(weekly_completed_sessions_goal > 0);

CREATE INDEX IF NOT EXISTS idx_achievements_slug
ON achievements(slug);
