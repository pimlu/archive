pub mod arena;
pub mod filters;
pub mod session;

#[tokio::main]
async fn main() {
    filters::warp_main().await
}
