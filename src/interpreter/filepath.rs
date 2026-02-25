use std::path::Path;

use super::{InterpretItem, InterpretResult, Interpreter};

pub struct FilePathInterpreter;

impl Interpreter for FilePathInterpreter {
    fn name(&self) -> &str {
        "File Path"
    }

    fn interpret(&self, content: &str) -> Option<InterpretResult> {
        let trimmed = content.trim();

        // Only consider absolute paths or tilde-expanded paths
        if !trimmed.starts_with('/') && !trimmed.starts_with('~') {
            return None;
        }

        let expanded: String = if trimmed.starts_with('~') {
            let home = std::env::var("HOME").unwrap_or_default();
            trimmed.replacen('~', &home, 1)
        } else {
            trimmed.to_string()
        };

        let path = Path::new(&expanded);
        let exists = path.exists();
        let is_symlink = path.is_symlink();

        let kind = if is_symlink {
            "Symlink".to_string()
        } else if path.is_file() {
            "File".to_string()
        } else if path.is_dir() {
            "Directory".to_string()
        } else if exists {
            "Other".to_string()
        } else {
            "—".to_string()
        };

        let mut items = vec![
            InterpretItem::text("Exists", exists.to_string()),
            InterpretItem::text("Type", kind),
        ];

        if exists {
            if let Some(parent) = path.parent() {
                items.push(InterpretItem::text("Parent", parent.to_string_lossy()));
            }
            if let Some(name) = path.file_name() {
                items.push(InterpretItem::text("Filename", name.to_string_lossy()));
            }
            if let Some(stem) = path.file_stem() {
                items.push(InterpretItem::text("Stem", stem.to_string_lossy()));
            }
            if let Some(ext) = path.extension() {
                items.push(InterpretItem::text("Extension", ext.to_string_lossy()));
            }
            if path.is_file() {
                match std::fs::metadata(&expanded) {
                    Ok(meta) => {
                        items.push(InterpretItem::text("Size", format_size(meta.len())));
                    }
                    Err(e) => {
                        items.push(InterpretItem::text("Size", format!("(error: {e})")));
                    }
                }
            }
            if is_symlink {
                if let Ok(target) = std::fs::read_link(&expanded) {
                    items.push(InterpretItem::text("Symlink target", target.to_string_lossy()));
                }
            }
        }

        Some(InterpretResult::new(items))
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        b if b >= GB => format!("{:.2} GB ({b} bytes)", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB ({b} bytes)", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB ({b} bytes)", b as f64 / KB as f64),
        b => format!("{b} bytes"),
    }
}
