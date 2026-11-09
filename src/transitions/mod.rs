// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors
//
//! Reverse-Engineered CapCut Transitions for Concat Video Engine.
//!
//! Maps and clones the 9 transitions extracted from CapCut desktop project
//! (`draft_content.json` and effects cache):
//! 1. Eco dello zoom (Zoom Echo) [7694578172561509633] -> `concat.zoom-echo` (is_overlap: true)
//! 2. Gira in senso antiorario (Spin CCW) [6724226603372515853] -> `concat.spin-ccw` (is_overlap: false)
//! 3. Sfocatura (Blur Dissolve) [6916426617455645186] -> `concat.blur-dissolve` (is_overlap: false)
//! 4. Divisione II (Split II - Horizontal) [6973849510145430018] -> `concat.split-h` (is_overlap: false)
//! 5. Divisione III (Split III - Vertical) [6973849570912506370] -> `concat.split-v` (is_overlap: false)
//! 6. Zoom in ritardo (Delayed Zoom) [7488094700159782145] -> `concat.delayed-zoom` (is_overlap: true)
//! 7. Scorrimento zoom (Zoom Scroll) [7488157742956350737] -> `concat.zoom-scroll` (is_overlap: true)
//! 8. Scatto a strappo (Rip Snap) [7497827305750793473] -> `concat.snap-tear` (is_overlap: true)
//! 9. Ricordi che scorrono II (Flowing Memories II) [7309740200605782530] -> `concat.flowing-memories` (is_overlap: true)

use std::f32::consts::PI;

/// Metadata for a reverse-engineered CapCut transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CapCutTransitionMeta {
    /// CapCut resource ID from `draft_content.json`.
    pub resource_id: &'static str,
    /// Italian display name from CapCut UI.
    pub name_it: &'static str,
    /// English display name.
    pub name_en: &'static str,
    /// Package ID in Concat effect catalogue.
    pub package_id: &'static str,
    /// Default duration in seconds (from CapCut micros).
    pub default_duration: f64,
    /// Whether CapCut overlaps adjacent clips by the transition duration.
    ///
    /// - `true`: Outgoing and incoming clips overlap temporally on the timeline;
    ///   the cut is placed at the midpoint of the overlap, shortening both clips' visible portions.
    /// - `false`: Clips touch end-to-end without temporal source overlap;
    ///   the transition is centered on the edit point between them.
    pub is_overlap: bool,
    /// FFmpeg `xfade` fallback name for CPU / headless export.
    pub xfade_fallback: &'static str,
}

pub const CAPCUT_TRANSITIONS: &[CapCutTransitionMeta] = &[
    CapCutTransitionMeta {
        resource_id: "7694578172561509633",
        name_it: "Eco dello zoom",
        name_en: "Zoom Echo",
        package_id: "concat.zoom-echo",
        default_duration: 1.466666,
        is_overlap: true,
        xfade_fallback: "zoomin",
    },
    CapCutTransitionMeta {
        resource_id: "6724226603372515853",
        name_it: "Gira in senso antiorario",
        name_en: "Spin Counter-Clockwise",
        package_id: "concat.spin-ccw",
        default_duration: 0.466666,
        is_overlap: false,
        xfade_fallback: "radial",
    },
    CapCutTransitionMeta {
        resource_id: "6916426617455645186",
        name_it: "Sfocatura",
        name_en: "Blur Dissolve",
        package_id: "concat.blur-dissolve",
        default_duration: 0.466666,
        is_overlap: false,
        xfade_fallback: "hblur",
    },
    CapCutTransitionMeta {
        resource_id: "6973849510145430018",
        name_it: "Divisione II",
        name_en: "Split II (Horizontal)",
        package_id: "concat.split-h",
        default_duration: 0.466666,
        is_overlap: false,
        xfade_fallback: "horzopen",
    },
    CapCutTransitionMeta {
        resource_id: "6973849570912506370",
        name_it: "Divisione III",
        name_en: "Split III (Vertical)",
        package_id: "concat.split-v",
        default_duration: 0.466666,
        is_overlap: false,
        xfade_fallback: "vertopen",
    },
    CapCutTransitionMeta {
        resource_id: "7488094700159782145",
        name_it: "Zoom in ritardo",
        name_en: "Delayed Zoom",
        package_id: "concat.delayed-zoom",
        default_duration: 1.466666,
        is_overlap: true,
        xfade_fallback: "zoomin",
    },
    CapCutTransitionMeta {
        resource_id: "7488157742956350737",
        name_it: "Scorrimento zoom",
        name_en: "Zoom Scroll",
        package_id: "concat.zoom-scroll",
        default_duration: 1.000000,
        is_overlap: true,
        xfade_fallback: "slideup",
    },
    CapCutTransitionMeta {
        resource_id: "7497827305750793473",
        name_it: "Scatto a strappo",
        name_en: "Rip Snap",
        package_id: "concat.snap-tear",
        default_duration: 1.200000,
        is_overlap: true,
        xfade_fallback: "fadewhite",
    },
    CapCutTransitionMeta {
        resource_id: "7309740200605782530",
        name_it: "Ricordi che scorrono II",
        name_en: "Flowing Memories II",
        package_id: "concat.flowing-memories",
        default_duration: 0.666666,
        is_overlap: true,
        xfade_fallback: "slideright",
    },
];

