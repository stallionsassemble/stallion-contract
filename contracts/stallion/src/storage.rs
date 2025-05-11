use crate::types::DataKey;

pub fn token_key() -> DataKey {
    DataKey::Token
}

pub fn next_id_key() -> DataKey {
    DataKey::NextId
}

pub fn bounty_key(id: u32) -> (DataKey, u32) {
    (DataKey::Bounty, id)
}
