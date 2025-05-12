use crate::types::DataKey;

pub fn next_id_key() -> DataKey {
    DataKey::NextId
}

pub fn bounty_key(id: u32) -> (DataKey, u32) {
    (DataKey::Bounty, id)
}

pub fn admin_key() -> DataKey {
    DataKey::Admin
}

pub fn fee_account_key() -> DataKey {
    DataKey::FeeAccount
}
