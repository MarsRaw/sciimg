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

    // ...
    // Gaussian blur logic: sample around (x,y) within radius,
    // apply weights using blur_params.sigma, accumulate, etc.
    // ...



    let idx = y * blur_params.width + x;
    output_data[idx] = input_data[idx];

   
}
