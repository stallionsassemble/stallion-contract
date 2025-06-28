#![cfg(test)]

extern crate std;

use crate::{
    Error, StallionContract, StallionContractClient, Status,
    utils::{adjust_for_decimals, get_token_decimals},
};
use soroban_sdk::{
    Address, Env, FromVal, IntoVal, String, Symbol, Vec,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _, Ledger},
    token::{StellarAssetClient as TokenAdminClient, TokenClient},
    vec,
};

fn create_token_contract(e: &Env) -> (TokenClient, Address) {
    e.mock_all_auths();

    let issuer = Address::generate(&e);
    let distributor = Address::generate(&e);

    // Create token contract with 7 decimals (standard for Stellar tokens)
    let sac = e.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();

    // client for SEP-41 functions
    let token = TokenClient::new(&e, &token_address);
    // client for Stellar Asset Contract functions
    let token_sac = TokenAdminClient::new(&e, &token_address);

    // note that we're explicitly working with a token that has 7 decimals
    // E.g. to mint 1 TOKEN, we need to use 1*10^7 in the mint function.
    let _decimals: u32 = 7; // Used implicitly through multiplier
    let genesis_amount: i128 = adjust_for_decimals(1_000_000_000, _decimals); // 1B tokens

    // Mint initial supply
    token_sac.mint(&distributor, &genesis_amount);

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
    let contract_id = env.register(StallionContract {}, (admin.clone(), fee_account.clone()));
    let client = StallionContractClient::new(&env, &contract_id);
    (client, token, distributor, fee_account, admin, contract_id)
}

fn verify_bounty_created_event(env: &Env, contract_id: &Address, bounty_id: &u32) {
    let event = env
        .events()
        .all()
        .try_last()
        .expect("No events found")
        .expect("Failed to get last event");

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &event.1.get_unchecked(0)),
        Symbol::new(env, "bounty_created")
    );
    assert_eq!(u32::from_val(env, &event.2), *bounty_id);
}

fn verify_bounty_updated_event(
    env: &Env,
    contract_id: &Address,
    bounty_id: &u32,
    updated_fields: &Vec<Symbol>,
) {
    let event = env
        .events()
        .all()
        .try_last()
        .expect("No events found")
        .expect("Failed to get last event");

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &event.1.get_unchecked(0)),
        Symbol::new(env, "bounty_updated")
    );
    assert_eq!(
        Vec::from_val(env, &event.2),
        vec![env, (bounty_id.clone(), updated_fields.clone())]
    );
}

fn verify_bounty_deleted_event(env: &Env, contract_id: &Address, bounty_id: &u32) {
    let event = env
        .events()
        .all()
        .try_last()
        .expect("No events found")
        .expect("Failed to get last event");

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(
        Symbol::from_val(env, &event.1.get_unchecked(0)),
        Symbol::new(env, "bounty_deleted")
    );
    assert_eq!(u32::from_val(env, &event.2), *bounty_id);
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

    // Define token amount with decimals - 1000 tokens with 7 decimals
    let user_friendly_amount = 1000; // Original token amount for contract input
    let token_amount = adjust_for_decimals(
        user_friendly_amount,
        get_token_decimals(&env, &token.address),
    );

    let owner = Address::generate(&env);
    // Transfer the amount adjusted for decimals
    token.transfer(&distributor, &owner, &token_amount);

    // Test valid bounty creation
    // Note: We pass user_friendly_amount, but internally it will be adjusted for token decimals
    let bounty_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );
    verify_bounty_created_event(&env, &_contract_id, &bounty_id);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, Status::Active);
    // Verify that the stored reward is the user-friendly amount
    assert_eq!(bounty.reward, user_friendly_amount);

    // Test invalid distribution
    let result = client.try_create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 60), (2, 30)], // Only adds to 90%
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );
    assert!(result.is_err());

    // Test invalid deadlines
    let result = client.try_create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
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

    let user_friendly_amount = 1000; // Original token amount for contract input
    let token_amount = adjust_for_decimals(
        user_friendly_amount,
        get_token_decimals(&env, &token.address),
    );

    let owner = Address::generate(&env);
    let applicant = Address::generate(&env);
    token.transfer(&distributor, &owner, &token_amount);

    // Create bounty
    let bounty_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Test valid submission
    client.apply_to_bounty(&applicant, &bounty_id, &String::from_str(&env, "link1"));

    // Test submission update
    client.apply_to_bounty(&applicant, &bounty_id, &String::from_str(&env, "link2"));

    let submissions = client.get_bounty_submissions(&bounty_id);
    assert_eq!(
        submissions.get(applicant.clone()).unwrap(),
        String::from_str(&env, "link2")
    );

    // Test submission after deadline
    env.ledger().set_timestamp(env.ledger().timestamp() + 1001);
    let result =
        client.try_apply_to_bounty(&applicant, &bounty_id, &String::from_str(&env, "link3"));
    assert!(result.is_err());
}

