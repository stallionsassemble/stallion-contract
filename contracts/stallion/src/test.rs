#![cfg(test)]

extern crate std;

use crate::{StallionContract, StallionContractClient, Status};
use soroban_sdk::{
    Address, Env, FromVal, IntoVal, String, Symbol, log,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _, Ledger},
    token::{StellarAssetClient as TokenAdminClient, TokenClient},
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
    let token_sac = TokenAdminClient::new(&e, &token_address);

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

fn setup_test(
    env: &Env,
) -> (
    StallionContractClient,
    TokenClient,
    Address,
    Address,
    Address,
    Address,
) {
    let (token, distributor) = create_token_contract(&env);
    let admin = Address::generate(&env);
    let fee_account = Address::generate(&env);
    let contract_id = env.register(
        StallionContract {},
        (token.address.clone(), admin.clone(), fee_account.clone()),
    );
    let client = StallionContractClient::new(&env, &contract_id);
    (client, token, distributor, fee_account, admin, contract_id)
}

fn verify_admin_updated_event(env: &Env, contract_id: &Address, admin: &Address) {
    let event = env
        .events()
        .all()
        .try_last()
        .expect("No events found")
        .expect("Failed to get last event");

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &event.1.get_unchecked(0)),
        Symbol::new(env, "admin_updated")
    );
    assert_eq!(Address::from_val(env, &event.2), admin.clone());
}

fn verify_fee_account_updated_event(env: &Env, contract_id: &Address, fee_account: &Address) {
    let event = env
        .events()
        .all()
        .try_last()
        .expect("No events found")
        .expect("Failed to get last event");

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &event.1.get_unchecked(0)),
        Symbol::new(env, "fee_account_updated")
    );
    assert_eq!(Address::from_val(env, &event.2), fee_account.clone());
}

fn verify_constructor_events(
    env: &Env,
    contract_id: &Address,
    admin: &Address,
    fee_account: &Address,
) {
    let mut events = env.events().all().iter();

    // Verify admin event
    let (event_contract_id, topics, data) = events.next().unwrap();
    assert_eq!(event_contract_id, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &topics.get_unchecked(0)),
        Symbol::new(env, "admin_updated")
    );
    assert_eq!(Address::from_val(env, &data), admin.clone());

    // Verify fee account event
    let (event_contract_id, topics, data) = events.next().unwrap();
    assert_eq!(event_contract_id, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &topics.get_unchecked(0)),
        Symbol::new(env, "fee_account_updated")
    );
    assert_eq!(Address::from_val(env, &data), fee_account.clone());
}

#[test]
fn test_constructor() {
    let env = Env::default();
    let (_client, _token, _distributor, fee_account, admin, contract_id) = setup_test(&env);
    verify_constructor_events(&env, &contract_id, &admin, &fee_account);
}

#[test]
fn test_bounty_creation() {
    let env = Env::default();
    let (client, token, distributor, _fee_account, _admin, _contract_id) = setup_test(&env);
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
    let (client, token, distributor, _fee_account, _admin, _contract_id) = setup_test(&env);
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
    let (client, token, distributor, fee_account, _admin, _contract_id) = setup_test(&env);
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

    // Verify token distribution includes fee going to fee_account
    assert_eq!(token.balance(&applicant1), 594); // 60% of 990 (after 1% fee)
    assert_eq!(token.balance(&applicant2), 396); // 40% of 990 (after 1% fee)
    assert_eq!(token.balance(&fee_account), 10); // 1% fee goes to fee account
}

#[test]
fn test_auto_distribution() {
    let env = Env::default();
    let (client, token, distributor, fee_account, _admin, _contract_id) = setup_test(&env);
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

    // Verify fee goes to fee_account
    assert_eq!(token.balance(&applicant1), 495);
    assert_eq!(token.balance(&applicant2), 495);
    assert_eq!(token.balance(&fee_account), 10);
}

