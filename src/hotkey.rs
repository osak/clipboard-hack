use rdev::{listen, EventType, Key};
use std::sync::mpsc::Sender;

// Change these constants to customize the hotkey.
// Current binding: Ctrl + Shift + H
const HOTKEY_CTRL: bool = true;
const HOTKEY_SHIFT: bool = true;
const HOTKEY_ALT: bool = false;
const HOTKEY_KEY: Key = Key::KeyH;

/// Spawns a background thread that listens for the global hotkey.
/// Sends a `()` message on `tx` whenever the hotkey is pressed.
pub fn start_hotkey_listener(tx: Sender<()>) {
    std::thread::spawn(move || {
        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;

        let callback = move |event: rdev::Event| {
            match event.event_type {
                EventType::KeyPress(k) => {
                    update_modifier(k, true, &mut ctrl, &mut shift, &mut alt);
                    if is_hotkey(k, ctrl, shift, alt) {
                        let _ = tx.send(());
                    }
                }
                EventType::KeyRelease(k) => {
                    update_modifier(k, false, &mut ctrl, &mut shift, &mut alt);
                }
                _ => {}
            }
        };

        if let Err(e) = listen(callback) {
            eprintln!("[hotkey] Listen error: {e:?}");
        }
    });
}

fn update_modifier(key: Key, pressed: bool, ctrl: &mut bool, shift: &mut bool, alt: &mut bool) {
    match key {
        Key::ControlLeft | Key::ControlRight => *ctrl = pressed,
        Key::ShiftLeft | Key::ShiftRight => *shift = pressed,
        Key::Alt | Key::AltGr => *alt = pressed,
        _ => {}
    }
}

fn is_hotkey(key: Key, ctrl: bool, shift: bool, alt: bool) -> bool {
    key == HOTKEY_KEY
        && ctrl == HOTKEY_CTRL
        && shift == HOTKEY_SHIFT
        && alt == HOTKEY_ALT
}

/// Human-readable description of the configured hotkey.
pub fn hotkey_display() -> &'static str {
    "Ctrl+Shift+H"
}
