@group(0) @binding(0)
var<storage, read_write> a: array<f32>;

@group(0) @binding(1)
var<storage, read_write> b: array<f32>;

@group(0) @binding(2)
var<storage, read_write> output: array<f32>;



@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    output[global_id.x] = a[global_id.x] + b[global_id.x];
}
