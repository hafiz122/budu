use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatEntry {
    /// Steam App ID.
    pub app_id: String,
    /// Human-readable game name.
    pub name: String,
    /// Compatibility rating.
    pub rating: CompatRating,
    /// Last tested Wine version.
    pub last_tested_wine: Option<String>,
    /// When this entry was last updated.
    pub last_tested_date: Option<String>,
    /// Recommended graphics settings.
    pub graphics: Option<CompatGraphics>,
    /// Known fixes required.
    #[serde(default)]
    pub fixes: Vec<CompatFix>,
    /// Known issues.
    #[serde(default)]
    pub known_issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatRating {
    /// Runs perfectly out of the box.
    Platinum,
    /// Runs well with minor tweaks.
    Gold,
    /// Runs with noticeable issues.
    Silver,
    /// Barely playable, major issues.
    Bronze,
    /// Does not work at all.
    Borked,
    /// Not yet tested.
    Unknown,
}

impl CompatRating {
    pub fn emoji(&self) -> &str {
        match self {
            Self::Platinum => "🟦",
            Self::Gold => "🟨",
            Self::Silver => "⬜",
            Self::Bronze => "🟫",
            Self::Borked => "🟥",
            Self::Unknown => "⬛",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatGraphics {
    pub backend: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CompatFix {
    #[serde(rename = "dll_override")]
    DllOverride { dll: String, mode: String },
    #[serde(rename = "env")]
    Env { key: String, value: String },
    #[serde(rename = "launch_option")]
    LaunchOption { value: String },
    #[serde(rename = "registry")]
    Registry { key: String, value: String },
    #[serde(rename = "winetricks")]
    Winetricks { verb: String },
}

/// A community-submitted compatibility report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatReport {
    pub app_id: String,
    pub app_name: String,
    pub rating: CompatRating,
    pub wine_version: String,
    pub graphics_backend: String,
    pub mac_model: String,
    pub macos_version: String,
    pub notes: String,
    pub submitted_at: String,
}

pub struct CompatEngine {
    /// In-memory cache of the compatibility database.
    entries: Vec<CompatEntry>,
}

impl CompatEngine {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load the compatibility database from JSON files on disk.
    pub fn load(&mut self, db_dir: &std::path::Path) -> Result<usize, String> {
        let entries_dir = db_dir.join("entries");
        if !entries_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        for entry in std::fs::read_dir(&entries_dir)
            .map_err(|e| format!("Failed to read compat-db: {e}"))?
            .flatten()
        {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(compat_entry) = serde_json::from_str::<CompatEntry>(&contents) {
                        self.entries.push(compat_entry);
                        count += 1;
                    }
                }
            }
        }

        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(count)
    }

    /// Look up a compatibility entry by Steam App ID.
    pub fn lookup(&self, app_id: &str) -> Option<&CompatEntry> {
        self.entries.iter().find(|e| e.app_id == app_id)
    }

    /// Search entries by name (case-insensitive substring match).
    pub fn search(&self, query: &str) -> Vec<&CompatEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&q))
            .collect()
    }

    /// Return all entries, optionally filtered by rating.
    pub fn list(&self, min_rating: Option<CompatRating>) -> Vec<&CompatEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(min) = min_rating {
                    rating_value(&e.rating) >= rating_value(&min)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Count entries in the database.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CompatEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn rating_value(r: &CompatRating) -> u8 {
    match r {
        CompatRating::Platinum => 5,
        CompatRating::Gold => 4,
        CompatRating::Silver => 3,
        CompatRating::Bronze => 2,
        CompatRating::Borked => 1,
        CompatRating::Unknown => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_not_found() {
        let engine = CompatEngine::new();
        assert!(engine.lookup("123").is_none());
    }

    #[test]
    fn test_search() {
        let mut engine = CompatEngine::new();
        engine.entries.push(CompatEntry {
            app_id: "111".into(),
            name: "Elden Ring".into(),
            rating: CompatRating::Gold,
            last_tested_wine: Some("9.14-staging".into()),
            last_tested_date: Some("2026-07-15".into()),
            graphics: None,
            fixes: vec![],
            known_issues: vec![],
        });
        engine.entries.push(CompatEntry {
            app_id: "222".into(),
            name: "Hades II".into(),
            rating: CompatRating::Platinum,
            last_tested_wine: Some("9.14-staging".into()),
            last_tested_date: Some("2026-07-15".into()),
            graphics: None,
            fixes: vec![],
            known_issues: vec![],
        });

        assert_eq!(engine.search("elden").len(), 1);
        assert_eq!(engine.search("hades").len(), 1);
        assert_eq!(engine.search("nonexistent").len(), 0);
    }

    #[test]
    fn test_rating_filter() {
        let mut engine = CompatEngine::new();
        engine.entries.push(CompatEntry {
            app_id: "111".into(),
            name: "Gold Game".into(),
            rating: CompatRating::Gold,
            last_tested_wine: None,
            last_tested_date: None,
            graphics: None,
            fixes: vec![],
            known_issues: vec![],
        });
        engine.entries.push(CompatEntry {
            app_id: "222".into(),
            name: "Borked Game".into(),
            rating: CompatRating::Borked,
            last_tested_wine: None,
            last_tested_date: None,
            graphics: None,
            fixes: vec![],
            known_issues: vec![],
        });

        // Filter to Gold and above.
        let filtered = engine.list(Some(CompatRating::Gold));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Gold Game");
    }
}
