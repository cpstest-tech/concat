// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Zoom in ritardo" (Delayed Zoom / Prism Flash) - ID: 7488094700159782145
// is_overlap: true

fn zoom_uv(uv: vec2<f32>, scale: f32) -> vec2<f32> {
    return (uv - vec2<f32>(0.5)) / max(scale, 0.0001) + vec2<f32>(0.5);
}

fn sample_prism_from(uv: vec2<f32>, scale: f32, dispersion: f32) -> vec4<f32> {
    let r_uv = zoom_uv(uv, scale * (1.0 + dispersion));
    let g_uv = zoom_uv(uv, scale);
    let b_uv = zoom_uv(uv, scale * (1.0 - dispersion));

    let r = from_at(r_uv).r;
    let g = from_at(g_uv).g;
    let b = from_at(b_uv).b;
    let a = from_at(g_uv).a;
    return vec4<f32>(r, g, b, a);
}

fn sample_prism_to(uv: vec2<f32>, scale: f32, dispersion: f32) -> vec4<f32> {
    let r_uv = zoom_uv(uv, scale * (1.0 + dispersion));
    let g_uv = zoom_uv(uv, scale);
    let b_uv = zoom_uv(uv, scale * (1.0 - dispersion));

    let r = to_at(r_uv).r;
    let g = to_at(g_uv).g;
    let b = to_at(b_uv).b;
    let a = to_at(g_uv).a;
    return vec4<f32>(r, g, b, a);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);

    // Easing curves matching LumiExportData keyframes
    // Clip A shrinks from 1.0 down to 0.90
    let p_a = smoothstep(0.0, 0.5, p);
    let scale_a = mix(1.0, 0.90, p_a);

    // Chromatic dispersion bell curve peaking around midpoint
    let disp_peak = smoothstep(0.25, 0.48, p) * (1.0 - smoothstep(0.48, 0.72, p));
    let dispersion = 0.025 * disp_peak;

    // Flash intensity peaking at cut midpoint
    let flash = smoothstep(0.25, 0.48, p) * (1.0 - smoothstep(0.48, 0.71, p));

    // Clip B starts delayed at p = 0.5, arrives at 1.20x scale and settles to 1.00x
    let p_b = smoothstep(0.5, 0.98, p);
    let scale_b = mix(1.20, 1.00, p_b);

    // Alpha cross-dissolve with quick decay for outgoing clip
    let a_opacity = 1.0 - smoothstep(0.48, 0.71, p);
    let b_opacity = smoothstep(0.48, 0.55, p);

    let col_a = sample_prism_from(uv, scale_a, dispersion);
    let col_b = sample_prism_to(uv, scale_b, dispersion);

    var mixed_col = mix(col_a, col_b, b_opacity);
    if (a_opacity < 1.0) {
        mixed_col = mix(vec4<f32>(col_b.rgb, col_b.a), mixed_col, a_opacity);
    }

    // Additive brightness flash
    let lit_rgb = mixed_col.rgb + vec3<f32>(flash * 0.4);
    return vec4<f32>(clamp(lit_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), mixed_col.a);
}
