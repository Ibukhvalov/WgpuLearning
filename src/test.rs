
#[test]
fn matmul() {
    use super::*;
    env_logger::builder()
    .filter_module("shader_learning", log::LevelFilter::Debug)
    .init();

    info!("Generating matrix data");

    


    //let a = Matrix::new_rand(MATRIX_SIZE);
    //let b = Matrix::new_rand(MATRIX_SIZE);

    let a = Matrix {
        val: vec![7.817355, 4.669319, 8.464355,
        4.4331923, 6.8329406, 3.967669,
        1.0603511, 4.3951917, 3.1678343],
    };

    let b = Matrix {
        val: vec![7.817355, 4.669319, 8.464355,
        4.4331923, 6.8329406, 3.967669,
        1.0603511, 4.3951917, 3.1678343],
    };
    
    a.print();
    b.print();

    let result_gpu = pollster::block_on(execute_gpu(&a,&b)).unwrap();

    let result_cpu = a*b;

    assert_eq!(result_cpu, result_gpu);   
}

