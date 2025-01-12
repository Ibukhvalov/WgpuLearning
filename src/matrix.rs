use std::ops::Mul;
use crate::MATRIX_SIZE;
use rand::{thread_rng, Rng};


#[repr(C)]
#[derive(Debug)]
pub struct Matrix {
    pub val: Vec<f32>,
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        let len = self.val.len();
        if len != other.val.len() { return false; }
        else {
            for i in 0..len {
                if self.val[i] != other.val[i] {
                    if (self.val[i] - other.val[i]).abs() > 0.001 {
                        log::debug!("{} {}", self.val[i], other.val[i]);
                        return false;
                    }
                }
            }
        }
        return true;
        
    }
}

impl Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Self{val: vec![0.0; self.val.len()]};
        
        for i in 0..MATRIX_SIZE {
            for j in 0..MATRIX_SIZE {
                for k in 0..MATRIX_SIZE {
                    result.val[i * MATRIX_SIZE + j] += self.val[i * MATRIX_SIZE + k] * rhs.val[k * MATRIX_SIZE + j];
                }
            }
        };

        result
    }
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

