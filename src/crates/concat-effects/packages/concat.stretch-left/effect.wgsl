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

fn mirror_x(x: f32) -> f32 {
    let m = abs(fract(x * 0.5 + 0.5) * 2.0 - 1.0);
    return clamp(m, 0.0, 1.0);
}

fn sample_stretched(uv: vec2<f32>, slide: f32, stretch_x: f32) -> vec4<f32> {
    let shifted_x = (uv.x - 0.5) / max(stretch_x, 0.001) + 0.5 + slide;
    if (shifted_x < 1.0) {
        return from_at(vec2<f32>(mirror_x(shifted_x), uv.y));
    } else {
        return to_at(vec2<f32>(mirror_x(shifted_x - 1.0), uv.y));
    }
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    if (p <= 0.0001) {
        return from_at(uv);
    }
    if (p >= 0.9999) {
        return to_at(uv);
    }

    let slide = solve_bezier_y(vec2<f32>(0.65, 0.0), vec2<f32>(0.15, 1.0), p);
    let pulse = sin(p * 3.14159265);
    let stretch_x = 1.0 + 0.40 * pow(pulse, 1.4);
    let blur_radius = pow(pulse, 1.6) * 0.055;

    var acc = vec4<f32>(0.0);
    for (var i: i32 = -4; i <= 4; i++) {
        let sample_uv = vec2<f32>(uv.x + f32(i) * (blur_radius / 4.0), uv.y);
        acc += sample_stretched(sample_uv, slide, stretch_x);
    }
    return acc / 9.0;
}
