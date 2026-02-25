/// Platform-aware clipboard reader.
///
/// Tries arboard first, then falls back to CLI tools:
/// - Wayland: `wl-paste`
/// - X11:     `xclip` or `xsel`
/// - macOS:   `pbpaste`
pub fn get_text(clipboard: &mut Option<arboard::Clipboard>) -> Result<String, String> {
    // 1. Try arboard
    if let Some(cb) = clipboard {
        match cb.get_text() {
            Ok(text) if !text.is_empty() => return Ok(text),
            Ok(_) => {} // empty – try other methods
            Err(_) => {} // failed – try other methods
        }
    }

    // 2. Wayland: wl-paste
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(text) = run_cmd("wl-paste", &["--no-newline"]) {
            if !text.is_empty() {
                return Ok(text);
            }
        }
    }

    // 3. X11: xclip
    if std::env::var("DISPLAY").is_ok() {
        if let Ok(text) = run_cmd("xclip", &["-selection", "clipboard", "-out"]) {
            if !text.is_empty() {
                return Ok(text);
            }
        }
        // xsel fallback
        if let Ok(text) = run_cmd("xsel", &["--clipboard", "--output"]) {
            if !text.is_empty() {
                return Ok(text);
            }
        }
    }

    // 4. macOS: pbpaste
    #[cfg(target_os = "macos")]
    if let Ok(text) = run_cmd("pbpaste", &[]) {
        if !text.is_empty() {
            return Ok(text);
        }
    }

    Err("Could not read clipboard (arboard failed and no CLI tool available)".to_string())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        String::from_utf8(out.stdout).map_err(|e| e.to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}
