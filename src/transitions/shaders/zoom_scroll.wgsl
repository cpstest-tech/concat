// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Scorrimento zoom" (Zoom Scroll / Vertical Slide Zoom) - ID: 7488157742956350737
// is_overlap: true

fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);

    // Vertical blur bell curve peaking at p = 0.5
    let blur_peak = smoothstep(0.2, 0.5, p) * (1.0 - smoothstep(0.5, 0.65, p));
    let blur_amt = blur_peak * 0.035;

    // Outgoing clip transformations
    let scale_a = mix(1.0, 1.5, smoothstep(0.0, 0.4, p));
    let shift_y_a = mix(0.0, -1.3, smoothstep(0.26, 0.5, p));

    // Incoming clip transformations
    let scale_b = mix(1.5, 1.0, smoothstep(0.5, 0.96, p));
    let shift_y_b = mix(1.1, 0.0, smoothstep(0.5, 0.96, p));

    let is_second_half = step(0.5, p);

    var sum = vec4<f32>(0.0);
    let taps: i32 = 7;

    for (var i: i32 = -3; i <= 3; i++) {
        let offset_y = f32(i) * blur_amt;

        if (is_second_half < 0.5) {
            let centered = (uv - vec2<f32>(0.5, 0.5)) / scale_a + vec2<f32>(0.5, 0.5);
            let sample_uv = mirror_uv(centered + vec2<f32>(0.0, shift_y_a + offset_y));
            sum += from_at(sample_uv);
        } else {
            let centered = (uv - vec2<f32>(0.5, 0.5)) / scale_b + vec2<f32>(0.5, 0.5);
            let sample_uv = mirror_uv(centered + vec2<f32>(0.0, shift_y_b + offset_y));
            sum += to_at(sample_uv);
        }
    }

    return sum / f32(taps);
}
