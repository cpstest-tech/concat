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
    if (p <= 0.0) {
        return sample(uv);
    }

    // CapCut Fumo (7229983825080619522):
    // Wave Warp 0->40, Radial Blur 0->50 with upward drifting center, Gaussian Blur 0->35
    let wave_env = solve_bezier_y(vec2<f32>(0.5, 0.0), vec2<f32>(0.999, 1.0), p);
    let radial_env = solve_bezier_y(vec2<f32>(0.001, 0.0), vec2<f32>(0.66, 1.0), p);
    let gauss_env = solve_bezier_y(vec2<f32>(0.48, 0.0), vec2<f32>(0.78, 1.0), p);

    let wave_offset = vec2<f32>(
        sin(uv.y * 10.0 + p * 7.0) * 0.028 * wave_env,
        cos(uv.x * 8.0 - p * 5.0) * 0.020 * wave_env
    );

    let center = vec2<f32>(0.5, 0.5 - 0.17 * radial_env);
    let radial_dir = (uv + wave_offset) - center;

    var acc = vec4<f32>(0.0);
    for (var i: i32 = -3; i <= 3; i++) {
        let t = f32(i) / 3.0;
        let scale = 1.0 + t * 0.12 * radial_env;
        let g_offset = vec2<f32>(t * 0.022 * gauss_env, -abs(t) * 0.025 * gauss_env);
        let sample_uv = clamp(center + radial_dir * scale + g_offset, vec2<f32>(0.0), vec2<f32>(1.0));
        acc += sample(sample_uv);
    }
    return acc / 7.0;
}
