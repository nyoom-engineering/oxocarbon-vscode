use std::sync::OnceLock;

use oxocarbon_utils::{find_nearest_index, luminance_from_u8, pack_rgb, parse_hex_rgba_u8};

// IBM Gray family (Carbon): darkest to lightest + white
const GRAY_RAMP: [&str; 10] = [
    "#161616", // Gray 100
    "#262626", // Gray 90
    "#393939", // Gray 80
    "#525252", // Gray 70
    "#6f6f6f", // Gray 60
    "#8d8d8d", // Gray 50
    "#a8a8a8", // Gray 40
    "#c6c6c6", // Gray 30
    "#e0e0e0", // Gray 20
    "#f4f4f4", // Gray 10
];

// IBM Cool Gray family
const COOL_GRAY_RAMP: [&str; 10] = [
    "#121619", // Cool Gray 100
    "#21272a", // Cool Gray 90
    "#343a3f", // Cool Gray 80
    "#4d5358", // Cool Gray 70
    "#697077", // Cool Gray 60
    "#878d96", // Cool Gray 50
    "#a2a9b0", // Cool Gray 40
    "#c1c7cd", // Cool Gray 30
    "#dde1e6", // Cool Gray 20
    "#f2f4f8", // Cool Gray 10
];

// IBM Warm Gray family
const WARM_GRAY_RAMP: [&str; 10] = [
    "#171414", // Warm Gray 100
    "#272525", // Warm Gray 90
    "#3c3838", // Warm Gray 80
    "#565151", // Warm Gray 70
    "#726e6e", // Warm Gray 60
    "#8f8b8b", // Warm Gray 50
    "#ada8a8", // Warm Gray 40
    "#cac5c4", // Warm Gray 30
    "#e5e0df", // Warm Gray 20
    "#f7f3f2", // Warm Gray 10
];

// common monochrome hues used in OLED mappings, include across all ramps
const MONO_RAMP_EXTRAS: [&str; 13] = [
    "#000000", "#0b0b0b", "#0f0f0f", "#161616", "#1b1b1b", "#1e1e1e", "#212121", "#262626",
    "#393939", "#525252", "#dde1e6", "#f2f4f8", "#ffffff",
];

const MONO_ACCENT_CANDIDATES: [u32; 10] = [
    0x08bdba, 0x33b1ff, 0x3ddbd9, 0x42be65, 0x78a9ff, 0x82cfff, 0xa6c8ff, 0xbe95ff, 0xee5396,
    0xff7eb6,
];

const MONO_PRINT_EXTRA_ACCENTS: u32 = 0x0f62fe;

static MONOCHROME_RAMPS: OnceLock<MonochromeRamps> = OnceLock::new();

#[repr(C)]
struct MonochromeRamps {
    default: MonoRamp,
    cool: MonoRamp,
    warm: MonoRamp,
}

pub(crate) struct MonoRamp {
    luminances: &'static [f32],
    rgbs: &'static [[u8; 3]],
}

impl MonoRamp {
    #[inline(always)]
    pub(crate) fn nearest_rgb(&self, target: f32) -> [u8; 3] {
        self.rgbs[find_nearest_index(self.luminances, target)]
    }
}

#[inline]
fn monochrome_ramps() -> &'static MonochromeRamps {
    MONOCHROME_RAMPS.get_or_init(build_monochrome_ramps)
}

fn build_monochrome_ramps() -> MonochromeRamps {
    MonochromeRamps {
        default: build_ramp(&GRAY_RAMP),
        cool: build_ramp(&COOL_GRAY_RAMP),
        warm: build_ramp(&WARM_GRAY_RAMP),
    }
}

fn build_ramp(base: &'static [&'static str]) -> MonoRamp {
    let mut entries: Vec<(f32, [u8; 3])> = Vec::with_capacity(MONO_RAMP_EXTRAS.len() + base.len());
    let mut seen = 0u64;

    for &hex in MONO_RAMP_EXTRAS.iter().chain(base.iter()) {
        let (rgb, _) = parse_hex_rgba_u8(hex).unwrap();
        let packed = pack_rgb(rgb) as u64;
        if seen & (1u64 << (packed % 64)) != 0 {
            continue;
        }
        seen |= 1u64 << (packed % 64);
        let lum = luminance_from_u8(rgb[0], rgb[1], rgb[2]);
        entries.push((lum, rgb));
    }

    entries.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));

    let (luminances, rgbs): (Vec<f32>, Vec<[u8; 3]>) = entries.into_iter().unzip();

    MonoRamp {
        luminances: Box::leak(luminances.into_boxed_slice()),
        rgbs: Box::leak(rgbs.into_boxed_slice()),
    }
}

#[inline(always)]
pub(crate) fn select_monochrome_ramp(family: &str) -> &'static MonoRamp {
    let ramps = monochrome_ramps();
    match family {
        "coolgray" | "cool-gray" | "cool" => &ramps.cool,
        "warmgray" | "warm-gray" | "warm" => &ramps.warm,
        _ => &ramps.default,
    }
}

#[inline(always)]
pub(crate) fn is_monochrome_candidate(rgb: [u8; 3], is_print: bool) -> bool {
    let value = pack_rgb(rgb);
    MONO_ACCENT_CANDIDATES.contains(&value) || (is_print && value == MONO_PRINT_EXTRA_ACCENTS)
}
