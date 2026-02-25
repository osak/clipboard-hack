use super::{InterpretItem, InterpretResult, Interpreter};
use uuid::Uuid;

pub struct UuidInterpreter;

impl Interpreter for UuidInterpreter {
    fn name(&self) -> &str {
        "UUID"
    }

    fn interpret(&self, content: &str) -> Option<InterpretResult> {
        let trimmed = content.trim();
        let u = Uuid::parse_str(trimmed).ok()?;

        let version = match u.get_version() {
            Some(v) => format!("{:?}", v),
            None => "Unknown".to_string(),
        };
        let variant = format!("{:?}", u.get_variant());

        let mut items = vec![
            InterpretItem::text("Version", version),
            InterpretItem::text("Variant", variant),
            InterpretItem::text("Hyphenated", u.hyphenated().to_string()),
            InterpretItem::text("Simple (no hyphens)", u.simple().to_string()),
            InterpretItem::text("URN", u.urn().to_string()),
            InterpretItem::text("Braced", u.braced().to_string()),
        ];

        // For v1 UUIDs, show the timestamp
        if let Some(ts) = u.get_timestamp() {
            let (secs, nanos) = ts.to_unix();
            items.push(InterpretItem::text(
                "Timestamp (Unix)",
                format!("{}.{:09}", secs, nanos),
            ));
        }

        Some(InterpretResult::new(items))
    }
}
