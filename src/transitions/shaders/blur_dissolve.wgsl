// SPDX-License-Identifier: AGPL-3.0-or-later
// CapCut Clone: "Sfocatura" (Blur Dissolve) - ID: 6916426617455645186
// is_overlap: false

const PI: f32 = 3.14159265359;

fn transition(uv: vec2<f32>, progress: f32) -> vec4<f32> {
    let p = clamp(progress, 0.0, 1.0);
    let mix_factor = smoothstep(0.5, 0.9, p);

    let blur_factor = sin(PI * p) * 1.72 * p;
    let unit_uv = (3.75 / max(frame.size, vec2<f32>(1.0, 1.0))) * blur_factor;

    // Standard 9-tap 1D Gaussian kernel decomposed to symmetric weights
    // w[0]=0.227027, w[1]=0.194595, w[2]=0.121622, w[3]=0.054054, w[4]=0.016216
    let w0 = 0.2270270270;
    let w1 = 0.1945945946;
    let w2 = 0.1216216216;
    let w3 = 0.0540540541;
    let w4 = 0.0162162162;

    // Cross-separable sample grid centered at uv
    var sum = mix(from_at(uv), to_at(uv), mix_factor) * w0;

    // Horizontal taps
    let h1_r = mix(from_at(uv + vec2<f32>(1.0 * unit_uv.x, 0.0)), to_at(uv + vec2<f32>(1.0 * unit_uv.x, 0.0)), mix_factor);
    let h1_l = mix(from_at(uv - vec2<f32>(1.0 * unit_uv.x, 0.0)), to_at(uv - vec2<f32>(1.0 * unit_uv.x, 0.0)), mix_factor);
    let h2_r = mix(from_at(uv + vec2<f32>(2.0 * unit_uv.x, 0.0)), to_at(uv + vec2<f32>(2.0 * unit_uv.x, 0.0)), mix_factor);
    let h2_l = mix(from_at(uv - vec2<f32>(2.0 * unit_uv.x, 0.0)), to_at(uv - vec2<f32>(2.0 * unit_uv.x, 0.0)), mix_factor);
    let h3_r = mix(from_at(uv + vec2<f32>(3.0 * unit_uv.x, 0.0)), to_at(uv + vec2<f32>(3.0 * unit_uv.x, 0.0)), mix_factor);
    let h3_l = mix(from_at(uv - vec2<f32>(3.0 * unit_uv.x, 0.0)), to_at(uv - vec2<f32>(3.0 * unit_uv.x, 0.0)), mix_factor);
    let h4_r = mix(from_at(uv + vec2<f32>(4.0 * unit_uv.x, 0.0)), to_at(uv + vec2<f32>(4.0 * unit_uv.x, 0.0)), mix_factor);
    let h4_l = mix(from_at(uv - vec2<f32>(4.0 * unit_uv.x, 0.0)), to_at(uv - vec2<f32>(4.0 * unit_uv.x, 0.0)), mix_factor);

    // Vertical taps
    let v1_d = mix(from_at(uv + vec2<f32>(0.0, 1.0 * unit_uv.y)), to_at(uv + vec2<f32>(0.0, 1.0 * unit_uv.y)), mix_factor);
    let v1_u = mix(from_at(uv - vec2<f32>(0.0, 1.0 * unit_uv.y)), to_at(uv - vec2<f32>(0.0, 1.0 * unit_uv.y)), mix_factor);
    let v2_d = mix(from_at(uv + vec2<f32>(0.0, 2.0 * unit_uv.y)), to_at(uv + vec2<f32>(0.0, 2.0 * unit_uv.y)), mix_factor);
    let v2_u = mix(from_at(uv - vec2<f32>(0.0, 2.0 * unit_uv.y)), to_at(uv - vec2<f32>(0.0, 2.0 * unit_uv.y)), mix_factor);
    let v3_d = mix(from_at(uv + vec2<f32>(0.0, 3.0 * unit_uv.y)), to_at(uv + vec2<f32>(0.0, 3.0 * unit_uv.y)), mix_factor);
    let v3_u = mix(from_at(uv - vec2<f32>(0.0, 3.0 * unit_uv.y)), to_at(uv - vec2<f32>(0.0, 3.0 * unit_uv.y)), mix_factor);
    let v4_d = mix(from_at(uv + vec2<f32>(0.0, 4.0 * unit_uv.y)), to_at(uv + vec2<f32>(0.0, 4.0 * unit_uv.y)), mix_factor);
    let v4_u = mix(from_at(uv - vec2<f32>(0.0, 4.0 * unit_uv.y)), to_at(uv - vec2<f32>(0.0, 4.0 * unit_uv.y)), mix_factor);

    let h_sum = sum + (h1_r + h1_l) * w1 + (h2_r + h2_l) * w2 + (h3_r + h3_l) * w3 + (h4_r + h4_l) * w4;
    let v_sum = sum + (v1_d + v1_u) * w1 + (v2_d + v2_u) * w2 + (v3_d + v3_u) * w3 + (v4_d + v4_u) * w4;

    return (h_sum + v_sum) * 0.5;
}
