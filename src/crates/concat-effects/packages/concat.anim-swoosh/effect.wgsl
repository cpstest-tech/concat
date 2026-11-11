struct Params { duration: f32, start: f32 }

fn solve_bezier_y(p1: vec2<f32>, p2: vec2<f32>, x: f32) -> f32 {
    let cx = 3.0 * p1.x;
    let bx = 3.0 * (p2.x - p1.x) - cx;
    let ax = 1.0 - cx - bx;

    let cy = 3.0 * p1.y;
    let by = 3.0 * (p2.y - p1.y) - cy;
    let ay = 1.0 - cy - by;

    var t = clamp(x, 0.0, 1.0);
    for (var i: i32 = 0; i < 5; i++) {
        let current_x = ((ax * t + bx) * t + cx) * t;
        let dx = (3.0 * ax * t + 2.0 * bx) * t + cx;
        if (abs(dx) >= 1e-5) {
            t = clamp(t - (current_x - x) / dx, 0.0, 1.0);
        }
    }

    let y = ((ay * t + by) * t + cy) * t;
    return clamp(y, 0.0, 1.0);
}

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let duration = max(params.duration, 0.001);
    let p = clamp((frame.clip_time - params.start) / duration, 0.0, 1.0);
    if (p <= 0.0 || p >= 1.0) {
        return sample(uv);
    }

    // CapCut Swoosh (7274915008939561473):
    // Wave warp over 0..8/21, vertical GaussianBlur 300->0 over 0..15/21, Exposure 1.83->0 over 0..1
    let p_wave = clamp(p / (8.0 / 21.0), 0.0, 1.0);
    let wave_env = 1.0 - solve_bezier_y(vec2<f32>(0.001, 0.0), vec2<f32>(0.1, 0.9), p_wave);
    let wave_dx = sin((uv.y - 0.5) * 6.2831853 * 1.4 + p * 8.0) * 0.035 * wave_env;

    let p_blur = clamp(p / (15.0 / 21.0), 0.0, 1.0);
    let blur_y = (1.0 - solve_bezier_y(vec2<f32>(0.001, 0.0), vec2<f32>(0.0, 1.0), p_blur)) * 0.085;
    let blur_x = (1.0 - solve_bezier_y(vec2<f32>(0.001, 0.0), vec2<f32>(0.5, 1.0), p_blur)) * 0.004;

    var acc = vec4<f32>(0.0);
    for (var i: i32 = -4; i <= 4; i++) {
        let t = f32(i) / 4.0;
        let sample_uv = clamp(uv + vec2<f32>(wave_dx + t * blur_x, t * blur_y), vec2<f32>(0.0), vec2<f32>(1.0));
        acc += sample(sample_uv);
    }
    let col = acc / 9.0;

    let exp_env = (1.0 - solve_bezier_y(vec2<f32>(0.52, 0.179), vec2<f32>(0.73, 1.0), p)) * smoothstep(0.0, 0.02, p);
    let stops = 1.83 * exp_env;
    return vec4<f32>(exposure(col.rgb, stops), col.a);
}
