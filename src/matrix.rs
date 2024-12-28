use log::debug;
use crate::MATRIX_SIZE;
use rand::{thread_rng, Rng};


#[repr(C)]
pub struct Matrix {
    pub val: Vec<f32>,
}



impl Matrix {
    pub fn new_rand(dim_size: usize) -> Self {
        let num_of_el = dim_size*dim_size;

        let mut rng = thread_rng();

        let mut mat = Self { val: Vec::with_capacity(num_of_el) };

        for _ in 0..num_of_el {
            mat.val.push(rng.gen_range(0f32..10f32));
        }

        mat
    }

    pub fn data_size(&self) -> usize {
        self.val.len() * size_of::<f32>()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {

        let unit_size = size_of::<f32>();

        if bytes.len() % unit_size != 0 {
            return Err(format!("Количество байтов должно быть кратно {}", unit_size))
        }

        let mut mat = Self { val: Vec::with_capacity(bytes.len() / unit_size) };

        for chunk in bytes.chunks(unit_size) {
            let float_bytes = chunk.try_into().unwrap();
            let float_val = f32::from_le_bytes(float_bytes);

            mat.val.push(float_val);
        }

        Ok(mat)
    }


    #[warn(dead_code)]
    pub fn print(&self) {
        let size = MATRIX_SIZE;
        for i in 0..size {
            for j in 0..size {
                print!{"{} ", self.val[i*size + j]};
            }
            println!();
        }
        println!();
    }
}

