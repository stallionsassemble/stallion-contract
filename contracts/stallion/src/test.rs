#![cfg(test)]

use super::*;
use crate::{Bounty, StallionContract, StallionContractClient};
use soroban_sdk::{Address, Env, String, testutils::Address as _, vec};

fn create_client(e: &Env) -> StallionContractClient {
    StallionContractClient::new(e, &e.register(StallionContract {}, ()))
}

#[test]
fn test() {
    let env = Env::default();
    env.mock_all_auths(); // Add authorization mock

    let client = create_client(&env);
    let owner = Address::generate(&env);

    let reward = 1000;
    let distribution = vec![&env, (1, 50), (2, 50)];
    let submission_deadline = env.ledger().timestamp() + 1000;
    let judging_deadline = submission_deadline + 1000;
    let description = String::from_str(&env, "Test bounty");

    let bounty_id = client.create_bounty(
        &owner,
        &reward,
        &distribution,
        &submission_deadline,
        &judging_deadline,
        &description,
    );

    // Verify the bounty was created correctly
    let bounty: Bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.owner, owner);
    assert_eq!(bounty.reward, reward);
}
