fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);

    let flash = (1.0 - abs(2.0 * p - 1.0)) * 0.22;
    let blur_amt = (1.0 - abs(2.0 * p - 1.0)) * 0.025;
    let slide = smoothstep(0.0, 1.0, p);
    let scale = 1.0 + (1.0 - abs(2.0 * p - 1.0)) * 0.18;

    var sum = vec4<f32>(0.0);
    let taps: i32 = 7;

    for (var i: i32 = -3; i <= 3; i++) {
        let offset_x = f32(i) * blur_amt;
        let centered = (uv - vec2<f32>(0.5, 0.5)) / scale + vec2<f32>(0.5, 0.5);
        let sx = centered.x + slide + offset_x;
        let sy = clamp(centered.y, 0.0, 1.0);
        if (sx < 1.0) {
            sum += from_at(vec2<f32>(clamp(sx, 0.0, 1.0), sy));
        } else {
            sum += to_at(vec2<f32>(clamp(sx - 1.0, 0.0, 1.0), sy));
        }
    }

    var result = sum / f32(taps);
    result = vec4<f32>(result.rgb + vec3<f32>(flash), result.a);
    return vec4<f32>(clamp(result.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), result.a);
}
