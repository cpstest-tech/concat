fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn rotate_stretch(uv: vec2<f32>, angle_rad: f32, scale_x: f32) -> vec2<f32> {
    let center = vec2<f32>(0.5, 0.5);
    var d = uv - center;
    d.x /= max(scale_x, 0.001);
    let s = sin(angle_rad);
    let c = cos(angle_rad);
    let rotated = vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c);
    return rotated + center;
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    let cut_point = 0.645;

    let flash = smoothstep(0.55, 0.645, p) * (1.0 - smoothstep(0.645, 0.77, p));

    var col = vec4<f32>(0.0);
    let taps: i32 = 7;

    let blend = smoothstep(0.32, 0.68, p);
    let t_a = clamp(p / cut_point, 0.0, 1.0);
    let eased_a = smoothstep(0.0, 1.0, t_a);
    let angle_a = eased_a * 0.32;
    let scale_x_a = mix(1.0, 1.65, eased_a);

    let t_b = clamp((p - 0.32) / 0.68, 0.0, 1.0);
    let eased_b = smoothstep(0.0, 1.0, t_b);
    let angle_b = mix(-0.26, 0.0, eased_b);
    let scale_x_b = mix(1.65, 1.0, eased_b);

    let blur_amt = (1.0 - abs(2.0 * p - 1.0)) * 0.015;
    let dir_a = vec2<f32>(cos(angle_a), sin(angle_a));
    let dir_b = vec2<f32>(cos(angle_b), sin(angle_b));

    for (var i: i32 = -3; i <= 3; i++) {
        let off_a = dir_a * (f32(i) * blur_amt);
        let off_b = dir_b * (f32(i) * blur_amt);
        let uv_a = clamp(rotate_stretch(uv + off_a, angle_a, scale_x_a), vec2<f32>(0.0), vec2<f32>(1.0));
        let uv_b = clamp(rotate_stretch(uv + off_b, angle_b, scale_x_b), vec2<f32>(0.0), vec2<f32>(1.0));
        col += mix(from_at(uv_a), to_at(uv_b), blend);
    }

    var result = col / f32(taps);
    result = vec4<f32>(result.rgb + vec3<f32>(flash * 0.5), result.a);
    return vec4<f32>(clamp(result.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), result.a);
}
