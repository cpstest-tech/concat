// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Eco dello zoom" (Zoom Echo) - ID: 7694578172561509633
// is_overlap: true

struct Params {
    blur_strength: f32,   // Default: 0.18 [0.0..0.5]
    zoom_scale_a: f32,    // Default: 0.82 [0.6..1.0]
    sat_boost: f32,       // Default: 0.45 [0.0..1.0]
    contrast_boost: f32,  // Default: 0.20 [0.0..0.6]
}

fn sample_zoom_blur_from(uv: vec2<f32>, strength: f32, scale: f32) -> vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let uv_scaled = (uv - center) * scale + center;
    let dir = uv_scaled - center;
    let ratio = frame.size.x / max(frame.size.y, 1.0);
    let dir_aspect = vec2<f32>(dir.x * ratio, dir.y);
    let len = length(dir_aspect);
    let unit_aspect = select(vec2<f32>(0.0), dir_aspect / len, len > 1e-6);
    let unit = vec2<f32>(unit_aspect.x / ratio, unit_aspect.y);

    let o1 = unit * strength * 0.8;
    let o2 = unit * strength * 0.4;

    var c = vec4<f32>(0.0);
    c += from_at(clamp(uv_scaled - o1, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += from_at(clamp(uv_scaled - o2, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += from_at(clamp(uv_scaled, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += from_at(clamp(uv_scaled + o2, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += from_at(clamp(uv_scaled + o1, vec2<f32>(0.0), vec2<f32>(1.0)));
    return c * 0.2;
}

fn sample_zoom_blur_to(uv: vec2<f32>, strength: f32, scale: f32) -> vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let uv_scaled = (uv - center) * scale + center;
    let dir = uv_scaled - center;
    let ratio = frame.size.x / max(frame.size.y, 1.0);
    let dir_aspect = vec2<f32>(dir.x * ratio, dir.y);
    let len = length(dir_aspect);
    let unit_aspect = select(vec2<f32>(0.0), dir_aspect / len, len > 1e-6);
    let unit = vec2<f32>(unit_aspect.x / ratio, unit_aspect.y);

    let o1 = unit * strength * 0.8;
    let o2 = unit * strength * 0.4;

    var c = vec4<f32>(0.0);
    c += to_at(clamp(uv_scaled - o1, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += to_at(clamp(uv_scaled - o2, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += to_at(clamp(uv_scaled, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += to_at(clamp(uv_scaled + o2, vec2<f32>(0.0), vec2<f32>(1.0)));
    c += to_at(clamp(uv_scaled + o1, vec2<f32>(0.0), vec2<f32>(1.0)));
    return c * 0.2;
}

fn adjust_saturation(color: vec3<f32>, sat: f32) -> vec3<f32> {
    let luma = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return mix(vec3<f32>(luma), color, sat);
}

fn adjust_contrast(color: vec3<f32>, ct: f32) -> vec3<f32> {
    return (color - vec3<f32>(0.5)) * ct + vec3<f32>(0.5);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let texel1 = from_at(uv);
    let texel2 = to_at(uv);

    let fade_in_end = 0.1;
    let fade_out_start = 0.9;

    let trans_p = clamp((progress - fade_in_end) / (fade_out_start - fade_in_end), 0.0, 1.0);
    let p = smoothstep(0.0, 1.0, trans_p);

    let bell = 1.0 - abs(2.0 * p - 1.0);
    let blur_amt = params.blur_strength * bell;

    let scale_a = mix(1.0, params.zoom_scale_a, p);
    let scale_b = mix(1.08, 1.0, p);

    let a_blur = sample_zoom_blur_from(uv, blur_amt, scale_a);
    let b_blur = sample_zoom_blur_to(uv, blur_amt, scale_b);

    let style_peak = smoothstep(0.20, 0.50, p) * (1.0 - smoothstep(0.50, 1.0, p));
    let sat = 1.0 + params.sat_boost * style_peak;
    let ct = 1.0 + params.contrast_boost * style_peak;

    var b_styled = adjust_saturation(b_blur.rgb, sat);
    b_styled = adjust_contrast(b_styled, ct);
    b_styled = clamp(b_styled, vec3<f32>(0.0), vec3<f32>(1.0));

    let intro = smoothstep(0.32, 0.55, p);
    var alpha_b = mix(0.0, 0.3, intro);
    alpha_b = mix(alpha_b, 1.0, smoothstep(0.55, 1.0, p));

    let trans_rgb = mix(a_blur.rgb, b_styled, alpha_b);
    let trans_a = mix(a_blur.a, b_blur.a, alpha_b);
    let trans_col = vec4<f32>(clamp(trans_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), clamp(trans_a, 0.0, 1.0));

    let blend_in = smoothstep(0.0, fade_in_end, progress);
    let blend_out = smoothstep(fade_out_start, 1.0, progress);

    var final_col = mix(texel1, trans_col, blend_in);
    final_col = mix(final_col, texel2, blend_out);
    return final_col;
}