#[test]
fn test_winner_selection() {
    let env = Env::default();
    let (client, token, distributor, fee_account, _admin, _contract_id) = setup_test(&env);
    env.mock_all_auths();

    let user_friendly_amount = 1000; // Original token amount for contract input
    let token_amount = adjust_for_decimals(
        user_friendly_amount,
        get_token_decimals(&env, &token.address),
    );

    let owner = Address::generate(&env);
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);
    token.transfer(&distributor, &owner, &token_amount);

    // Create bounty with two winners
    let bounty_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Submit applications
    client.apply_to_bounty(&applicant1, &bounty_id, &String::from_str(&env, "link1"));
    client.apply_to_bounty(&applicant2, &bounty_id, &String::from_str(&env, "link2"));

    // Test winner selection
    let winners = vec![&env, applicant1.clone(), applicant2.clone()];
    client.select_winners(&owner, &bounty_id, &winners);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, Status::Completed);

    // Calculate expected balances with 7 decimals
    // 1% fee = 10 tokens = 10 * 10^7 = 100_000_000 units
    // Net reward = 990 tokens = 990 * 10^7 = 9_900_000_000 units
    // Applicant1 gets 60% of 990 = 594 tokens = 594 * 10^7 = 5_940_000_000 units
    // Applicant2 gets 40% of 990 = 396 tokens = 396 * 10^7 = 3_960_000_000 units

    // Verify token distribution includes fee going to fee_account
    assert_eq!(token.balance(&applicant1), 594 * 10_000_000); // 60% of 990 (after 1% fee)
    assert_eq!(token.balance(&applicant2), 396 * 10_000_000); // 40% of 990 (after 1% fee)
    assert_eq!(token.balance(&fee_account), 10 * 10_000_000); // 1% fee goes to fee account
}

#[test]
fn test_auto_distribution() {
    let env = Env::default();
    let (client, token, distributor, fee_account, _admin, _contract_id) = setup_test(&env);
    env.mock_all_auths();

    let user_friendly_amount = 1000; // Original token amount for contract input
    let token_amount = adjust_for_decimals(
        user_friendly_amount,
        get_token_decimals(&env, &token.address),
    );

    let owner = Address::generate(&env);
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);
    token.transfer(&distributor, &owner, &token_amount);

    // Create bounty
    let bounty_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Test bounty"),
    );

    // Submit applications
    client.apply_to_bounty(&applicant1, &bounty_id, &String::from_str(&env, "link1"));
    client.apply_to_bounty(&applicant2, &bounty_id, &String::from_str(&env, "link2"));

    // Move past judging deadline
    env.ledger().set_timestamp(env.ledger().timestamp() + 2001);

    // Trigger auto-distribution
    client.check_judging(&bounty_id);

    // Calculate expected balances with 7 decimals
    // 1% fee = 10 tokens = 10 * 10^7 = 100_000_000 units
    // Net reward = 990 tokens = 990 * 10^7 = 9_900_000_000 units
    // Split equally between two applicants = 495 tokens each = 495 * 10^7 = 4_950_000_000 units

    // Verify token distribution
    assert_eq!(token.balance(&applicant1), 495 * 10_000_000);
    assert_eq!(token.balance(&applicant2), 495 * 10_000_000);
    assert_eq!(token.balance(&fee_account), 10 * 10_000_000);
}

