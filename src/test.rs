
#[test]
fn matmul() {
    use super::*;
    env_logger::builder()
    .filter_module("shader_learning", log::LevelFilter::Debug)
    .init();

    info!("Generating matrix data");

    


    let a = Matrix::new_rand(MATRIX_SIZE);
    let b = Matrix::new_rand(MATRIX_SIZE);

    a.print();
    b.print();

    let result_gpu = pollster::block_on(execute_gpu(&a,&b)).unwrap();

    if MATRIX_SIZE<250 {
        info!("Start computing on cpu");
        let result_cpu = a*b;
        assert_eq!(result_cpu, result_gpu);   
    }

    info!("Cpu computing skipped (matrix size should be less than 250) ");

}

