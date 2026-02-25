pub mod color;
pub mod filepath;
pub mod hex;
pub mod uuid;

/// A single interpreted field to display.
pub struct InterpretItem {
    pub label: String,
    pub value: String,
    /// Optional RGBA color for a swatch preview (used by ColorInterpreter).
    pub color: Option<[u8; 4]>,
}

impl InterpretItem {
    pub fn text(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
        }
    }

    pub fn with_color(label: impl Into<String>, value: impl Into<String>, rgba: [u8; 4]) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: Some(rgba),
        }
    }
}

/// The result of one interpreter run, containing a list of display items.
pub struct InterpretResult {
    pub items: Vec<InterpretItem>,
}

impl InterpretResult {
    pub fn new(items: Vec<InterpretItem>) -> Self {
        Self { items }
    }
}

/// Trait for clipboard content interpreters.
/// Returns `None` if the interpreter does not apply to the given content.
///
/// # Adding a new interpreter
/// 1. Create `src/interpreter/myformat.rs` and implement this trait.
/// 2. Add `pub mod myformat;` above.
/// 3. Append `Box::new(myformat::MyFormatInterpreter)` to `get_interpreters()`.
pub trait Interpreter: Send + Sync {
    fn name(&self) -> &str;
    fn interpret(&self, content: &str) -> Option<InterpretResult>;
}

/// Returns the ordered list of all active interpreters.
pub fn get_interpreters() -> Vec<Box<dyn Interpreter>> {
    vec![
        Box::new(hex::HexInterpreter),
        Box::new(uuid::UuidInterpreter),
        Box::new(color::ColorInterpreter),
        Box::new(filepath::FilePathInterpreter),
    ]
}
