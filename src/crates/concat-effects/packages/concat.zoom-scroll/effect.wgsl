fn mirror_uv(p: vec2<f32>) -> vec2<f32> {
    return abs(fract(p * 0.5 + 0.5) * 2.0 - 1.0);
}

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);

    let bell = 1.0 - abs(2.0 * p - 1.0);
    let blur_amt = bell * 0.025;
    let scale = 1.0 + bell * 0.22;
    let slide = smoothstep(0.0, 1.0, p);

    var sum = vec4<f32>(0.0);
    let taps: i32 = 7;

    for (var i: i32 = -3; i <= 3; i++) {
        let offset_y = f32(i) * blur_amt;
        let centered = (uv - vec2<f32>(0.5, 0.5)) / scale + vec2<f32>(0.5, 0.5);
        let sx = clamp(centered.x, 0.0, 1.0);
        let sy = centered.y + slide + offset_y;

        if (sy < 1.0) {
            sum += from_at(vec2<f32>(sx, clamp(sy, 0.0, 1.0)));
        } else {
            sum += to_at(vec2<f32>(sx, clamp(sy - 1.0, 0.0, 1.0)));
        }
    }

    return sum / f32(taps);
}
