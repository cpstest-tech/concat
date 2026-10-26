struct Params { duration: f32, start: f32, steps: f32, snap: f32, reverse: f32 }

// The title painter supplies a map whose red channel is the reading-order
// index of each word. Sampling it here keeps the original glyph raster and
// layout intact while making the reveal deterministic at every frame.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    let duration = max(params.duration, 0.001);
    let progress = clamp((frame.clip_time - params.start) / duration, 0.0, 1.0);
    var threshold = progress * 1.02 - 0.01;
    if params.snap > 0.5 && params.steps > 1.0 {
        if params.reverse > 0.5 {
            threshold = select(1.0 - floor(progress * params.steps) / (params.steps - 1.0), -0.001, progress >= 1.0);
        } else {
            threshold = select(floor(progress * params.steps) / (params.steps - 1.0) - 0.001, 1.0, progress >= 1.0);
        }
    } else if params.reverse > 0.5 {
        threshold = 1.02 - progress * 1.03;
    }
    let visible = reveal_order(uv) <= threshold;
    return select(vec4<f32>(0.0), c, visible);
}
