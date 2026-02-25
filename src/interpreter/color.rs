use super::{InterpretItem, InterpretResult, Interpreter};

pub struct ColorInterpreter;

impl Interpreter for ColorInterpreter {
    fn name(&self) -> &str {
        "Color Code"
    }

    fn interpret(&self, content: &str) -> Option<InterpretResult> {
        let trimmed = content.trim();
        parse_color(trimmed).map(|(r, g, b, a)| build_result(r, g, b, a))
    }
}

/// Parse color string into (r, g, b, a) with u8 components.
fn parse_color(s: &str) -> Option<(u8, u8, u8, u8)> {
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    let lower = s.to_lowercase();
    if lower.starts_with("rgb(") && lower.ends_with(')') {
        return parse_rgb_fn(&lower[4..lower.len() - 1], false);
    }
    if lower.starts_with("rgba(") && lower.ends_with(')') {
        return parse_rgb_fn(&lower[5..lower.len() - 1], true);
    }
    None
}

fn parse_hex(hex: &str) -> Option<(u8, u8, u8, u8)> {
    match hex.len() {
        3 => {
            // #RGB → #RRGGBB
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b, 255))
        }
        4 => {
            // #RGBA
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
            Some((r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

fn parse_rgb_fn(inner: &str, with_alpha: bool) -> Option<(u8, u8, u8, u8)> {
    let parts: Vec<&str> = inner.split(',').collect();
    if with_alpha && parts.len() == 4 {
        let r = parse_channel(parts[0])?;
        let g = parse_channel(parts[1])?;
        let b = parse_channel(parts[2])?;
        let a_f: f32 = parts[3].trim().parse().ok()?;
        let a = (a_f.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some((r, g, b, a))
    } else if !with_alpha && parts.len() == 3 {
        let r = parse_channel(parts[0])?;
        let g = parse_channel(parts[1])?;
        let b = parse_channel(parts[2])?;
        Some((r, g, b, 255))
    } else {
        None
    }
}

fn parse_channel(s: &str) -> Option<u8> {
    s.trim().parse::<u8>().ok()
}

fn build_result(r: u8, g: u8, b: u8, a: u8) -> InterpretResult {
    let hex6 = format!("#{:02x}{:02x}{:02x}", r, g, b);
    let hex8 = format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a);
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let alpha_pct = format!("{:.1}%", a as f32 / 255.0 * 100.0);

    InterpretResult::new(vec![
        InterpretItem::with_color("Preview", &hex6, [r, g, b, a]),
        InterpretItem::text("Hex (RGB)", hex6),
        InterpretItem::text("Hex (RGBA)", hex8),
        InterpretItem::text("R", r.to_string()),
        InterpretItem::text("G", g.to_string()),
        InterpretItem::text("B", b.to_string()),
        InterpretItem::text("A", format!("{} ({})", a, alpha_pct)),
        InterpretItem::text("HSL", format!("hsl({:.0}°, {:.1}%, {:.1}%)", h, s * 100.0, l * 100.0)),
    ])
}

/// Convert RGB (0–255) to HSL (H: 0–360, S: 0–1, L: 0–1).
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

    let h = if (max - r).abs() < 1e-6 {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}
