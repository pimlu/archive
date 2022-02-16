use crate::*;

use hecs::*;

pub(super) type ReplPool = token_pool::TokenPool;
pub(super) type ReplToken = token_pool::PoolToken;
pub(super) type ReplKey = token_pool::PoolKey;

pub(super) type Priority = u32;

derive_components! {
    pub struct Replicated {
        pub(super) blueprint: Option<Blueprint>
    }
}

pub(super) trait ReplicationMap {
    fn entity_for_token(&self, token: &ReplToken) -> Entity;
}
