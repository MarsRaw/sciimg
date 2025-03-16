
//NOTE: Keep up to date with the sister struct on the rust side.
struct GaussianBlurUniform {
    radius : u32,
    sigma  : f32,
    width  : u32,
    height : u32,
};

@group(0) @binding(0) var<uniform> blur_params: GaussianBlurUniform;
@group(0) @binding(100) var<storage, read>  input_data:  array<vec3<f32>>;
@group(1) @binding(100) var<storage, read_write> output_data: array<vec3<f32>>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= blur_params.width || y >= blur_params.height {
        return;
    }

    // We do a simple 2D Gaussian around (x, y).
    let idx = y * blur_params.width + x;

    let rad = i32(blur_params.radius);
    let sigma2 = blur_params.sigma * blur_params.sigma;
    let two_sigma2 = 2.0 * sigma2;
    let denom = 3.1415926535 * two_sigma2; // π * 2σ² //TODO: conts...

    var sum_weights = 0.0;
    var accum = vec3<f32>(0.0, 0.0, 0.0);

    for (var dy = -rad; dy <= rad; dy = dy + 1) {
        for (var dx = -rad; dx <= rad; dx = dx + 1) {
            let sample_x = clamp(x + u32(max(dx, 0)), 0u, blur_params.width  - 1u);
            let sample_y = clamp(y + u32(max(dy, 0)), 0u, blur_params.height - 1u);

            // actual offset distance:
            let dist_x = f32(dx);
            let dist_y = f32(dy);
            let dist2 = (dist_x * dist_x) + (dist_y * dist_y);

            // Gaussian weight
            let w = exp(-dist2 / two_sigma2) / denom;

            let sample_idx = sample_y * blur_params.width + sample_x;
            let sample_color = input_data[sample_idx];
            accum = accum + (sample_color * w);
            sum_weights = sum_weights + w;
        }
    }

    output_data[idx] = accum / sum_weights;
}