#[test]
fn test_get_active_bounties() {
    let env = Env::default();
    let (client, token, distributor, _fee_account, _admin, _contract_id) = setup_test(&env);
    env.mock_all_auths();

    let user_friendly_amount = 1000; // Amount per bounty in user-friendly format
    let token_amount = adjust_for_decimals(5000, get_token_decimals(&env, &token.address));

    let owner = Address::generate(&env);
    token.transfer(&distributor, &owner, &token_amount);

    // Create first active bounty
    let bounty1_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "First bounty"),
    );

    // Create second active bounty
    let bounty2_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Second bounty"),
    );

    // Create and complete third bounty
    let bounty3_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount,
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

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    // Transfer tokens to owners (with decimals)
    token.transfer(
        &distributor,
        &owner1,
        &adjust_for_decimals(2000, get_token_decimals(&env, &token.address)),
    );
    token.transfer(
        &distributor,
        &owner2,
        &adjust_for_decimals(1000, get_token_decimals(&env, &token.address)),
    );

    // Create second token contract and get its admin client to mint tokens
    let (token2, token2_distributor) = create_token_contract(&env);

    // Mint additional tokens to owners for both tokens (with decimals)
    token.transfer(
        &distributor,
        &owner1,
        &adjust_for_decimals(2000, get_token_decimals(&env, &token.address)),
    );
    token.transfer(
        &distributor,
        &owner2,
        &adjust_for_decimals(1000, get_token_decimals(&env, &token.address)),
    );

    // Mint tokens for token2 (with decimals)
    let token2_admin = TokenAdminClient::new(&env, &token2.address);
    token2_admin.mint(
        &token2_distributor,
        &adjust_for_decimals(10000, get_token_decimals(&env, &token2.address)),
    );
    token2.transfer(
        &token2_distributor,
        &owner1,
        &adjust_for_decimals(1000, get_token_decimals(&env, &token2.address)),
    );

    // Define user-friendly amounts for bounties
    let bounty1_amount = 1000;
    let bounty2_amount = 500;
    let bounty3_amount = 750;

    // Bounty 1: Owner1, Token1
    let bounty1_id = client.create_bounty(
        &owner1,
        &token.address,
        &bounty1_amount, // User-friendly amount, contract handles decimal adjustment
        &vec![&env, (1, 60), (2, 40)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Bounty 1"),
    );

    // Bounty 2: Owner1, Token2
    let bounty2_id = client.create_bounty(
        &owner1,
        &token2.address,
        &bounty2_amount, // User-friendly amount, contract handles decimal adjustment
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Bounty 2"),
    );

    // Bounty 3: Owner2, Token1
    let bounty3_id = client.create_bounty(
        &owner2,
        &token.address,
        &bounty3_amount, // User-friendly amount, contract handles decimal adjustment
        &vec![&env, (1, 50), (2, 30), (3, 20)],
        &(env.ledger().timestamp() + 1000),
        &(env.ledger().timestamp() + 2000),
        &String::from_str(&env, "Bounty 3"),
    );

    // Test get_bounties
    let all_bounties = client.get_bounties();
    assert_eq!(all_bounties.len(), 3);
    assert!(all_bounties.contains(&bounty1_id));
    assert!(all_bounties.contains(&bounty2_id));
    assert!(all_bounties.contains(&bounty3_id));

    // Test get_owner_bounties and get_owner_bounties_count
    let owner1_bounties = client.get_owner_bounties(&owner1);
    assert_eq!(owner1_bounties.len(), 2);
    assert!(owner1_bounties.contains(&bounty1_id));
    assert!(owner1_bounties.contains(&bounty2_id));
    assert_eq!(client.get_owner_bounties_count(&owner1), 2);

    let owner2_bounties = client.get_owner_bounties(&owner2);
    assert_eq!(owner2_bounties.len(), 1);
    assert!(owner2_bounties.contains(&bounty3_id));
    assert_eq!(client.get_owner_bounties_count(&owner2), 1);

    // Test get_user_bounties and get_user_bounties_count after making submissions
    // These will be our submitters
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);

    // Submit to bounties
    client.apply_to_bounty(&applicant1, &bounty1_id, &String::from_str(&env, "link1"));
    client.apply_to_bounty(&applicant1, &bounty2_id, &String::from_str(&env, "link2"));
    client.apply_to_bounty(&applicant2, &bounty1_id, &String::from_str(&env, "link3"));

    // Test get_user_bounties for applicant1 (should have submissions to bounty1 and bounty2)
    let applicant1_bounties = client.get_user_bounties(&applicant1);
    assert_eq!(applicant1_bounties.len(), 2);
    assert!(applicant1_bounties.contains(&bounty1_id));
    assert!(applicant1_bounties.contains(&bounty2_id));
    assert_eq!(client.get_user_bounties_count(&applicant1), 2);

    // Test get_user_bounties for applicant2 (should have submission to bounty1 only)
    let applicant2_bounties = client.get_user_bounties(&applicant2);
    assert_eq!(applicant2_bounties.len(), 1);
    assert!(applicant2_bounties.contains(&bounty1_id));
    assert_eq!(client.get_user_bounties_count(&applicant2), 1);

    // Test get_user_bounties for owner1 (should be empty since owner1 hasn't made any submissions)
    assert_eq!(client.get_user_bounties(&owner1).len(), 0);
    assert_eq!(client.get_user_bounties_count(&owner1), 0);

    // Test get_bounties_by_token and get_bounties_by_token_count
    let token1_bounties = client.get_bounties_by_token(&token.address);
    assert_eq!(token1_bounties.len(), 2);
    assert!(token1_bounties.contains(&bounty1_id));
    assert!(token1_bounties.contains(&bounty3_id));
    assert_eq!(client.get_bounties_by_token_count(&token.address), 2);

    let token2_bounties = client.get_bounties_by_token(&token2.address);
    assert_eq!(token2_bounties.len(), 1);
    assert!(token2_bounties.contains(&bounty2_id));
    assert_eq!(client.get_bounties_by_token_count(&token2.address), 1);

    // Test submissions and applicants on bounty1
    let applicants = client.get_bounty_applicants(&bounty1_id);
    assert_eq!(applicants.len(), 2);
    assert!(applicants.contains(&applicant1));
    assert!(applicants.contains(&applicant2));

    let submissions = client.get_bounty_submissions(&bounty1_id);
    assert_eq!(submissions.len(), 2);
    assert_eq!(
        submissions.get(applicant1.clone()).unwrap(),
        String::from_str(&env, "link1")
    );
    assert_eq!(
        submissions.get(applicant2.clone()).unwrap(),
        String::from_str(&env, "link3")
    );

    // Test get submission
    let submission = client.get_submission(&bounty1_id, &applicant1);
    assert_eq!(submission, String::from_str(&env, "link1"));

    // Test winners getter
    let winners = vec![&env, applicant1.clone(), applicant2.clone()];
    client.select_winners(&owner1, &bounty1_id, &winners);

    let stored_winners = client.get_bounty_winners(&bounty1_id);
    assert_eq!(stored_winners.len(), 2);
    assert!(stored_winners.contains(&applicant1));
    assert!(stored_winners.contains(&applicant2));

    // Verify status change
    assert_eq!(client.get_bounty_status(&bounty1_id), Status::Completed);

    // Test full bounty getter
    let bounty = client.get_bounty(&bounty1_id);
    assert_eq!(bounty.owner, owner1);
    assert_eq!(bounty.reward, 1000);
    assert_eq!(bounty.status, Status::Completed);
    assert_eq!(bounty.winners, stored_winners);
    assert_eq!(bounty.applicants, applicants);
    assert_eq!(bounty.submissions, submissions);
}

