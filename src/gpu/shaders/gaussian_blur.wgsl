struct GaussianBlurUniform {
    radius : u32,
    sigma  : f32,
    width  : u32,
    height : u32,
};

struct GpuImg{
    length: u32,
    data: array<vec4<f32>>,
};

@group(0) @binding(0) var<uniform> blur_params: GaussianBlurUniform;
@group(0) @binding(1) var<storage, read> input_data: GpuImg;
@group(1) @binding(0) var<storage, read_write>  output_data: GpuImg;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    output_data.length = input_data.length;

    let x = i32(gid.x);
    let y = i32(gid.y);
    let w = i32(blur_params.width);
    let h = i32(blur_params.height);

    if (x >= w || y >= h) { return; }

    // CPU logic uses radius ~ 3*sigma.
    //TODO: move this 'sigma' multiplier into the params.
    // let rad = max(1, i32(round(3.0 * blur_params.sigma)));
    let rad = i32(blur_params.radius);
    let sigma2 = blur_params.sigma * blur_params.sigma;
    let two_sigma2 = 2.0 * sigma2;
    // For a normalised 2D Gaussian: divisor = pi * 2 * sigma^2, i.e. TAU * sigma^2
    let denom = 6.283185307 * sigma2; // ~ std::f32::consts::TAU * sigma^2

    var sum_weights = 0.0;
    var accum = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    for (var dy = -rad; dy <= rad; dy = dy + 1) {
        for (var dx = -rad; dx <= rad; dx = dx + 1) {
            let sx = clamp(x + dx, 0, w - 1);
            let sy = clamp(y + dy, 0, h - 1);
            let sample_idx = u32(sy) * blur_params.width + u32(sx);

            let dist2 = f32(dx * dx + dy * dy);
            let w_gauss = exp(-dist2 / two_sigma2) / denom;

            accum += input_data.data[sample_idx] * w_gauss;
            sum_weights += w_gauss;
        }
    }

    // Normalise
    output_data.data[u32(y) * blur_params.width + u32(x)] = accum / sum_weights;
}
