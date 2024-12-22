struct Matrix {
    val: array<f32>,
}


@group(0) @binding(0)
var<storage, read_write> a: Matrix;

@group(0) @binding(1)
var<storage, read_write> b: Matrix;

@group(0) @binding(2)
var<storage, read_write> output: Matrix;

@group(0) @binding(3)
var<uniform> matrix_size: u32;

fn get_index_by_grid(row: u32, column: u32) -> u32 {
    return row*matrix_size + column;
}


@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var result: f32 = 0.0;

    for(var i: u32 = 0; i < matrix_size; i++) {
        let a_index = get_index_by_grid(global_id.y, i);
        let b_index = get_index_by_grid(i, global_id.x);
        result = result + a.val[a_index] * b.val[b_index];
    }

    let result_index = get_index_by_grid(global_id.y, global_id.x);

    output.val[result_index] = result;
}