#[test]
fn test_update_submission() {
    let env = Env::default();
    let (client, token, distributor, _fee_account, _admin, _contract_id) = setup_test(&env);
    env.mock_all_auths();

    let bounty_amount = 1000;
    let transfer_amount = adjust_for_decimals(2000, get_token_decimals(&env, &token.address));

    // Setup test data
    let owner = Address::generate(&env);
    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);

    // Transfer tokens to owner (with decimals)
    token.transfer(&distributor, &owner, &transfer_amount);

    // Create a bounty
    let bounty_id = client.create_bounty(
        &owner,
        &token.address,
        &bounty_amount,
        &vec![&env, (1, 100)],
        &(env.ledger().timestamp() + 1000), // submission deadline
        &(env.ledger().timestamp() + 2000), // judging deadline
        &String::from_str(&env, "Test Bounty"),
    );

    // Make some submissions
    client.apply_to_bounty(
        &applicant1,
        &bounty_id,
        &String::from_str(&env, "initial_link1"),
    );
    client.apply_to_bounty(&applicant2, &bounty_id, &String::from_str(&env, "link2"));

    // Verify initial state
    let initial_submissions = client.get_bounty_submissions(&bounty_id);
    assert_eq!(
        initial_submissions.get(applicant1.clone()).unwrap(),
        String::from_str(&env, "initial_link1")
    );
    assert_eq!(
        initial_submissions.get(applicant2.clone()).unwrap(),
        String::from_str(&env, "link2")
    );

    // Test successful update
    client.update_submission(
        &applicant1,
        &bounty_id,
        &String::from_str(&env, "updated_link1"),
    );

    // Verify the update
    let updated_submissions = client.get_bounty_submissions(&bounty_id);
    assert_eq!(
        updated_submissions.get(applicant1.clone()).unwrap(),
        String::from_str(&env, "updated_link1")
    );
    assert_eq!(
        updated_submissions.get(applicant2.clone()).unwrap(),
        String::from_str(&env, "link2")
    );

    // Test updating a non-existent submission (should fail with InternalError)
    let non_applicant = Address::generate(&env);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.update_submission(
            &non_applicant,
            &bounty_id,
            &String::from_str(&env, "should_fail"),
        );
    }));
    assert!(
        result.is_err(),
        "Should not be able to update a non-existent submission"
    );

    // Test updating after deadline (should fail with BountyDeadlinePassed)
    let bounty = client.get_bounty(&bounty_id);
    env.ledger().with_mut(|li| {
        li.timestamp = bounty.submission_deadline + 1;
    });

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.update_submission(
            &applicant1,
            &bounty_id,
            &String::from_str(&env, "should_fail_due_to_deadline"),
        );
    }));
    assert!(
        result.is_err(),
        "Should not be able to update submission after deadline"
    );
}

