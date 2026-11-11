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

    // CapCut Sfocatura orizzontale (7302338783930749442):
    // Horizontal Gaussian blur 350->0, vertical blur 14->0, exposure +1.0->0, saturation +0.4->0
    let blur_x = (1.0 - solve_bezier_y(vec2<f32>(0.001, 0.0), vec2<f32>(0.0, 0.895), p)) * 0.095;
    let blur_y = pow(max(1.0 - p, 0.0), 3.0) * 0.006;

    var acc = vec4<f32>(0.0);
    for (var i: i32 = -4; i <= 4; i++) {
        let t = f32(i) / 4.0;
        let sample_uv = clamp(uv + vec2<f32>(t * blur_x, t * blur_y), vec2<f32>(0.0), vec2<f32>(1.0));
        acc += sample(sample_uv);
    }
    let col = acc / 9.0;

    let gate = smoothstep(0.0, 0.02, p);
    let exp_env = (1.0 - solve_bezier_y(vec2<f32>(0.6, 0.22), vec2<f32>(0.66, 1.0), p)) * gate;
    let sat_env = (1.0 - solve_bezier_y(vec2<f32>(0.001, 0.0), vec2<f32>(0.66, 1.0), p)) * gate;

    let lit = exposure(col.rgb, 1.0 * exp_env);
    let grey = dot(lit, vec3<f32>(0.2126, 0.7152, 0.0722));
    let sat = mix(vec3<f32>(grey), lit, 1.0 + 0.40 * sat_env);
    return vec4<f32>(sat, col.a);
}
