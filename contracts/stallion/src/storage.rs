use crate::types::DataKey;

pub fn next_id_key() -> DataKey {
    DataKey::NextId
}

pub fn bounty_key(id: u64) -> (DataKey, u64) {
    (DataKey::Bounty, id)
}

pub fn admin_key() -> DataKey {
    DataKey::Admin
}

pub fn fee_account_key() -> DataKey {
    DataKey::FeeAccount
}

pub fn next_project_id_key() -> DataKey {
    DataKey::NextProjectId
}

pub fn project_key(id: u64) -> (DataKey, u64) {
    (DataKey::Project, id)
}

pub fn next_hackathon_id_key() -> DataKey {
    DataKey::NextHackathonId
}

pub fn hackathon_key(id: u64) -> (DataKey, u64) {
    (DataKey::Hackathon, id)
}

pub fn deployment_seq_key() -> DataKey {
    DataKey::DeploymentSeq
}
