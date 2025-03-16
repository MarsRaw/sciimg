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
    let x = gid.x;
    let y = gid.y;
    if (x >= blur_params.width || y >= blur_params.height) {
        return;
    }
    let idx = y * blur_params.width + x;

    let rad = i32(blur_params.radius);
    let sigma2 = blur_params.sigma * blur_params.sigma;
    let two_sigma2 = 2.0 * sigma2;
    let denom = 3.1415926535 * two_sigma2;

    var sum_weights = 0.0;
    var accum = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    for (var dy = -rad; dy <= rad; dy = dy + 1) {
        for (var dx = -rad; dx <= rad; dx = dx + 1) {
            let sx = clamp(i32(x) + dx, 0, i32(blur_params.width) - 1);
            let sy = clamp(i32(y) + dy, 0, i32(blur_params.height) - 1);
            let dist_x = f32(dx);
            let dist_y = f32(dy);
            let dist2 = (dist_x * dist_x) + (dist_y * dist_y);
            let w = exp(-dist2 / two_sigma2) / denom;
            let sample_idx = (sy * i32(blur_params.width) + sx);
            let sample_color = input_data.data[sample_idx];
            accum = accum + (sample_color * w);
            sum_weights = sum_weights + w;
        }
    }

    output_data.data[idx] = accum / sum_weights;
}
