use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientOffer {
    pub sdp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAnswer {
    pub sdp: String,
}
