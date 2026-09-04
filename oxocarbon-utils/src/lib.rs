#![warn(clippy::pedantic)]

const INV_255: f32 = 1.0 / 255.0;
const INVALID: u8 = 0xFF;
const HEX_DECODE: [u8; 256] = build_hex_decode();
const HEX_ENCODE: &[u8; 16] = b"0123456789abcdef";

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
fn decode_nibble(b: u8) -> Option<u8> {
    // SAFETY: `b` is u8 and the table is 256 entries.
    let value = unsafe { *HEX_DECODE.get_unchecked(b as usize) };
    (value != INVALID).then_some(value)
}

#[inline(always)]
fn decode_pair(h: u8, l: u8) -> Option<u8> {
    Some((decode_nibble(h)? << 4) | decode_nibble(l)?)
}

#[inline(always)]
pub fn parse_hex_rgba_u8(input: &str) -> Option<([u8; 3], Option<u8>)> {
    let data = input.as_bytes().strip_prefix(b"#")?;
    match data.len() {
        3 => Some((
            [
                expand_nibble(decode_nibble(data[0])?),
                expand_nibble(decode_nibble(data[1])?),
                expand_nibble(decode_nibble(data[2])?),
            ],
            None,
        )),
        4 => Some((
            [
                expand_nibble(decode_nibble(data[0])?),
                expand_nibble(decode_nibble(data[1])?),
                expand_nibble(decode_nibble(data[2])?),
            ],
            Some(expand_nibble(decode_nibble(data[3])?)),
        )),
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
        c * (1.0 / 12.92)
    } else {
        let x = (c + 0.055) * 0.9478672985781991;
        x * x * x.sqrt()
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
    let len = 7 + usize::from(alpha.is_some()) * 2;
    let mut out = String::with_capacity(len);
    // SAFETY: we write `len` ASCII bytes then set_len; HEX_ENCODE is 0-9a-f.
    unsafe {
        let buf = out.as_mut_vec();
        buf.set_len(len);
        buf[0] = b'#';
        let mut idx = 1;
        for &byte in &rgb {
            buf[idx] = HEX_ENCODE[(byte >> 4) as usize];
            buf[idx + 1] = HEX_ENCODE[(byte & 0x0f) as usize];
            idx += 2;
        }
        if let Some(a) = alpha {
            buf[idx] = HEX_ENCODE[(a >> 4) as usize];
            buf[idx + 1] = HEX_ENCODE[(a & 0x0f) as usize];
        }
    }
    out
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
    parse_hex_rgba_u8(input)
        .unwrap_or_else(|| panic!("{label}"))
        .0
}

/// Nearest neighbour on a luminance table sorted ascending.
#[inline]
#[must_use]
pub fn find_nearest_index_sorted(luminances: &[f32], target: f32) -> usize {
    match luminances.binary_search_by(|x| x.total_cmp(&target)) {
        Ok(i) => i,
        Err(0) => 0,
        Err(i) if i >= luminances.len() => luminances.len().saturating_sub(1),
        Err(i) => {
            if (target - luminances[i - 1]).abs() <= (luminances[i] - target).abs() {
                i - 1
            } else {
                i
            }
        }
    }
}

#[inline(always)]
pub fn find_nearest_index(luminances: &[f32], target: f32) -> usize {
    let len = luminances.len();
    if len == 0 {
        return 0;
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { nearest_index_neon(luminances, target) };
    }

    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
    {
        nearest_index_scalar(luminances, target)
    }
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
#[target_feature(enable = "neon")]
unsafe fn nearest_index_neon(luminances: &[f32], target: f32) -> usize {
    use std::arch::aarch64::*;

    let len = luminances.len();
    if len == 0 {
        return 0;
    }

    let ptr = luminances.as_ptr();
    let target_vec = vdupq_n_f32(target);
    let mut best_idx = 0;
    let mut best_diff = f32::INFINITY;

    // smaller arrays scaler anyways
    if len < 4 {
        return nearest_index_scalar(luminances, target);
    }

    let chunks = len / 4;
    let remainder = len % 4;

    // prefetch for better performance with larger arrays
    if chunks > 8 {
        unsafe { std::arch::asm!("prfm pldl1keep, [{}]", in(reg) ptr) };
        unsafe { std::arch::asm!("prfm pldl1keep, [{}]", in(reg) ptr.add(32)) };
    }

    for chunk in 0..chunks {
        let i = chunk * 4;
        let v = unsafe { vld1q_f32(ptr.add(i)) };
        let diff = vabsq_f32(vsubq_f32(v, target_vec));

        let min_in_vec = vminvq_f32(diff);
        if min_in_vec >= best_diff {
            continue;
        }

        best_diff = min_in_vec;
        let min_mask = vceqq_f32(diff, vdupq_n_f32(min_in_vec));

        if vgetq_lane_u32(min_mask, 0) != 0 {
            best_idx = i;
        } else if vgetq_lane_u32(min_mask, 1) != 0 {
            best_idx = i + 1;
        } else if vgetq_lane_u32(min_mask, 2) != 0 {
            best_idx = i + 2;
        } else {
            best_idx = i + 3;
        }

        if best_diff == 0.0 {
            return best_idx;
        }
    }

    if remainder > 0 {
        let remainder_start = chunks * 4;
        for i in remainder_start..len {
            let diff = unsafe { (*ptr.add(i) - target).abs() };
            if diff < best_diff {
                best_diff = diff;
                best_idx = i;
                if best_diff == 0.0 {
                    return best_idx;
                }
            }
        }
    }

    best_idx
}

#[inline]
fn nearest_index_scalar(luminances: &[f32], target: f32) -> usize {
    let mut best_idx = 0;
    let mut best_diff = f32::INFINITY;

    for (i, &value) in luminances.iter().enumerate() {
        let diff = (value - target).abs();
        if diff < best_diff {
            best_diff = diff;
            best_idx = i;
        }
    }

    best_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rgb_and_rgba() {
        assert_eq!(
            parse_hex_rgba_u8("#1a2b3c"),
            Some(([0x1a, 0x2b, 0x3c], None))
        );
        assert_eq!(
            parse_hex_rgba_u8("#1a2b3c80"),
            Some(([0x1a, 0x2b, 0x3c], Some(0x80)))
        );
        assert_eq!(parse_hex_rgba_u8("#abc"), Some(([0xaa, 0xbb, 0xcc], None)));
        assert_eq!(parse_hex_rgba_u8("nope"), None);
    }

    #[test]
    fn format_roundtrip() {
        let rgb = [0x08, 0xbd, 0xba];
        assert_eq!(format_hex_color(rgb, None), "#08bdba");
        assert_eq!(format_hex_color(rgb, Some(0x40)), "#08bdba40");
        let (parsed, alpha) = parse_hex_rgba_u8("#08bdba40").unwrap();
        assert_eq!(parsed, rgb);
        assert_eq!(alpha, Some(0x40));
    }

    #[test]
    fn midpoint_is_channelwise_average() {
        assert_eq!(midpoint_hex("#000000", "#ffffff"), "#7f7f7f");
        assert_eq!(midpoint_hex("#161616", "#262626"), "#1e1e1e");
    }

    #[test]
    fn nearest_luminance_picks_closest() {
        let lum = [0.0_f32, 0.25, 0.5, 0.75, 1.0];
        assert_eq!(find_nearest_index(&lum, 0.0), 0);
        assert_eq!(find_nearest_index(&lum, 0.51), 2);
        assert_eq!(find_nearest_index(&lum, 1.0), 4);
        assert_eq!(find_nearest_index_sorted(&lum, 0.0), 0);
        assert_eq!(find_nearest_index_sorted(&lum, 0.51), 2);
        assert_eq!(find_nearest_index_sorted(&lum, 1.0), 4);
        assert_eq!(find_nearest_index_sorted(&lum, -1.0), 0);
        assert_eq!(find_nearest_index_sorted(&lum, 2.0), 4);
    }
}
