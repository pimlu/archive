
use log::warn;
use once_cell::sync::OnceCell;


pub trait RandomImpl {
    fn gen(&mut self) -> f64;
    fn gen32(&mut self) -> f32 {
        self.gen() as f32
    }
}

pub type Random = Box<dyn RandomImpl>;

// this spits out non-thread-safe instances of an RNG.
pub trait RandomBuilderImpl: Sync + Send {
    fn create(&self) -> Random;
}

type RandomBuilder = Box<dyn RandomBuilderImpl>;

static RANDOM_BUILDER: OnceCell<RandomBuilder> = OnceCell::new();
pub fn new() -> Random {
    let builder = RANDOM_BUILDER.get().expect("no reigstered random");
    builder.create()
}

// sets the global builder for RNGs.
pub fn register(builder: impl RandomBuilderImpl + 'static) {
    let res = RANDOM_BUILDER.set(Box::new(builder));
    if res.is_err() {
        warn!("already registered random");
    }
}
