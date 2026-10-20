fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    let incoming_uv = uv + vec2<f32>(0.0, 1.0 - p);
    if (incoming_uv.y <= 1.0) {
        return to_at(incoming_uv);
    }
    return from_at(uv - vec2<f32>(0.0, p));
}
