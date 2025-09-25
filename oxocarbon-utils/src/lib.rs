#![warn(clippy::pedantic)]

const INV_255: f32 = 1.0 / 255.0;
const INVALID: u8 = 0xFF;
const HEX_DECODE: [u8; 256] = build_hex_decode();

#[inline(always)]
const fn expand_nibble(n: u8) -> u8 {
    (n << 4) | n
}

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

#[inline(always)]
fn decode_pair(h: u8, l: u8) -> Option<u8> {
    let h = HEX_DECODE[h as usize];
    let l = HEX_DECODE[l as usize];
    ((h | l) != INVALID).then_some((h << 4) | l)
}

#[inline(always)]
pub fn parse_hex_rgba_u8(input: &str) -> Option<([u8; 3], Option<u8>)> {
    let data = input.as_bytes();
    if data.is_empty() || data[0] != b'#' {
        return None;
    }
    let data = unsafe { data.get_unchecked(1..) };
    match data.len() {
        3 => {
            let r = HEX_DECODE[data[0] as usize];
            let g = HEX_DECODE[data[1] as usize];
            let b = HEX_DECODE[data[2] as usize];
            ((r | g | b) != INVALID)
                .then_some(([expand_nibble(r), expand_nibble(g), expand_nibble(b)], None))
        }
        4 => {
            let r = HEX_DECODE[data[0] as usize];
            let g = HEX_DECODE[data[1] as usize];
            let b = HEX_DECODE[data[2] as usize];
            let a = HEX_DECODE[data[3] as usize];
            ((r | g | b | a) != INVALID).then_some((
                [expand_nibble(r), expand_nibble(g), expand_nibble(b)],
                Some(expand_nibble(a)),
            ))
        }
        6 => Some((
            [
                decode_pair(data[0], data[1])?,
                decode_pair(data[2], data[3])?,
                decode_pair(data[4], data[5])?,
            ],
            None,
        )),
        8 => Some((
            [
                decode_pair(data[0], data[1])?,
                decode_pair(data[2], data[3])?,
                decode_pair(data[4], data[5])?,
            ],
            Some(decode_pair(data[6], data[7])?),
        )),
        _ => None,
    }
}

/// parses hex and returns normalized floats (0..1) rgba.
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
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
fn srgb_u8_to_linear(c: u8) -> f32 {
    srgb_to_linear(f32::from(c) * INV_255)
}

#[inline]
fn linear_to_srgb_u8(c: f32) -> u8 {
    let s = linear_to_srgb(c.clamp(0.0, 1.0));
    (s.mul_add(255.0, 0.5)).clamp(0.0, 255.0) as u8
}

#[inline(always)]
pub fn format_hex_color(rgb: [u8; 3], alpha: Option<u8>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let has_alpha = alpha.is_some();
    let len = if has_alpha { 9 } else { 7 };
    let mut buf = [0u8; 9];
    buf[0] = b'#';
    buf[1] = HEX[(rgb[0] >> 4) as usize];
    buf[2] = HEX[(rgb[0] & 0x0f) as usize];
    buf[3] = HEX[(rgb[1] >> 4) as usize];
    buf[4] = HEX[(rgb[1] & 0x0f) as usize];
    buf[5] = HEX[(rgb[2] >> 4) as usize];
    buf[6] = HEX[(rgb[2] & 0x0f) as usize];
    if let Some(a) = alpha {
        buf[7] = HEX[(a >> 4) as usize];
        buf[8] = HEX[(a & 0x0f) as usize];
    }
    unsafe { String::from_utf8_unchecked(buf[..len].to_vec()) }
}

/// computes relative luminance
/// suitable for wcag contrast checks
#[inline]
pub fn luminance_from_u8(r: u8, g: u8, b: u8) -> f32 {
    // linearize components before applying luminance weights
    let rf = srgb_u8_to_linear(r);
    let gf = srgb_u8_to_linear(g);
    let bf = srgb_u8_to_linear(b);
    0.2126 * rf + 0.7152 * gf + 0.0722 * bf
}

/// returns the rounded midpoint of two channels without overflow
#[inline]
pub const fn average_channel(a: u8, b: u8) -> u8 {
    (a & b).wrapping_add((a ^ b) >> 1)
}

/// computes midpoint color between two hex strings (ignores alpha)
#[must_use]
pub fn midpoint_hex(a_hex: &str, b_hex: &str) -> String {
    let [ar, ag, ab] = strict_rgb(a_hex, "invalid hex a");
    let [br, bg, bb] = strict_rgb(b_hex, "invalid hex b");
    format_hex_color(
        [
            average_channel(ar, br),
            average_channel(ag, bg),
            average_channel(ab, bb),
        ],
        None,
    )
}

/// averages two hex colors in linear sRGB space (ignores alpha)
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

#[inline]
pub fn pack_rgb(rgb: [u8; 3]) -> u32 {
    (u32::from(rgb[0]) << 16) | (u32::from(rgb[1]) << 8) | u32::from(rgb[2])
}

#[inline(always)]
fn strict_rgb(input: &str, label: &'static str) -> [u8; 3] {
    if let Some((rgb, _)) = parse_hex_rgba_u8(input) {
        rgb
    } else {
        invalid_hex(label)
    }
}

#[cold]
#[inline(never)]
fn invalid_hex(label: &'static str) -> ! {
    panic!("{label}");
}

#[inline(always)]
pub fn find_nearest_index(luminances: &[f32], target: f32) -> usize {
    let len = luminances.len();
    if len <= 1 {
        return 0;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        if len >= 8 {
            return unsafe { nearest_index_neon(luminances, target) };
        }
    }

    let mut left = 0;
    let mut right = len;

    while left < right {
        let mid = left + (right - left) / 2;
        if luminances[mid] < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    match left {
        0 => 0,
        n if n == len => n - 1,
        idx => {
            let prev = luminances[idx - 1];
            let next = luminances[idx];
            if ((target - prev) * 1000.0) as i32 <= ((next - target) * 1000.0) as i32 {
                idx - 1
            } else {
                idx
            }
        }
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn nearest_index_neon(luminances: &[f32], target: f32) -> usize {
    use std::arch::aarch64::*;

    unsafe {
        let len = luminances.len();
        let ptr = luminances.as_ptr();
        let target_vec = vdupq_n_f32(target);

        // process main blocks
        let chunks = len / 4;
        let mut best_idx = 0;
        let mut best_diff = f32::INFINITY;

        for chunk in 0..chunks {
            let i = chunk * 4;
            let v = vld1q_f32(ptr.add(i));
            let diff = vabsq_f32(vsubq_f32(v, target_vec));

            // Extract lanes and find minimum
            let d0 = vgetq_lane_f32(diff, 0);
            let d1 = vgetq_lane_f32(diff, 1);
            let d2 = vgetq_lane_f32(diff, 2);
            let d3 = vgetq_lane_f32(diff, 3);

            if d0 < best_diff {
                best_diff = d0;
                best_idx = i;
            }
            if d1 < best_diff {
                best_diff = d1;
                best_idx = i + 1;
            }
            if d2 < best_diff {
                best_diff = d2;
                best_idx = i + 2;
            }
            if d3 < best_diff {
                best_diff = d3;
                best_idx = i + 3;
            }
        }

        // handle remaining elements
        for i in (chunks * 4)..len {
            let diff = (*ptr.add(i) - target).abs();
            if diff < best_diff {
                best_diff = diff;
                best_idx = i;
            }
        }

        best_idx
    }
}
