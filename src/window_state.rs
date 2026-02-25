use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self { x: 100.0, y: 100.0, width: 900.0, height: 600.0 }
    }
}

/// Returns the path where window state is persisted.
/// Linux/others: $XDG_DATA_HOME/clipboard-hack/window_state.json
/// macOS:        ~/Library/Application Support/clipboard-hack/window_state.json
pub fn window_state_file_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("clipboard-hack")
            .join("window_state.json")
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".local").join("share")
            });
        base.join("clipboard-hack").join("window_state.json")
    }
}

/// Load window state from a JSON file. Returns WindowState::default() on any error.
pub fn load(path: &Path) -> WindowState {
    let Ok(json) = std::fs::read_to_string(path) else {
        return WindowState::default();
    };
    let Ok(state) = serde_json::from_str::<WindowState>(&json) else {
        eprintln!("[window_state] Failed to parse {}", path.display());
        return WindowState::default();
    };
    state
}

/// Persist window state to a JSON file, creating parent directories as needed.
pub fn save(state: &WindowState, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}
