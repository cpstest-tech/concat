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

fn mirror_uv(uv: vec2<f32>) -> vec2<f32> {
    let m = abs(fract(uv * 0.5 + vec2<f32>(0.5)) * 2.0 - vec2<f32>(1.0));
    return clamp(m, vec2<f32>(0.0), vec2<f32>(1.0));
}

fn zoom_uv(uv: vec2<f32>, scale: f32) -> vec2<f32> {
    return (uv - vec2<f32>(0.5)) / max(scale, 0.0001) + vec2<f32>(0.5);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    if (p <= 0.0001) {
        return from_at(uv);
    }
    if (p >= 0.9999) {
        return to_at(uv);
    }

    var split_dist: f32;
    var to_scale: f32;
    if (p <= 0.5) {
        let e1 = solve_bezier_y(vec2<f32>(0.2, 0.0), vec2<f32>(0.2, 1.0), p * 2.0);
        split_dist = mix(0.0, 0.185, e1);
        to_scale = mix(0.40, 0.70, e1);
    } else {
        let e2 = solve_bezier_y(vec2<f32>(0.8, 0.0), vec2<f32>(0.2, 1.0), (p - 0.5) * 2.0);
        split_dist = mix(0.185, 0.75, e2);
        to_scale = mix(0.70, 1.00, e2);
    }

    let diag = ((uv.x - 0.5) + (uv.y - 0.5)) * 0.70710678;
    let shift = vec2<f32>(split_dist * 0.70710678, split_dist * 0.70710678);

    if (diag >= split_dist) {
        return from_at(clamp(uv - shift, vec2<f32>(0.0), vec2<f32>(1.0)));
    }
    if (diag <= -split_dist) {
        return from_at(clamp(uv + shift, vec2<f32>(0.0), vec2<f32>(1.0)));
    }

    let bg_uv = mirror_uv(zoom_uv(uv, to_scale));
    let bg_col = to_at(bg_uv);

    let edge_dist = split_dist - abs(diag);
    let shadow = (1.0 - smoothstep(0.0, 0.065, edge_dist)) * 0.55;
    return vec4<f32>(bg_col.rgb * (1.0 - shadow), bg_col.a);
}
