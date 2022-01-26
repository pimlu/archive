use archive_engine::random::{Random, RandomBuilderImpl, RandomImpl};

pub struct WasmRandomBuilder {}
pub struct WasmRandom {}

impl RandomBuilderImpl for WasmRandomBuilder {
    fn create(&self) -> Random {
        Box::new(WasmRandom {})
    }
}
impl RandomImpl for WasmRandom {
    fn gen(&mut self) -> f64 {
        js_sys::Math::random()
    }
}
