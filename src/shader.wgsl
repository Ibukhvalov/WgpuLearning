
@group(0)
@binding(0)
var <storage, read_write> v_indices: array<u32>;

fn evenOne(x: u32) -> u32 {
    if(x%2 == 0) {return 1u;}
    return 0u;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    v_indices[global_id.x] = evenOne(v_indices[global_id.x]);
}