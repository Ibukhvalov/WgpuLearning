use std::ops::Mul;
use xorshift::{Rng, SeedableRng, Xorshift128};

use clock_ticks::precise_time_ns;

#[repr(C)]
#[derive(Debug)]
pub struct Matrix {
    pub val:  Vec<f32>,
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        //self.val == other.val
        let len = self.val.len();
        if len != other.val.len() { return false; }
        else {
            for i in 0..len {
                if self.val[i] != other.val[i] {
                    if (self.val[i] - other.val[i]).abs() > 0.01 {
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
        let size = self.size();
        for i in 0..size {
            for j in 0..size {
                for k in 0..size {
                    result.val[i * size + j] += self.val[i * size + k] * rhs.val[k * size + j];
                }
            }
        };

        result
    }
}



impl Matrix {
    pub fn new_rand(size: usize) -> Self {
        let now = precise_time_ns();
        let seed = [now, now];
        let mut rng: Xorshift128 = SeedableRng::from_seed(&seed[..]);

        let nb_elements = size*size;
        let mut mat = Self { val: Vec::with_capacity(nb_elements) };

        for _ in 0..nb_elements {
            mat.val.push((rng.next_f32() - 0.5f32) * 100f32);
        }

        mat
    }

    pub fn data_size(&self) -> usize {
        self.val.len() * size_of::<f32>()
    }

    pub fn size(&self) -> usize {
        let size = self.val.len().isqrt();
        assert_eq!(size*size, self.val.len());
        size
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
        let size = self.size();
        for i in 0..size {
            for j in 0..size {
                print!{"{} ", self.val[i*size + j]};
            }
            println!();
        }
        println!();
    }
}

