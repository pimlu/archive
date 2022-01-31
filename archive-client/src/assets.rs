#[macro_export]
macro_rules! include_asset {
    ($path:expr) => {
        include_bytes!(concat!("../assets/", $path))
    };
}
