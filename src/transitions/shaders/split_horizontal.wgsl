// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Divisione II" (Split II - Horizontal Split) - ID: 6973849510145430018
// is_overlap: false

fn solve_bezier_y(p1: vec2<f32>, p2: vec2<f32>, x: f32) -> vec2<f32> {
    let cx = 3.0 * p1.x;
    let bx = 3.0 * (p2.x - p1.x) - cx;
    let ax = 1.0 - cx - bx;

    let cy = 3.0 * p1.y;
    let by = 3.0 * (p2.y - p1.y) - cy;
    let ay = 1.0 - cy - by;

    // Newton-Raphson iteration to find parameter t where Bx(t) == x
    var t = clamp(x, 0.0, 1.0);
    for (var i: i32 = 0; i < 5; i++) {
        let current_x = ((ax * t + bx) * t + cx) * t;
        let dx = (3.0 * ax * t + 2.0 * bx) * t + cx;
        if (abs(dx) < 1e-5) { break; }
        t = clamp(t - (current_x - x) / dx, 0.0, 1.0);
    }

    let y = ((ay * t + by) * t + cy) * t;
    let dy = (3.0 * ay * t + 2.0 * by) * t + cy;
    // Returns vec2(y_value, derivative_dy_dt)
    return vec2<f32>(clamp(y, 0.0, 1.0), max(dy, 0.0));
}

fn sample_half_split(uv: vec2<f32>, temp: f32) -> vec3<f32> {
    let flag = step(uv.y, 0.5); // 1.0 if bottom half (y < 0.5), 0.0 if top half
    let effective_temp = temp * flag + (1.0 - temp) * (1.0 - flag);

    var uv1 = uv;
    uv1.x += (-effective_temp) * flag + (1.0 - effective_temp) * (1.0 - flag);
    let col1 = from_at(uv1).rgb;

    var uv2 = uv;
    uv2.x += (1.0 - effective_temp) * flag + (-effective_temp) * (1.0 - flag);
    let col2 = to_at(uv2).rgb;

    return mix(col1, col2, step(uv.x, effective_temp)) * flag +
           mix(col2, col1, step(uv.x, effective_temp)) * (1.0 - flag);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    // CapCut Bézier curve controls
    let bezier1 = vec4<f32>(0.79, 0.00, 0.17, 0.96); // Top half (y >= 0.5)
    let bezier2 = vec4<f32>(0.77, 0.06, 0.07, 0.99); // Bottom half (y < 0.5)

    let res1 = solve_bezier_y(bezier1.xy, bezier1.zw, progress);
    let res2 = solve_bezier_y(bezier2.xy, bezier2.zw, progress);

    let is_top = step(0.5, uv.y);
    let temp = res1.x * is_top + res2.x * (1.0 - is_top);
    let velocity = res1.y * is_top + res2.y * (1.0 - is_top);

    // 9-tap horizontal motion blur along dir = (-1.0, 0.0)
    var m = vec3<f32>(0.0);
    let dir = vec2<f32>(-1.0, 0.0);
    let blur_step = velocity * 0.002;

    for (var i: i32 = 0; i < 9; i++) {
        let sample_uv = uv + f32(i) * blur_step * dir;
        m += sample_half_split(sample_uv, temp);
    }

    return vec4<f32>(m / 9.0, 1.0);
}
