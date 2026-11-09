// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Scatto a strappo" (Rip Snap / Whip Snap) - ID: 7497827305750793473
// is_overlap: true

const PI: f32 = 3.14159265359;

fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn rotate_stretch(uv: vec2<f32>, angle_rad: f32, scale_x: f32) -> vec2<f32> {
    let center = vec2<f32>(0.5, 0.5);
    var d = uv - center;
    d.x /= max(scale_x, 0.001);
    let s = sin(angle_rad);
    let c = cos(angle_rad);
    let rotated = vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c);
    return rotated + center;
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    let cut_point = 0.645; // 0.8s of 1.24s total duration

    // Deep Glow / Exposure flash near the tear point
    let flash = smoothstep(0.55, 0.645, p) * (1.0 - smoothstep(0.645, 0.77, p));

    var col = vec4<f32>(0.0);
    let taps: i32 = 7;

    if (p < cut_point) {
        let t = p / cut_point;
        let eased = smoothstep(0.0, 1.0, t);
        // Rotate 0 -> 25 degrees (0.4363 rad), Stretch X: 1.0 -> 2.5
        let angle = eased * 0.4363;
        let scale_x = mix(1.0, 2.5, eased);
        let blur_amt = eased * 0.02;

        let dir = vec2<f32>(cos(angle), sin(angle));
        for (var i: i32 = -3; i <= 3; i++) {
            let offset = dir * (f32(i) * blur_amt);
            let sample_uv = mirror_uv(rotate_stretch(uv + offset, angle, scale_x));
            col += from_at(sample_uv);
        }
    } else {
        let t = (p - cut_point) / (1.0 - cut_point);
        let eased = smoothstep(0.0, 1.0, t);
        // Snap back from -17 degrees (-0.2967 rad) -> 0, Stretch X: 2.5 -> 1.0
        let angle = mix(-0.2967, 0.0, eased);
        let scale_x = mix(2.5, 1.0, eased);
        let blur_amt = (1.0 - eased) * 0.02;

        let dir = vec2<f32>(cos(angle), sin(angle));
        for (var i: i32 = -3; i <= 3; i++) {
            let offset = dir * (f32(i) * blur_amt);
            let sample_uv = mirror_uv(rotate_stretch(uv + offset, angle, scale_x));
            col += to_at(sample_uv);
        }
    }

    var result = col / f32(taps);
    result = vec4<f32>(result.rgb + vec3<f32>(flash * 0.5), result.a);
    return vec4<f32>(clamp(result.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), result.a);
}
