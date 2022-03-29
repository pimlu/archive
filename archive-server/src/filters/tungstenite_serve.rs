use crate::{session::MpscRtcSession, *};

use anyhow::{Context, Result};
use archive_engine::*;
use futures::StreamExt;
use std::time::Duration;

use log::{debug, error, info};

use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};

const WS_TIMEOUT: Duration = Duration::from_secs(1);

pub async fn tungstenite_serve(arena_map: arena::ArenaMapLock) {
    let addr = "127.0.0.1:8080";

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, arena_map.clone()));
    }
}

async fn accept_connection(stream: TcpStream, arena_map: arena::ArenaMapLock) -> Result<()> {
    let result = accept_connection_inner(stream, arena_map).await;
    if let Err(ref err) = result {
        error!("error accepting ws connection: {err}");
    }
    result
}

async fn accept_connection_inner(stream: TcpStream, arena_map: arena::ArenaMapLock) -> Result<()> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    debug!("attempting ws negotiation");
    info!("New WebSocket connection: {}", addr);

    let ticket = timeout(WS_TIMEOUT, ws_stream.next()).await;

    let ticket = ticket.context("connection timed out")?;
    let ticket = ticket.context("ws read yielded None (socket closed)")?;
    let ticket = ticket.context("ws error while socket is open")?;
    let ticket = ticket.into_data();

    let ticket: rtc::ArenaTicket =
        bincode::deserialize(&ticket[..]).context("failed to parse ticket")?;

    let (client_id, arena_lock) = arena::process_client_ticket(ticket, arena_map.clone()).await?;

    let session = MpscRtcSession::new_from_tungstenite(ws_stream).await?;

    let mut arena = arena_lock.write().await;

    // register this websocket session with the arena which takes over control
    arena.process_client_session(client_id, session.into())?;

    Ok(())
}
