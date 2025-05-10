use soroban_sdk::{Address, Env, Vec, token};

// Constants
pub const PLATFORM_FEE_PERCENT: u32 = 1;

pub fn calculate_fee(reward: i128) -> i128 {
    reward * PLATFORM_FEE_PERCENT as i128 / 100
}

pub fn get_token_client(env: &Env, token_address: Address) -> token::Client {
    token::Client::new(env, &token_address)
}

pub fn validate_distribution_sum(distribution: &Vec<(u32, u32)>) -> bool {
    let mut total: u32 = 0;
    for (_, pct) in distribution {
        total += pct;
    }
    total == 100
}