#[test]
fn test_update_and_delete_bounty() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, token, distributor, _fee_account, _admin, contract_id) = setup_test(&env);
    let owner = Address::generate(&env);
    let not_owner = Address::generate(&env);

    // Fund the owner with some tokens - ensure enough for our tests
    let token_client = TokenClient::new(&env, &token.address);
    token_client.transfer(
        &distributor,
        &owner,
        &adjust_for_decimals(2000, get_token_decimals(&env, &token.address)),
    );

    // Define user-friendly amounts for testing
    let user_friendly_amount1 = 1000;
    let user_friendly_amount2 = 500;

    // Create a bounty
    let distribution = vec![&env, (1, 60), (2, 40)];
    let submission_deadline = env.ledger().timestamp() + 1000;
    let judging_deadline = submission_deadline + 1000;
    let title = String::from_str(&env, "Test Bounty");

    let bounty_id = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount1, // 1000 tokens in user-friendly format
        &distribution,
        &submission_deadline,
        &judging_deadline,
        &title,
    );

    // Test 1: Try to update with non-owner (should fail)
    let result = client.try_update_bounty(
        &not_owner,
        &bounty_id,
        &Some(String::from_str(&env, "New Title")),
        &vec![&env],
        &None,
    );
    assert_eq!(result, Err(Ok(Error::OnlyOwner)));

    // Test 2: Update title
    client.update_bounty(
        &owner,
        &bounty_id,
        &Some(String::from_str(&env, "Updated Title")),
        &vec![&env],
        &None,
    );
    verify_bounty_updated_event(
        &env,
        &contract_id,
        &bounty_id,
        &vec![&env, Symbol::new(&env, "title")],
    );

    // Verify update
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.title, String::from_str(&env, "Updated Title"));

    // Test 3: Update distribution
    let new_distribution = vec![&env, (1, 70), (2, 30)];
    client.update_bounty(&owner, &bounty_id, &None, &new_distribution, &None);

    // Verify update
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.distribution.get(1).unwrap(), 70);
    assert_eq!(bounty.distribution.get(2).unwrap(), 30);

    // Test 4: Update submission deadline
    let new_submission_deadline = env.ledger().timestamp() + 500;
    client.update_bounty(
        &owner,
        &bounty_id,
        &None,
        &vec![&env],
        &Some(new_submission_deadline),
    );
    verify_bounty_updated_event(
        &env,
        &contract_id,
        &bounty_id,
        &vec![&env, Symbol::new(&env, "submission_deadline")],
    );

    // Verify update
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.submission_deadline, new_submission_deadline);

    // Test 5: Try to update with invalid deadline (in the past)
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let past_deadline = env.ledger().timestamp() - 1;
    let result =
        client.try_update_bounty(&owner, &bounty_id, &None, &vec![&env], &Some(past_deadline));
    assert_eq!(result, Err(Ok(Error::InvalidDeadlineUpdate)));

    // Test 6: Try to delete with non-owner (should fail)
    let result = client.try_delete_bounty(&not_owner, &bounty_id);
    assert_eq!(result, Err(Ok(Error::OnlyOwner)));

    // Test 7: Try to delete with submissions (should fail)
    let applicant = Address::generate(&env);
    let submission_link = String::from_str(&env, "https://example.com/submission");
    client.apply_to_bounty(&applicant, &bounty_id, &submission_link);

    let result = client.try_delete_bounty(&owner, &bounty_id);
    assert_eq!(result, Err(Ok(Error::BountyHasSubmissions)));

    // Create a new bounty without submissions to test deletion
    let bounty_id2 = client.create_bounty(
        &owner,
        &token.address,
        &user_friendly_amount2, // 500 tokens in user-friendly format
        &distribution,
        &submission_deadline,
        &judging_deadline,
        &title,
    );

    // Get initial owner balance
    let initial_balance = token_client.balance(&owner);

    // Test 8: Delete bounty without submissions (should succeed)
    client.delete_bounty(&owner, &bounty_id2);
    verify_bounty_deleted_event(&env, &contract_id, &bounty_id2);

    // Verify bounty is deleted
    let result = client.try_get_bounty(&bounty_id2);
    assert!(result.is_err());

    // Verify funds were returned to owner - the full amount with decimals
    let final_balance = token_client.balance(&owner);
    assert_eq!(
        final_balance,
        initial_balance
            + adjust_for_decimals(
                user_friendly_amount2,
                get_token_decimals(&env, &token.address)
            )
    );
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
    assert_eq!(result, Err(Ok(Error::FeeAccountCannotBeZero)));

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

    // Test updating to the same fee account (should fail)
    let result = client.try_update_fee_account(&new_fee_account);
    assert!(result.is_err());
    assert_eq!(result, Err(Ok(Error::SameFeeAccount)));

    // Clear previous mock auths
    env.set_auths(&[]);

    // Test that old admin can't make changes
    let result = client.try_update_fee_account(&new_fee_account);
    assert!(result.is_err());

    // Test that non-admin can't make changes
    let random_user = Address::generate(&env);
    let result = client.try_update_fee_account(&random_user);
    assert!(result.is_err());
}
