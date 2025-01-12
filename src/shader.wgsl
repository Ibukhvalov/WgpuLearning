struct Matrix {
    val: array<f32>,
};

// must equals tile_size in main.rs
const TILE_SIZE: u32 = 2;

@group(0) @binding(0)
var<storage, read> a: Matrix;

@group(0) @binding(1)
var<storage, read> b: Matrix;

@group(0) @binding(2)
var<storage, read_write> output: Matrix;

@group(0) @binding(3)
var<uniform> matrix_size: u32;

var<workgroup> tile_a: array<array<f32, TILE_SIZE>, TILE_SIZE>; 
var<workgroup> tile_b: array<array<f32, TILE_SIZE>, TILE_SIZE>; 

fn get_index_by_grid(row: u32, column: u32) -> u32 {
    return row*matrix_size + column;
}



@compute
@workgroup_size(TILE_SIZE, TILE_SIZE)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {    
    var result = 0.0;

    for(var k: u32 = 0; k < matrix_size; k += TILE_SIZE) {        
        let global_row = u32(global_id.y);
        let global_col = u32(global_id.x);

        let local_row = u32(local_id.y);
        let local_col = u32(local_id.x);

        let global_a = get_index_by_grid(global_row, k + local_col);
        let global_b = get_index_by_grid(k + local_row, global_col);

        // select(false, true, condition);
        tile_a[local_row][local_col] =  select(0.0, a.val[global_a], k + local_col < matrix_size);
        tile_b[local_row][local_col] =  select(0.0, b.val[global_b], k + local_row < matrix_size);
    
        workgroupBarrier();

        for(var i: u32 = 0; i < TILE_SIZE; i++) {
            result += tile_a[local_row][i] * tile_b[i][local_col];
        }

        workgroupBarrier();
    }

    let result_index = get_index_by_grid(global_id.y, global_id.x);
    output.val[result_index] = result;
    
}
