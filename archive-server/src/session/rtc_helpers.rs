// don't use tokio constructs in the browser/traits that are used in the browser
pub fn map_try_recv_to_std(
    e: tokio::sync::mpsc::error::TryRecvError,
) -> std::sync::mpsc::TryRecvError {
    match e {
        tokio::sync::mpsc::error::TryRecvError::Empty => std::sync::mpsc::TryRecvError::Empty,
        tokio::sync::mpsc::error::TryRecvError::Disconnected => {
            std::sync::mpsc::TryRecvError::Disconnected
        }
    }
}
