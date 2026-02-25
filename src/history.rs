use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, SystemTime};

use chrono::{Local, TimeZone as _};
use serde::{Deserialize, Serialize};

pub struct ClipboardEntry {
    content: String,
    captured_at: SystemTime,
}

impl ClipboardEntry {
    pub fn new(content: String) -> Self {
        Self {
            content,
            captured_at: SystemTime::now(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns a truncated preview for display in the history list.
    pub fn preview(&self, max_chars: usize) -> String {
        let trimmed = self.content.trim();
        let single_line: String = trimmed
            .chars()
            .map(|c| if c == '\n' || c == '\r' || c == '\t' { ' ' } else { c })
            .collect();
        if single_line.chars().count() > max_chars {
            format!("{}…", single_line.chars().take(max_chars).collect::<String>())
        } else {
            single_line
        }
    }

    /// Formatted timestamp string in the system local timezone.
    pub fn timestamp_str(&self) -> String {
        let unix_secs = self
            .captured_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Local
            .timestamp_opt(unix_secs, 0)
            .single()
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "??:??:??".to_string())
    }
}

// ── Serialization helpers ─────────────────────────────────────────────────────

/// JSON-friendly representation of a single history entry.
#[derive(Serialize, Deserialize)]
struct StoredEntry {
    content: String,
    unix_secs: u64,
}

impl From<&ClipboardEntry> for StoredEntry {
    fn from(e: &ClipboardEntry) -> Self {
        let unix_secs = e
            .captured_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        StoredEntry { content: e.content.clone(), unix_secs }
    }
}

impl From<StoredEntry> for ClipboardEntry {
    fn from(s: StoredEntry) -> Self {
        ClipboardEntry {
            content: s.content,
            captured_at: SystemTime::UNIX_EPOCH + Duration::from_secs(s.unix_secs),
        }
    }
}

// ── ClipboardHistory ──────────────────────────────────────────────────────────

pub struct ClipboardHistory {
    entries: VecDeque<ClipboardEntry>,
    max_size: usize,
}

impl ClipboardHistory {
    pub fn new(max_size: usize) -> Self {
        Self { entries: VecDeque::new(), max_size }
    }

    /// Load history from a JSON file. Returns an empty history on any error.
    pub fn load(path: &Path, max_size: usize) -> Self {
        let mut history = Self::new(max_size);
        let Ok(json) = std::fs::read_to_string(path) else {
            return history;
        };
        let Ok(stored) = serde_json::from_str::<Vec<StoredEntry>>(&json) else {
            eprintln!("[history] Failed to parse {}", path.display());
            return history;
        };
        // File is stored newest-first; rebuild the deque in the same order.
        for entry in stored.into_iter().take(max_size) {
            history.entries.push_back(ClipboardEntry::from(entry));
        }
        history
    }

    /// Persist the history to a JSON file, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let stored: Vec<StoredEntry> = self.entries.iter().map(StoredEntry::from).collect();
        let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Add a new entry (deduplicates against the most recent). Returns true if added.
    pub fn add(&mut self, content: String) -> bool {
        if let Some(front) = self.entries.front() {
            if front.content() == content {
                return false;
            }
        }
        if self.entries.len() >= self.max_size {
            self.entries.pop_back();
        }
        self.entries.push_front(ClipboardEntry::new(content));
        true
    }

    pub fn entries(&self) -> &VecDeque<ClipboardEntry> {
        &self.entries
    }

    pub fn get(&self, index: usize) -> Option<&ClipboardEntry> {
        self.entries.get(index)
    }

    pub fn remove(&mut self, index: usize) {
        self.entries.remove(index);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
