// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Ricordi che scorrono II" (Flowing Memories II) - ID: 7309740200605782530
// is_overlap: true

fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);

    // Exposure flash peaking right at the transition center (p = 0.5)
    let flash = (1.0 - abs(2.0 * p - 1.0)) * 0.35;

    // Motion blur bell curve peaking at p = 0.5
    let blur_amt = (1.0 - abs(2.0 * p - 1.0)) * 0.04;

    var sum = vec4<f32>(0.0);
    let taps: i32 = 7;

    if (p < 0.5) {
        let t = p * 2.0;
        let eased = smoothstep(0.0, 1.0, t);
        // Clip A pans rightward: x offset 0.0 -> +1.52, scale 1.0 -> 2.0
        let shift_x = mix(0.0, 1.517, eased);
        let scale = mix(1.0, 2.0, eased);

        for (var i: i32 = -3; i <= 3; i++) {
            let offset_x = f32(i) * blur_amt;
            let centered = (uv - vec2<f32>(0.5, 0.5)) / scale + vec2<f32>(0.5, 0.5);
            let sample_uv = mirror_uv(centered - vec2<f32>(shift_x + offset_x, 0.0));
            sum += from_at(sample_uv);
        }
    } else {
        let t = (p - 0.5) * 2.0;
        let eased = smoothstep(0.0, 1.0, t);
        // Clip B snaps from left: x offset -0.76 -> 0.0, scale 0.6 -> 1.0
        let shift_x = mix(-0.755, 0.0, eased);
        let scale = mix(0.6, 1.0, eased);

        for (var i: i32 = -3; i <= 3; i++) {
            let offset_x = f32(i) * blur_amt;
            let centered = (uv - vec2<f32>(0.5, 0.5)) / scale + vec2<f32>(0.5, 0.5);
            let sample_uv = mirror_uv(centered - vec2<f32>(shift_x + offset_x, 0.0));
            sum += to_at(sample_uv);
        }
    }

    var result = sum / f32(taps);
    result = vec4<f32>(result.rgb + vec3<f32>(flash), result.a);
    return vec4<f32>(clamp(result.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), result.a);
}
