#![cfg(test)]

extern crate std;

use crate::{StallionContract, StallionContractClient, Status};
use soroban_sdk::{
    Address, Env, String, Symbol,
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec,
};

fn create_token_contract(e: &Env) -> (TokenClient, Address) {
    e.mock_all_auths();

    let issuer = Address::generate(&e);
    let distributor = Address::generate(&e);

    let sac = e.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();

    // client for SEP-41 functions
    let token = TokenClient::new(&e, &token_address);
    // client for Stellar Asset Contract functions
    let token_sac = StellarAssetClient::new(&e, &token_address);

    // note that you need to account for the difference between the minimal
    // unit and the unit itself when working with amounts.
    // E.g. to mint 1 TOKEN, we need to use 1*1e7 in the mint function.
    let genesis_amount: i128 = 1_000_000_000 * 1_000_000_000;

    // Mint initial supply
    token_sac.mint(&distributor, &genesis_amount);

    // Make issuer AuthRequired and AuthRevocable
    // sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    // sac.issuer().set_flag(IssuerFlags::RequiredFlag);

    (token, distributor)
}

fn setup_test(env: &Env) -> (StallionContractClient, TokenClient, Address) {
    let (token, distributor) = create_token_contract(&env);
    let contract_id = env.register(StallionContract {}, (token.address.clone(),));
    let client = StallionContractClient::new(&env, &contract_id);
    (client, token, distributor)
}

#[test]
fn test_bounty_creation() {
    let env = Env::default();
    let (client, token, distributor) = setup_test(&env);
    env.mock_all_auths();

    let owner = Address::generate(&env);
    token.transfer(&distributor, &owner, &1000);

    // Test valid bounty creation
    let bounty_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, Status::Active);

    // Test invalid distribution
    let result = client.try_create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 60), (2, 30)], // Only adds to 90%
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );
    assert!(result.is_err());

    // Test invalid deadlines
    let result = client.try_create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 2000),
        &(env.ledger().timestamp() + 1000), // Judging before submission
        &String::from_str(&env, "Test bounty"),
    );
    assert!(result.is_err());
}

#[test]
fn test_bounty_submissions() {
    let env = Env::default();
    let (client, token, distributor) = setup_test(&env);
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let applicant = Address::generate(&env);
    token.transfer(&distributor, &owner, &1000);

    // Create bounty
    let bounty_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Test valid submission
    client.apply_to_bounty(&applicant, &bounty_id, &Symbol::new(&env, "link1"));

    // Test submission update
    client.apply_to_bounty(&applicant, &bounty_id, &Symbol::new(&env, "link2"));

    let submissions = client.get_bounty_submissions(&bounty_id);
    assert_eq!(
        submissions.get(applicant.clone()).unwrap(),
        Symbol::new(&env, "link2")
    );

    // Test submission after deadline
    env.ledger().set_timestamp(env.ledger().timestamp() + 1001);
    let result = client.try_apply_to_bounty(&applicant, &bounty_id, &Symbol::new(&env, "link3"));
    assert!(result.is_err());
}

#[test]
fn test_winner_selection() {
    let env = Env::default();
    let (client, token, distributor) = setup_test(&env);
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);
    token.transfer(&distributor, &owner, &1000);

    // Create bounty with two winners
    let bounty_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Submit applications
    client.apply_to_bounty(&applicant1, &bounty_id, &Symbol::new(&env, "link1"));
    client.apply_to_bounty(&applicant2, &bounty_id, &Symbol::new(&env, "link2"));

    // Test winner selection
    let winners = vec![&env, applicant1.clone(), applicant2.clone()];
    client.select_winners(&owner, &bounty_id, &winners);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, Status::WinnersSelected);

    // Verify token distribution
    assert_eq!(token.balance(&applicant1), 594); // 60% of 990 (after 1% fee)
    assert_eq!(token.balance(&applicant2), 396); // 40% of 990 (after 1% fee)
    assert_eq!(token.balance(&owner), 10); // 1% fee
}

#[test]
fn test_auto_distribution() {
    let env = Env::default();
    let (client, token, distributor) = setup_test(&env);
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);
    token.transfer(&distributor, &owner, &1000);

    // Create bounty
    let bounty_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Submit applications
    client.apply_to_bounty(&applicant1, &bounty_id, &Symbol::new(&env, "link1"));
    client.apply_to_bounty(&applicant2, &bounty_id, &Symbol::new(&env, "link2"));

    // Move past judging deadline
    env.ledger().set_timestamp(env.ledger().timestamp() + 2001);

    // Trigger auto-distribution
    client.check_judging(&bounty_id);

    // Verify equal distribution
    assert_eq!(token.balance(&applicant1), 495); // 50% of 900 (after 1% fee)
    assert_eq!(token.balance(&applicant2), 495); // 50% of 900 (after 1% fee)
    assert_eq!(token.balance(&owner), 10); // 1% fee
}
