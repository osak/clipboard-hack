use super::{InterpretItem, InterpretResult, Interpreter};

pub struct HexInterpreter;

impl Interpreter for HexInterpreter {
    fn name(&self) -> &str {
        "Hex Dump"
    }

    fn interpret(&self, content: &str) -> Option<InterpretResult> {
        let bytes = content.as_bytes();
        let byte_count = bytes.len();
        let char_count = content.chars().count();

        // Build hex string in groups of 8 bytes per line
        let hex_lines: Vec<String> = bytes
            .chunks(16)
            .enumerate()
            .map(|(i, chunk)| {
                let offset = format!("{:04x}", i * 16);
                let hex: String = chunk
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let ascii: String = chunk
                    .iter()
                    .map(|&b| {
                        if (0x20..0x7f).contains(&b) {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();
                format!("{offset}  {hex:<47}  {ascii}")
            })
            .collect();

        let hex_display = hex_lines.join("\n");

        // Also provide a plain compact hex string (useful for short content)
        let compact_hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

        Some(InterpretResult::new(vec![
            InterpretItem::text("Bytes", format!("{byte_count}")),
            InterpretItem::text("Chars (UTF-8)", format!("{char_count}")),
            InterpretItem::text("Compact hex", compact_hex),
            InterpretItem::text("Hex dump", hex_display),
        ]))
    }
}
