#![warn(clippy::pedantic)]

const INV_255: f32 = 1.0 / 255.0;

#[inline]
const fn expand_nibble(n: u8) -> u8 { (n << 4) | n }

const fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// parses hex color strings: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
/// returns (rgb, alpha) with 8-bit channels; alpha is None if not provided.
pub fn parse_hex_rgba_u8(input: &str) -> Option<([u8; 3], Option<u8>)> {
    let s = input.strip_prefix('#')?;
    let b = s.as_bytes();
    let byte = |h: usize, l: usize| -> Option<u8> {
        Some((hex_nibble(*b.get(h)?)? << 4) | hex_nibble(*b.get(l)?)?)
    };
    match b.len() {
        3 => Some(([expand_nibble(hex_nibble(b[0])?), expand_nibble(hex_nibble(b[1])?), expand_nibble(hex_nibble(b[2])?)], None)),
        4 => Some(([
            expand_nibble(hex_nibble(b[0])?),
            expand_nibble(hex_nibble(b[1])?),
            expand_nibble(hex_nibble(b[2])?),
        ], Some(expand_nibble(hex_nibble(b[3])?)))),
        6 => Some(([byte(0, 1)?, byte(2, 3)?, byte(4, 5)?], None)),
        8 => Some(([byte(0, 1)?, byte(2, 3)?, byte(4, 5)?], Some(byte(6, 7)?))),
        _ => None,
    }
}

/// Parses hex and returns normalized floats (0..1) rgba.
pub fn parse_hex_rgba_f32(input: &str) -> Option<(f32, f32, f32, f32)> {
    let (rgb, a) = parse_hex_rgba_u8(input)?;
    let [r, g, b] = rgb;
    Some((
        f32::from(r) * INV_255,
        f32::from(g) * INV_255,
        f32::from(b) * INV_255,
        f32::from(a.unwrap_or(255)) * INV_255,
    ))
}

#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

#[inline]
fn srgb_u8_to_linear(c: u8) -> f32 { srgb_to_linear(f32::from(c) * INV_255) }

#[inline]
fn linear_to_srgb_u8(c: f32) -> u8 {
    let s = linear_to_srgb(c.clamp(0.0, 1.0));
    (s.mul_add(255.0, 0.5)).clamp(0.0, 255.0) as u8
}

#[must_use]
pub fn format_hex_color(rgb: [u8; 3], alpha: Option<u8>) -> String {
    #[inline]
    fn push_hex(s: &mut String, byte: u8) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let n = byte as usize;
        s.push(HEX[n >> 4] as char);
        s.push(HEX[n & 0x0f] as char);
    }
    let [r, g, b] = rgb;
    let mut out = String::with_capacity(if alpha.is_some() { 9 } else { 7 });
    out.push('#');
    push_hex(&mut out, r);
    push_hex(&mut out, g);
    push_hex(&mut out, b);
    if let Some(a) = alpha { push_hex(&mut out, a); }
    out
}

#[inline]
pub fn luminance_from_u8(r: u8, g: u8, b: u8) -> f32 {
    let rf = srgb_u8_to_linear(r);
    let gf = srgb_u8_to_linear(g);
    let bf = srgb_u8_to_linear(b);
    0.2126 * rf + 0.7152 * gf + 0.0722 * bf
}

#[inline]
pub const fn average_channel(a: u8, b: u8) -> u8 { (a & b).wrapping_add((a ^ b) >> 1) }

/// computes midpoint color between two hex strings (ignores alpha)
#[must_use]
pub fn midpoint_hex(a_hex: &str, b_hex: &str) -> String {
    let ([ar, ag, ab], _) = parse_hex_rgba_u8(a_hex).expect("invalid hex a");
    let ([br, bg, bb], _) = parse_hex_rgba_u8(b_hex).expect("invalid hex b");
    let mr = average_channel(ar, br);
    let mg = average_channel(ag, bg);
    let mb = average_channel(ab, bb);
    format_hex_color([mr, mg, mb], None)
}

/// midpoint in linear sRGB (ignores alpha)
#[must_use]
pub fn midpoint_hex_linear(a_hex: &str, b_hex: &str) -> String {
    let ([ar, ag, ab], _) = parse_hex_rgba_u8(a_hex).expect("invalid hex a");
    let ([br, bg, bb], _) = parse_hex_rgba_u8(b_hex).expect("invalid hex b");
    let r = 0.5 * (srgb_u8_to_linear(ar) + srgb_u8_to_linear(br));
    let g = 0.5 * (srgb_u8_to_linear(ag) + srgb_u8_to_linear(bg));
    let b = 0.5 * (srgb_u8_to_linear(ab) + srgb_u8_to_linear(bb));
    let rgb = [linear_to_srgb_u8(r), linear_to_srgb_u8(g), linear_to_srgb_u8(b)];
    format_hex_color(rgb, None)
}

pub mod serde_helpers {
    use serde::Deserialize;

    /// Deserialize scope which can be string or array of strings, joined by ", ".
    pub fn deserialize_scope<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Scope { S(String), V(Vec<String>) }
        Ok(Option::<Scope>::deserialize(deserializer)?.map(|s| match s { Scope::S(s) => s, Scope::V(v) => v.join(", ") }))
    }
}


