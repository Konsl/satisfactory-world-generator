use std::ops::Range;

#[derive(Debug)]
pub struct RandomStream(u32);

impl RandomStream {
    pub fn new(seed: i32) -> Self {
        Self(seed.cast_unsigned())
    }

    fn mutate_seed(&mut self) {
        self.0 = (self.0.wrapping_mul(196314165)).wrapping_add(907633515);
    }

    fn get_fraction(&mut self) -> f32 {
        self.mutate_seed();
        f32::from_bits(0x3F800000 | (self.0 >> 9)) - 1.0
    }

    pub fn frand(&mut self) -> f32 {
        self.get_fraction()
    }

    pub fn frand_range(&mut self, range: Range<f32>) -> f32 {
        range.start + (range.end - range.start) * self.frand()
    }
}
