struct Params { amplitude: f32, speed: f32 }

fn effect(uv: vec2<f32>) -> vec4<f32> {
    let order = reveal_order(uv);
    let phase = frame.clip_time * params.speed * 6.2831853 + order * 6.2831853;
    let shifted = uv + vec2<f32>(0.0, sin(phase) * params.amplitude);
    return sample(shifted);
}
