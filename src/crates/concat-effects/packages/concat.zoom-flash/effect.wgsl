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

fn zoom_uv(uv: vec2<f32>, scale: f32) -> vec2<f32> {
    return clamp((uv - vec2<f32>(0.5)) / max(scale, 0.0001) + vec2<f32>(0.5), vec2<f32>(0.0), vec2<f32>(1.0));
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    if (p <= 0.0001) {
        return from_at(uv);
    }
    if (p >= 0.9999) {
        return to_at(uv);
    }

    let pulse = sin(p * 3.14159265);
    let radial_strength = pow(pulse, 1.8) * 0.14;
    let flash_stops = pow(pulse, 2.5) * 3.2;

    let e1 = solve_bezier_y(vec2<f32>(0.55, 0.0), vec2<f32>(0.95, 0.45), clamp(p * 2.0, 0.0, 1.0));
    let scale_a = mix(1.0, 3.5, e1);

    let e2 = solve_bezier_y(vec2<f32>(0.05, 0.55), vec2<f32>(0.35, 1.0), clamp((p - 0.5) * 2.0, 0.0, 1.0));
    let scale_b = mix(1.22, 1.0, e2);

    let mix_ab = smoothstep(0.42, 0.58, p);

    var acc = vec4<f32>(0.0);
    for (var i: i32 = -3; i <= 3; i++) {
        let factor = 1.0 + f32(i) * (radial_strength / 3.0);
        let col_a = from_at(zoom_uv(uv, scale_a * factor));
        let col_b = to_at(zoom_uv(uv, scale_b * factor));
        acc += mix(col_a, col_b, mix_ab);
    }
    let base = acc / 7.0;
    let lit = exposure(base.rgb, flash_stops * 0.45) + vec3<f32>(pow(pulse, 3.2) * 0.18);
    return vec4<f32>(clamp(lit, vec3<f32>(0.0), vec3<f32>(1.0)), base.a);
}
