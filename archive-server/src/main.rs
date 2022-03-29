use tokio::select;

pub mod arena;
pub mod filters;
pub mod session;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arena_map = arena::ArenaMapLock::default();

    select! {
        _ = filters::warp_serve(arena_map.clone()) => {},
        _ = filters::tungstenite_serve(arena_map.clone()) => {}
    };
}
