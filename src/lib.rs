pub mod matrix;
pub mod gpu_exec;

#[cfg(test)]
mod tests {
    use log::{info, debug};
    use super::{matrix::*, gpu_exec::*};
    use std::fs;
    use safetensors::SafeTensors;

    #[test]
    fn tensor_matmul() {
        env_logger::builder()
        .filter_module("shader_learning", log::LevelFilter::Debug)
        .init();
        info!("Reading tensors");
        let data = fs::read("./data/matmul_data.safetensors").expect("First generate a data using python script");
        let tensors = SafeTensors::deserialize(&data).expect("Unable to deserialize data from file");
        info!("Parsing tensors to matrixes");
        let a = Matrix::from_bytes(tensors.tensor("A").unwrap().data()).unwrap();
        let b = Matrix::from_bytes(tensors.tensor("B").unwrap().data()).unwrap();
        let c = Matrix::from_bytes(tensors.tensor("C").unwrap().data()).unwrap();

        let result_gpu = pollster::block_on(execute_gpu(&a,&b)).unwrap();
        debug!("First 100 elements are {:?}", &c.val[..std::cmp::min(100, c.val.len())]);
        assert_eq!(c, result_gpu);
    }
}
