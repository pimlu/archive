use archive_engine::random::{Random, RandomBuilderImpl, RandomImpl};
use rand::prelude::*;

pub struct NativeRandomBuilder {}
struct NativeRandom {
    rng: ThreadRng,
}

impl RandomBuilderImpl for NativeRandomBuilder {
    fn create(&self) -> Random {
        Box::new(NativeRandom { rng: thread_rng() })
    }
}
impl RandomImpl for NativeRandom {
    fn gen(&mut self) -> f64 {
        self.rng.gen()
    }
}