#[test]
fn test_get_active_bounties() {
    let env = Env::default();
    let (client, token, distributor, _fee_account, _admin, _contract_id) = setup_test(&env);
    env.mock_all_auths();

    let owner = Address::generate(&env);
    token.transfer(&distributor, &owner, &3000);

    // Create first active bounty
    let bounty1_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "First bounty"),
    );

    // Create second active bounty
    let bounty2_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Second bounty"),
    );

    // Create and complete third bounty
    let bounty3_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Third bounty"),
    );
    let winner = Address::generate(&env);
    client.select_winners(&owner, &bounty3_id, &vec![&env, winner]);

    // Get active bounties
    let active_bounties = client.get_active_bounties();

    // Verify only the first two bounties are active
    assert_eq!(active_bounties.len(), 2);
    assert!(active_bounties.contains(&bounty1_id));
    assert!(active_bounties.contains(&bounty2_id));
    assert!(!active_bounties.contains(&bounty3_id));
}

#[test]
fn test_getters() {
    let env = Env::default();
    let (client, token, distributor, _fee_account, _admin, _contract_id) = setup_test(&env);
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);
    token.transfer(&distributor, &owner, &1000);

    // Create bounty
    let bounty_id = client.create_bounty(
        &owner,
        &1000,
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Test initial status
    assert_eq!(client.get_bounty_status(&bounty_id), Status::Active);

    // Test submissions and applicants
    client.apply_to_bounty(&applicant1, &bounty_id, &Symbol::new(&env, "link1"));
    client.apply_to_bounty(&applicant2, &bounty_id, &Symbol::new(&env, "link2"));

    let applicants = client.get_bounty_applicants(&bounty_id);
    assert_eq!(applicants.len(), 2);
    assert!(applicants.contains(&applicant1));
    assert!(applicants.contains(&applicant2));

    let submissions = client.get_bounty_submissions(&bounty_id);
    assert_eq!(submissions.len(), 2);
    assert_eq!(
        submissions.get(applicant1.clone()).unwrap(),
        Symbol::new(&env, "link1")
    );
    assert_eq!(
        submissions.get(applicant2.clone()).unwrap(),
        Symbol::new(&env, "link2")
    );

    // Test winners getter
    let winners = vec![&env, applicant1.clone(), applicant2.clone()];
    client.select_winners(&owner, &bounty_id, &winners);

    let stored_winners = client.get_bounty_winners(&bounty_id);
    assert_eq!(stored_winners.len(), 2);
    assert!(stored_winners.contains(&applicant1));
    assert!(stored_winners.contains(&applicant2));

    // Verify status change
    assert_eq!(
        client.get_bounty_status(&bounty_id),
        Status::WinnersSelected
    );

    // Test full bounty getter
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.owner, owner);
    assert_eq!(bounty.reward, 1000);
    assert_eq!(bounty.status, Status::WinnersSelected);
    assert_eq!(bounty.winners, stored_winners);
    assert_eq!(bounty.applicants, applicants);
    assert_eq!(bounty.submissions, submissions);
}

#[test]
fn test_admin_functions() {
    let env = Env::default();
    let (client, _token, _distributor, _fee_account, admin, contract_id) = setup_test(&env);

    let new_admin = Address::generate(&env);
    let new_fee_account = Address::generate(&env);
    let zero_address = Address::from_string(&String::from_str(
        &env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    ));

    // Test update admin with zero address (should fail)
    let result = client.try_update_admin(&zero_address);
    assert!(result.is_err());

    // Test update admin with valid address
    client.update_admin(&new_admin);
    assert_eq!(
        env.auths(),
        [(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&env, "update_admin"),
                    (&new_admin,).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    verify_admin_updated_event(&env, &contract_id, &new_admin);

    // Test update fee account with zero address (should fail)
    let result = client.try_update_fee_account(&zero_address);
    assert!(result.is_err());

    // Test update fee account with valid address
    client.update_fee_account(&new_fee_account);
    assert_eq!(
        env.auths(),
        [(
            new_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&env, "update_fee_account"),
                    (&new_fee_account,).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    verify_fee_account_updated_event(&env, &contract_id, &new_fee_account);

    // FIX: Test that old admin can't make changes
    // let result = client.try_update_fee_account(&new_fee_account).unwrap();
    // assert!(result.is_err());

    // FIX: Test that non-admin can't make changes
    // let random_user = Address::generate(&env);
    // let result = client.try_update_fee_account(&random_user).unwrap();
    // assert!(result.is_err());
}