/// Find transition metadata by CapCut resource ID or name.
pub fn find_capcut_transition(query: &str) -> Option<&'static CapCutTransitionMeta> {
    CAPCUT_TRANSITIONS.iter().find(|t| {
        t.resource_id == query
            || t.package_id == query
            || t.name_it.eq_ignore_ascii_case(query)
            || t.name_en.eq_ignore_ascii_case(query)
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Timeline Cut Alignment & is_overlap Arithmetic
// ─────────────────────────────────────────────────────────────────────────────

/// Represents the planned cut and transition placement between two adjacent clips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlannedCut {
    /// Timeline position of the cut point in seconds.
    pub cut_time: f64,
    /// Timeline start of the transition effect span.
    pub transition_start: f64,
    /// Duration of the transition span.
    pub transition_duration: f64,
    /// Adjusted duration for the outgoing (left) clip.
    pub left_duration: f64,
    /// Adjusted timeline start for the incoming (right) clip.
    pub right_start: f64,
    /// Adjusted timeline duration for the incoming (right) clip.
    pub right_duration: f64,
    /// Source offset shift applied to the incoming (right) clip.
    pub right_source_shift: f64,
}

/// Calculate timeline cut parameters handling CapCut's `is_overlap` semantic.
///
/// In CapCut:
/// - `is_overlap = true`: Clips on track overlap each other by `trans_duration`.
///   To map this to Concat's end-to-end clip structure, the cut is placed at `right_start + overlap / 2.0`.
///   Both clips are trimmed to meet at the cut, and the transition spans the overlap.
/// - `is_overlap = false`: Clips are adjacent (touching) without temporal overlap.
///   The cut is placed precisely at `right_start`. The clips retain their source durations,
///   and the transition spans `[cut - trans_duration / 2, cut + trans_duration / 2]`.
pub fn plan_transition_cut(
    left_start: f64,
    left_duration: f64,
    right_start: f64,
    right_duration: f64,
    speed_right: f64,
    trans_duration: f64,
    is_overlap: bool,
) -> PlannedCut {
    const MIN_CLIP_DURATION: f64 = 0.05;
    const MIN_TRANSITION: f64 = 0.05;

    let end_left = left_start + left_duration;
    let actual_overlap = (end_left - right_start).max(0.0);

    if is_overlap && actual_overlap > 1e-4 {
        // Clips overlap temporally
        let cut_time = right_start + actual_overlap / 2.0;
        let shift = cut_time - right_start;
        let adj_left_duration = (cut_time - left_start).max(MIN_CLIP_DURATION);
        let adj_right_duration = (right_duration - shift).max(MIN_CLIP_DURATION);
        let source_shift = shift * speed_right;

        let max_span = 2.0 * adj_left_duration.min(adj_right_duration);
        let span = actual_overlap.min(max_span).max(MIN_TRANSITION);
        let trans_start = (cut_time - span / 2.0).max(0.0);

        PlannedCut {
            cut_time,
            transition_start: trans_start,
            transition_duration: span,
            left_duration: adj_left_duration,
            right_start: cut_time,
            right_duration: adj_right_duration,
            right_source_shift: source_shift,
        }
    } else {
        // Clips touch without source overlap
        let cut_time = right_start;
        let max_span = 2.0 * left_duration.min(right_duration);
        let span = trans_duration.min(max_span).max(MIN_TRANSITION);
        let trans_start = (cut_time - span / 2.0).max(0.0);

        PlannedCut {
            cut_time,
            transition_start: trans_start,
            transition_duration: span,
            left_duration,
            right_start,
            right_duration,
            right_source_shift: 0.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mathematical Easing Curves
// ─────────────────────────────────────────────────────────────────────────────

#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// CapCut's cubic ease-in-out: `x < 0.5 ? 4x^3 : 1 - (-2x + 2)^3 / 2`.
#[inline]
pub fn ease_in_out_cubic(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
    }
}

/// Exact cubic Bézier curve evaluator used in CapCut's Split transitions.
///
/// Evaluates parameter `t` for given `x` via Newton-Raphson, then computes `(y, dy/dt)`.
pub fn solve_cubic_bezier(p1: [f32; 2], p2: [f32; 2], x: f32) -> (f32, f32) {
    let cx = 3.0 * p1[0];
    let bx = 3.0 * (p2[0] - p1[0]) - cx;
    let ax = 1.0 - cx - bx;

    let cy = 3.0 * p1[1];
    let by = 3.0 * (p2[1] - p1[1]) - cy;
    let ay = 1.0 - cy - by;

    let target_x = x.clamp(0.0, 1.0);
    let mut t = target_x;

    for _ in 0..6 {
        let current_x = ((ax * t + bx) * t + cx) * t;
        let dx = (3.0 * ax * t + 2.0 * bx) * t + cx;
        if dx.abs() < 1e-5 {
            break;
        }
        t = (t - (current_x - target_x) / dx).clamp(0.0, 1.0);
    }

    let y = ((ay * t + by) * t + cy) * t;
    let dy = (3.0 * ay * t + 2.0 * by) * t + cy;
    (y.clamp(0.0, 1.0), dy.max(0.0))
}

/// Mirrored coordinate wrap in normalized [0, 1] UV space.
#[inline]
pub fn mirror_uv(u: f32, v: f32) -> (f32, f32) {
    let wrap = |coord: f32| -> f32 {
        let f = (coord * 0.5 + 0.5).fract();
        (f * 2.0 - 1.0).abs()
    };
    (wrap(u), wrap(v))
}

// ─────────────────────────────────────────────────────────────────────────────
// Embedded WGSL Shaders for Concat GPU Engine
// ─────────────────────────────────────────────────────────────────────────────

pub mod shaders {
    pub const ZOOM_ECHO: &str = include_str!("shaders/zoom_echo.wgsl");
    pub const SPIN_CCW: &str = include_str!("shaders/spin_ccw.wgsl");
    pub const BLUR_DISSOLVE: &str = include_str!("shaders/blur_dissolve.wgsl");
    pub const SPLIT_HORIZONTAL: &str = include_str!("shaders/split_horizontal.wgsl");
    pub const SPLIT_VERTICAL: &str = include_str!("shaders/split_vertical.wgsl");
    pub const DELAYED_ZOOM: &str = include_str!("shaders/delayed_zoom.wgsl");
    pub const ZOOM_SCROLL: &str = include_str!("shaders/zoom_scroll.wgsl");
    pub const SNAP_TEAR: &str = include_str!("shaders/snap_tear.wgsl");
    pub const FLOWING_MEMORIES: &str = include_str!("shaders/flowing_memories.wgsl");
}
