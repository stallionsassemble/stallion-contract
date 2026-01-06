use soroban_sdk::{Address, Env, String, Vec, token};

// Constants
pub const BOUNTY_PLATFORM_FEE_PERCENT: i128 = 5;
pub const GIG_PLATFORM_FEE_PERCENT: i128 = 3;
pub const JOB_PLATFORM_FEE_PERCENT: i128 = 2;

pub enum FeeType {
    Bounty,
    Gig,
    Job,
}

pub fn calculate_fee(reward: i128, fee_type: FeeType) -> i128 {
    match fee_type {
        FeeType::Bounty => reward * BOUNTY_PLATFORM_FEE_PERCENT as i128 / 100,
        FeeType::Gig => reward * GIG_PLATFORM_FEE_PERCENT as i128 / 100,
        FeeType::Job => reward * JOB_PLATFORM_FEE_PERCENT as i128 / 100,
    }
}

pub fn get_token_client(env: &'_ Env, token_address: Address) -> token::Client<'_> {
    token::Client::new(env, &token_address)
}

pub fn get_token_decimals(env: &'_ Env, token_address: &Address) -> u32 {
    let token_client = get_token_client(env, token_address.clone());
    token_client.decimals()
}

pub fn adjust_for_decimals(amount: i128, decimals: u32) -> i128 {
    // Calculate 10^decimals to convert from user-friendly amount to token amount
    let mut multiplier: i128 = 1;
    for _ in 0..decimals {
        multiplier *= 10;
    }
    amount * multiplier
}

pub fn _convert_from_token_amount(amount: i128, decimals: u32) -> i128 {
    // Calculate 10^decimals to convert from token amount to user-friendly amount
    let mut divisor: i128 = 1;
    for _ in 0..decimals {
        divisor *= 10;
    }
    amount / divisor
}

pub fn validate_distribution_sum(distribution: &Vec<(u32, u32)>) -> bool {
    let mut total: u32 = 0;
    for (_, pct) in distribution {
        total += pct;
    }
    total == 100
}

pub fn is_zero_address(env: &Env, addr: &Address) -> bool {
    // The byte representation of a zero address would be all zeros
    addr.to_string()
        == String::from_str(
            env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        )
}
