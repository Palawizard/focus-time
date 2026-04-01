ALTER TABLE user_preferences
ADD COLUMN sound_enabled INTEGER NOT NULL DEFAULT 0
CHECK(sound_enabled IN (0, 1));

ALTER TABLE user_preferences
ADD COLUMN launch_on_startup INTEGER NOT NULL DEFAULT 0
CHECK(launch_on_startup IN (0, 1));

ALTER TABLE user_preferences
ADD COLUMN tray_enabled INTEGER NOT NULL DEFAULT 1
CHECK(tray_enabled IN (0, 1));

ALTER TABLE user_preferences
ADD COLUMN close_to_tray INTEGER NOT NULL DEFAULT 1
CHECK(close_to_tray IN (0, 1));
