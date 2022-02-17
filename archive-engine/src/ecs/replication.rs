use super::*;
use crate::*;

use hecs::*;

pub(super) type ReplPool = containers::TokenPool;
pub(super) type ReplToken = containers::PoolToken;
pub(super) type ReplKey = containers::PoolKey;

pub(super) type Priority = u32;

derive_components! {
    pub struct Replicated {
        pub(super) blueprint: Option<Blueprint>
    }
}

pub(super) trait ReplicationMap {
    fn entity_for_token(&self, token: &ReplToken) -> Entity;
}
