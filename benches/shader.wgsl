@group(0) @binding(0) var<storage, read> input : array<f32>;
@group(0) @binding(1) var<storage, write> output : array<f32>;

 @compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let width = {width}u;
    let index = global_id.x + global_id.y * width;
    if (index >= arrayLength(&input_data)) {{
        return;
    }}
    
    output_data[index] = input_data[index] * 1.1;
}}