const PI: f32 = 3.14159265359;

fn ease_in_out_cubic(x: f32) -> f32 {
    let t = clamp(x, 0.0, 1.0);
    return select(1.0 - pow(-2.0 * t + 2.0, 3.0) * 0.5, 4.0 * t * t * t, t < 0.5);
}

fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn rotate_point(p: vec2<f32>, center: vec2<f32>, rad: f32, aspect: f32) -> vec2<f32> {
    var d = p - center;
    d.x *= aspect;
    let s = sin(rad);
    let c = cos(rad);
    var rotated = vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c);
    rotated.x /= aspect;
    return rotated + center;
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let aspect = frame.size.x / max(frame.size.y, 1.0);
    let center = vec2<f32>(0.5, 0.5);
    let t = clamp(progress * 1.01, 0.0, 1.0);
    let bezier = ease_in_out_cubic(t);

    let angle = -bezier * PI * 2.0;
    let rotated_coord = rotate_point(uv, center, angle, aspect);

    let dxy = (rotated_coord - center) * vec2<f32>(aspect, 1.0);
    let r = length(dxy);
    let base_angle = atan2(dxy.y, dxy.x);

    let intensity = select((1.0 - t) * 2.0, t * 2.0, t <= 0.5);
    let step_rad = 0.01 * intensity;

    var sum = vec4<f32>(0.0);
    let num_samples: i32 = 15;

    let blend = smoothstep(0.32, 0.68, t);
    for (var i: i32 = 0; i < num_samples; i++) {
        let cur_angle = base_angle + f32(i) * step_rad;
        let offset = vec2<f32>(r * cos(cur_angle) / aspect, r * sin(cur_angle));
        let sample_uv = clamp(center + offset, vec2<f32>(0.0), vec2<f32>(1.0));
        sum += mix(from_at(sample_uv), to_at(sample_uv), blend);
    }

    return sum / f32(num_samples);
}
