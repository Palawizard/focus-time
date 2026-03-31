use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct TableDefinition {
    pub name: &'static str,
    pub purpose: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DatabaseSchema {
    pub version: u32,
    pub tables: Vec<TableDefinition>,
}

pub fn initial_schema() -> DatabaseSchema {
    DatabaseSchema {
        version: 3,
        tables: vec![
            TableDefinition {
                name: "sessions",
                purpose: "Stores focus sessions and their lifecycle.",
            },
            TableDefinition {
                name: "session_segments",
                purpose: "Stores app-level segments captured inside a session.",
            },
            TableDefinition {
                name: "tracked_apps",
                purpose: "Stores known applications and exclusion flags.",
            },
            TableDefinition {
                name: "tracked_window_events",
                purpose: "Stores raw window activity events for tracking adapters.",
            },
            TableDefinition {
                name: "tracking_exclusion_rules",
                purpose: "Stores explicit user-managed exclusion rules for tracking.",
            },
            TableDefinition {
                name: "daily_stats",
                purpose: "Stores daily aggregates for quick dashboard reads.",
            },
            TableDefinition {
                name: "achievements",
                purpose: "Stores unlocked achievements and badge metadata.",
            },
            TableDefinition {
                name: "user_preferences",
                purpose: "Stores the current local user preferences snapshot.",
            },
        ],
    }
}
