struct Params { duration: f32, start: f32, steps: f32 }

// CapCut Traccia del cursore (7663119181289721106):
// Progressive character reveal with LumiDirectionalBlurs + LumiSoftGlow at the active leading edge.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let duration = max(params.duration, 0.001);
    let progress = clamp((frame.clip_time - params.start) / duration, 0.0, 1.0);
    if (progress <= 0.0001) {
        return vec4<f32>(0.0);
    }
    if (progress >= 0.9999) {
        return sample(uv);
    }

    let order = reveal_order(uv);
    let threshold = progress * 1.18 - 0.02;
    if (order > threshold) {
        return vec4<f32>(0.0);
    }

    let trail_width = 0.18;
    let trail_t = clamp((threshold - order) / trail_width, 0.0, 1.0);
    let edge = 1.0 - trail_t;

    let blur_x = edge * edge * 0.026;
    var acc = vec4<f32>(0.0);
    for (var i: i32 = -3; i <= 3; i++) {
        let offset_x = f32(i) * (blur_x / 3.0);
        let s_uv = clamp(uv + vec2<f32>(offset_x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0));
        acc += sample(s_uv);
    }
    let col = acc / 7.0;

    let glow = pow(edge, 1.6) * 1.4;
    let lit = exposure(col.rgb, glow) + vec3<f32>(0.70, 0.88, 1.0) * (glow * 0.28 * col.a);
    let alpha_ramp = smoothstep(0.0, 0.15, trail_t);
    return vec4<f32>(lit * alpha_ramp, col.a * alpha_ramp);
}
