#![warn(clippy::pedantic)]

const INV_255: f32 = 1.0 / 255.0;

#[inline]
const fn expand_nibble(n: u8) -> u8 { (n << 4) | n }

const INVALID: u8 = 0xFF;

const fn build_hex_decode() -> [u8; 256] {
    let mut table = [INVALID; 256];
    let mut i = 0;
    while i < 10 {
        table[b'0' as usize + i] = i as u8;
        i += 1;
    }
    i = 0;
    while i < 6 {
        let value = (10 + i) as u8;
        table[b'a' as usize + i] = value;
        table[b'A' as usize + i] = value;
        i += 1;
    }
    table
}

const HEX_DECODE: [u8; 256] = build_hex_decode();

#[inline(always)]
fn nibble_value(b: u8) -> u8 { HEX_DECODE[b as usize] }

#[inline(always)]
fn decode_nibble(b: u8) -> Option<u8> {
    let value = nibble_value(b);
    (value != INVALID).then_some(value)
}

#[inline(always)]
fn decode_hex_u32(bytes: &[u8]) -> Option<u32> {
    let mut acc = 0u32;
    for &b in bytes {
        acc = (acc << 4) | u32::from(decode_nibble(b)?);
    }
    Some(acc)
}

/// parses hex color strings: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
/// returns (rgb, alpha) with 8-bit channels; alpha is None if not provided.
pub fn parse_hex_rgba_u8(input: &str) -> Option<([u8; 3], Option<u8>)> {
    let data = input.as_bytes().strip_prefix(b"#")?;
    match data.len() {
        3 => {
            let raw = decode_hex_u32(data)?;
            let rgb = [
                expand_nibble(((raw >> 8) & 0xF) as u8),
                expand_nibble(((raw >> 4) & 0xF) as u8),
                expand_nibble((raw & 0xF) as u8),
            ];
            Some((rgb, None))
        }
        4 => {
            let raw = decode_hex_u32(data)?;
            let rgb = [
                expand_nibble(((raw >> 12) & 0xF) as u8),
                expand_nibble(((raw >> 8) & 0xF) as u8),
                expand_nibble(((raw >> 4) & 0xF) as u8),
            ];
            let alpha = expand_nibble((raw & 0xF) as u8);
            Some((rgb, Some(alpha)))
        }
        6 => {
            let raw = decode_hex_u32(data)?;
            let rgb = [
                (raw >> 16) as u8,
                (raw >> 8) as u8,
                raw as u8,
            ];
            Some((rgb, None))
        }
        8 => {
            let raw = decode_hex_u32(data)?;
            let rgb = [
                (raw >> 24) as u8,
                (raw >> 16) as u8,
                (raw >> 8) as u8,
            ];
            Some((rgb, Some(raw as u8)))
        }
        _ => None,
    }
}

/// Parses hex and returns normalized floats (0..1) rgba.
pub fn parse_hex_rgba_f32(input: &str) -> Option<(f32, f32, f32, f32)> {
    parse_hex_rgba_u8(input).map(|([r, g, b], a)| {
        let scale = INV_255;
        (
            f32::from(r) * scale,
            f32::from(g) * scale,
            f32::from(b) * scale,
            f32::from(a.unwrap_or(255)) * scale,
        )
    })
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

#[inline(always)]
pub fn format_hex_color(rgb: [u8; 3], alpha: Option<u8>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let len = 7 + alpha.map_or(0, |_| 2);
    let mut out = String::with_capacity(len);
    unsafe {
        let vec = out.as_mut_vec();
        vec.set_len(len);
        vec[0] = b'#';
        let mut idx = 1;
        for &byte in &rgb {
            vec[idx] = HEX[(byte >> 4) as usize];
            vec[idx + 1] = HEX[(byte & 0x0f) as usize];
            idx += 2;
        }
        if let Some(a) = alpha {
            vec[idx] = HEX[(a >> 4) as usize];
            vec[idx + 1] = HEX[(a & 0x0f) as usize];
        }
    }
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
    let [ar, ag, ab] = strict_rgb(a_hex, "invalid hex a");
    let [br, bg, bb] = strict_rgb(b_hex, "invalid hex b");
    format_hex_color([
        average_channel(ar, br),
        average_channel(ag, bg),
        average_channel(ab, bb),
    ], None)
}

/// midpoint in linear sRGB (ignores alpha)
#[must_use]
pub fn midpoint_hex_linear(a_hex: &str, b_hex: &str) -> String {
    let [ar, ag, ab] = strict_rgb(a_hex, "invalid hex a");
    let [br, bg, bb] = strict_rgb(b_hex, "invalid hex b");
    let rgb = [
        linear_to_srgb_u8(0.5 * (srgb_u8_to_linear(ar) + srgb_u8_to_linear(br))),
        linear_to_srgb_u8(0.5 * (srgb_u8_to_linear(ag) + srgb_u8_to_linear(bg))),
        linear_to_srgb_u8(0.5 * (srgb_u8_to_linear(ab) + srgb_u8_to_linear(bb))),
    ];
    format_hex_color(rgb, None)
}

#[inline(always)]
fn strict_rgb(input: &str, label: &'static str) -> [u8; 3] {
    if let Some((rgb, _)) = parse_hex_rgba_u8(input) { rgb } else { invalid_hex(label) }
}

#[cold]
#[inline(never)]
fn invalid_hex(label: &'static str) -> ! {
    panic!("{label}");
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


