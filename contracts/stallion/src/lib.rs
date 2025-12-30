// SPDX-License-Identifier: Boost Software License, Version 1.0.0
#![no_std]

use soroban_sdk::{Address, Env, Map, String, Symbol, Vec, contract, contractimpl, contractmeta};

mod events;
mod storage;
mod types;
mod utils;

use crate::types::*;
use crate::utils::{
    adjust_for_decimals, calculate_fee, get_token_client, get_token_decimals, is_zero_address,
    validate_distribution_sum,
};
use events::Events;
use storage::{admin_key, bounty_key, fee_account_key, next_id_key};

contractmeta!(key = "Version", val = "0.1.0");
contractmeta!(
    key = "Description",
    val = "Stallion decentralized bounty platform"
);
contractmeta!(
    key = "License",
    val = "Boost Software License, Version 1.0.0"
);

#[contract]
pub struct StallionContract;

#[contractimpl]
impl StallionContract {
    pub fn __constructor(env: Env, admin: Address, fee_account: Address) {
        if is_zero_address(&env, &admin) {
            panic!("admin cannot be zero address");
        }
        if is_zero_address(&env, &fee_account) {
            panic!("fee account cannot be zero address");
        }

        let storage = env.storage().persistent();
        storage.set(&admin_key(), &admin);
        storage.set(&fee_account_key(), &fee_account);
        Events::emit_admin_updated(&env, admin);
        Events::emit_fee_account_updated(&env, fee_account);
    }

    fn get_admin(env: &Env) -> Address {
        env.storage().persistent().get(&admin_key()).unwrap()
    }

    fn get_fee_account(env: &Env) -> Address {
        env.storage().persistent().get(&fee_account_key()).unwrap()
    }

    pub fn get_bounties(env: Env) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut bounties = Vec::new(&env);
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            bounties.push_back(id);
        }
        bounties
    }

    pub fn get_user_bounties(env: Env, user: Address) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut bounties = Vec::new(&env);
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.submissions.contains_key(user.clone()) {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_user_bounties_count(env: Env, user: Address) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut count = 0;
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.submissions.contains_key(user.clone()) {
                count += 1;
            }
        }
        count
    }

    pub fn get_owner_bounties(env: Env, owner: Address) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut bounties = Vec::new(&env);
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.owner == owner {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_owner_bounties_count(env: Env, owner: Address) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut count = 0;
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.owner == owner {
                count += 1;
            }
        }
        count
    }

    pub fn get_bounties_by_token(env: Env, token: Address) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut bounties = Vec::new(&env);
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.token == token {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_bounties_by_token_count(env: Env, token: Address) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut count = 0;
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.token == token {
                count += 1;
            }
        }
        count
    }

    pub fn get_active_bounties(env: Env) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut active = Vec::new(&env);

        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.status == Status::Active {
                active.push_back(id);
            }
        }
        active
    }

    pub fn get_bounties_count(env: Env) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);

        let mut count = 0;
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            count += 1;
        }

        count
    }

    pub fn get_bounties_by_status(env: Env, status: Status) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut bounties = Vec::new(&env);
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.status == status {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_bounties_by_status_count(env: Env, status: Status) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let mut count = 0;
        for id in 1..next_id {
            let bounty: Option<Bounty> = storage.get(&bounty_key(id));
            if bounty.is_none() {
                continue;
            }

            let bounty = bounty.unwrap();
            if bounty.status == status {
                count += 1;
            }
        }
        count
    }

    pub fn get_bounty(env: Env, bounty_id: u32) -> Result<Bounty, Error> {
        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();
        Ok(bounty)
    }

    pub fn get_submission(env: Env, bounty_id: u32, user: Address) -> Result<String, Error> {
        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();
        let submission = bounty.submissions.get(user);
        if submission.is_none() {
            return Err(Error::SubmissionNotFound);
        }

        Ok(submission.unwrap())
    }

    pub fn get_bounty_submissions(env: Env, bounty_id: u32) -> Result<Map<Address, String>, Error> {
        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();
        Ok(bounty.submissions)
    }

    pub fn get_bounty_applicants(env: Env, bounty_id: u32) -> Result<Vec<Address>, Error> {
        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();
        Ok(bounty.applicants)
    }

    pub fn get_bounty_winners(env: Env, bounty_id: u32) -> Result<Vec<Address>, Error> {
        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();
        Ok(bounty.winners)
    }

    pub fn get_bounty_status(env: Env, bounty_id: u32) -> Result<Status, Error> {
        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();
        Ok(bounty.status)
    }

    pub fn update_admin(env: Env, new_admin: Address) -> Result<Address, Error> {
        let admin = Self::get_admin(&env);
        admin.require_auth();

        if is_zero_address(&env, &new_admin) {
            return Err(Error::AdminCannotBeZero);
        }

        env.storage().persistent().set(&admin_key(), &new_admin);
        Events::emit_admin_updated(&env, new_admin.clone());
        Ok(new_admin)
    }

    pub fn update_fee_account(env: Env, new_fee_account: Address) -> Result<Address, Error> {
        let admin = Self::get_admin(&env);
        admin.require_auth();

        if is_zero_address(&env, &new_fee_account) {
            return Err(Error::FeeAccountCannotBeZero);
        }

        // Check if the new fee account is the same as the current fee account
        let current_fee_account = Self::get_fee_account(&env);
        if current_fee_account == new_fee_account {
            return Err(Error::SameFeeAccount);
        }

        env.storage()
            .persistent()
            .set(&fee_account_key(), &new_fee_account);
        Events::emit_fee_account_updated(&env, new_fee_account.clone());
        Ok(new_fee_account)
    }

    pub fn create_bounty(
        env: Env,
        owner: Address,
        token: Address,
        reward: i128,
        distribution: Vec<(u32, u32)>,
        submission_deadline: u64,
        judging_deadline: u64,
        title: String,
    ) -> Result<u32, Error> {
        let storage = env.storage().persistent();

        if reward <= 0 {
            return Err(Error::InvalidReward);
        }

        if !validate_distribution_sum(&distribution) {
            return Err(Error::DistributionMustSumTo100);
        }

        // Validate deadlines
        if judging_deadline <= submission_deadline {
            return Err(Error::JudgingDeadlineMustBeAfterSubmissionDeadline);
        }

        // Transfer reward to contract
        owner.require_auth();
        let token_client = get_token_client(&env, token.clone());

        // Get token decimals and adjust reward amount
        let decimals = get_token_decimals(&env, &token);
        let adjusted_reward = adjust_for_decimals(reward, decimals);

        // Transfer the adjusted amount to the account
        token_client.transfer(&owner, &env.current_contract_address(), &adjusted_reward);

        // Assign new bounty ID
        let id: u32 = storage.get(&next_id_key()).unwrap_or(1);
        let next = id + 1;
        storage.set(&next_id_key(), &next);

        // Initialize bounty - store the original reward amount (not adjusted for decimals)
        // This way when displaying, we show the user-friendly amount
        let mut distribution_map = Map::new(&env);
        for (rank, percent) in distribution.iter() {
            distribution_map.set(rank, percent);
        }
        let bounty = Bounty {
            owner: owner.clone(),
            token: token.clone(),
            reward, // Store the original reward amount for display purposes
            distribution: distribution_map,
            submission_deadline,
            judging_deadline,
            title: title.clone(),
            status: Status::Active,
            applicants: Vec::new(&env),
            submissions: Map::new(&env),
            winners: Vec::new(&env),
        };
        storage.set(&bounty_key(id), &bounty);
        Events::emit_bounty_created(&env, id);

        Ok(id)
    }

    // Update an existing bounty
    pub fn update_bounty(
        env: Env,
        owner: Address,
        bounty_id: u32,
        new_title: Option<String>,
        new_distribution: Vec<(u32, u32)>,
        new_submission_deadline: Option<u64>,
    ) -> Result<(), Error> {
        owner.require_auth();

        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));

        // Only the bounty owner can update the bounty
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let mut bounty = bounty.unwrap();
        if bounty.owner != owner {
            return Err(Error::OnlyOwner);
        }

        // Check if the bounty is still active
        if bounty.status != Status::Active {
            return Err(Error::InactiveBounty);
        }

        let now = env.ledger().timestamp();

        // Update distribution if provided
        if !new_distribution.is_empty() {
            if !validate_distribution_sum(&new_distribution) {
                return Err(Error::DistributionMustSumTo100);
            }
            let mut distribution_map = Map::new(&env);
            for (rank, percent) in new_distribution.iter() {
                distribution_map.set(rank, percent);
            }
            bounty.distribution = distribution_map;
        }

        // Update submission deadline if provided
        if let Some(submission_deadline) = new_submission_deadline {
            // Can't move submission deadline to the past
            if submission_deadline < now {
                return Err(Error::InvalidDeadlineUpdate);
            }
            // Can't move submission deadline past the judging deadline
            if submission_deadline >= bounty.judging_deadline {
                return Err(Error::JudgingDeadlineMustBeAfterSubmissionDeadline);
            }
            bounty.submission_deadline = submission_deadline;
        }

        // Update title if provided
        if let Some(title) = &new_title {
            bounty.title = title.clone();
        }

        let mut updated_fields: Vec<Symbol> = Vec::new(&env);
        if new_title.is_some() {
            updated_fields.push_back(Symbol::new(&env, "title"));
        }
        if !new_distribution.is_empty() {
            updated_fields.push_back(Symbol::new(&env, "distribution"));
        }
        if let Some(_submission_deadline) = new_submission_deadline {
            updated_fields.push_back(Symbol::new(&env, "submission_deadline"));
        }

        // Save the updated bounty
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_bounty_updated(&env, bounty_id, updated_fields);

        Ok(())
    }

    // Delete a bounty if it has no submissions
    pub fn delete_bounty(env: Env, owner: Address, bounty_id: u32) -> Result<(), Error> {
        owner.require_auth();

        let storage = env.storage().persistent();

        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let bounty = bounty.unwrap();

        if bounty.owner != owner {
            return Err(Error::OnlyOwner);
        }

        // Check if there are any submissions
        if bounty.submissions.len() > 0 {
            return Err(Error::BountyHasSubmissions);
        }

        // Get token decimals for adjustment
        let token_client = get_token_client(&env, bounty.token.clone());
        let decimals = get_token_decimals(&env, &bounty.token);

        // Adjust reward amount according to token decimals
        let adjusted_reward = adjust_for_decimals(bounty.reward, decimals);

        // Return funds to owner if the bounty has not been closed
        if bounty.status != Status::Closed {
            token_client.transfer(&env.current_contract_address(), &owner, &adjusted_reward);
        }

        // Remove bounty
        storage.remove(&bounty_key(bounty_id));
        Events::emit_bounty_deleted(&env, bounty_id);

        Ok(())
    }

    // Close a bounty if it has no submissions
    pub fn close_bounty(env: Env, owner: Address, bounty_id: u32) -> Result<(), Error> {
        owner.require_auth();

        let storage = env.storage().persistent();

        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let mut bounty = bounty.unwrap();

        if bounty.owner != owner {
            return Err(Error::OnlyOwner);
        }

        // Check if there are any submissions
        if bounty.submissions.len() > 0 {
            return Err(Error::BountyHasSubmissions);
        }

        // Get token decimals for adjustment
        let token_client = get_token_client(&env, bounty.token.clone());
        let decimals = get_token_decimals(&env, &bounty.token);

        // Adjust reward amount according to token decimals
        let adjusted_reward = adjust_for_decimals(bounty.reward, decimals);

        // Return funds to owner
        token_client.transfer(&env.current_contract_address(), &owner, &adjusted_reward);

        // Update bounty status to Closed
        bounty.status = Status::Closed;
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_bounty_closed(&env, bounty_id);

        Ok(())
    }

    // Apply to an active bounty
    pub fn apply_to_bounty(
        env: Env,
        applicant: Address,
        bounty_id: u32,
        submission_link: String,
    ) -> Result<(), Error> {
        applicant.require_auth();

        let storage = env.storage().persistent();
        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let mut bounty = bounty.unwrap();
        let now = env.ledger().timestamp();
        if bounty.status != Status::Active {
            return Err(Error::InactiveBounty);
        }
        if now > bounty.submission_deadline {
            return Err(Error::BountyDeadlinePassed);
        }
        // Register applicant if new
        if !bounty.submissions.contains_key(applicant.clone()) {
            bounty.applicants.push_back(applicant.clone());
        }
        // Set/update submission
        bounty
            .submissions
            .set(applicant.clone(), submission_link.clone());
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_submission_added(&env, bounty_id, applicant);

        Ok(())
    }

    // Update an existing submission before the deadline
    pub fn update_submission(
        env: Env,
        applicant: Address,
        bounty_id: u32,
        new_submission_link: String,
    ) -> Result<(), Error> {
        applicant.require_auth();

        let storage = env.storage().persistent();

        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }
        let mut bounty = bounty.unwrap();

        let now = env.ledger().timestamp();

        // Check if bounty is active
        if bounty.status != Status::Active {
            return Err(Error::InactiveBounty);
        }

        // Check if submission deadline has passed
        if now > bounty.submission_deadline {
            return Err(Error::BountyDeadlinePassed);
        }

        // Check if the applicant has an existing submission
        if !bounty.submissions.contains_key(applicant.clone()) {
            return Err(Error::SubmissionNotFound);
        }

        // Update the submission
        bounty
            .submissions
            .set(applicant.clone(), new_submission_link);
        storage.set(&bounty_key(bounty_id), &bounty);

        // Emit an event for the update
        Events::emit_submission_updated(&env, bounty_id, applicant);

        Ok(())
    }

    // Select winners before judging deadline
    pub fn select_winners(
        env: Env,
        owner: Address,
        bounty_id: u32,
        winners: Vec<Address>,
    ) -> Result<(), Error> {
        owner.require_auth();

        let storage = env.storage().persistent();

        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let mut bounty = bounty.unwrap();

        if bounty.owner != owner {
            return Err(Error::OnlyOwner);
        }
        let now = env.ledger().timestamp();
        if now < bounty.submission_deadline {
            return Err(Error::CannotSelectWinnersBeforeSubmissionDeadline);
        }
        if now > bounty.judging_deadline {
            return Err(Error::JudgingDeadlinePassed);
        }
        let num_spec = bounty.distribution.len();
        if winners.len() < num_spec {
            return Err(Error::NotEnoughWinners);
        }

        // Get token decimals to adjust amounts for transfers
        let token_client = get_token_client(&env, bounty.token.clone());
        let decimals = get_token_decimals(&env, &bounty.token);

        // Use the original reward amount (user-friendly) for calculations
        let fee = calculate_fee(bounty.reward);
        let net = bounty.reward - fee;
        let fee_account = Self::get_fee_account(&env);

        // Calculate how many winners we can actually reward
        let actual_winners = winners.len().min(bounty.applicants.len());
        let mut distributed = 0i128;

        // Distribute to available winners
        for i in 0..actual_winners {
            let rank = (i + 1) as u32;
            if let Some(pct) = bounty.distribution.get(rank) {
                let amount = net * (pct as i128) / 100;
                let winner = winners.get(i as u32).unwrap();

                // Adjust amount for token decimals before transfer
                let adjusted_amount = adjust_for_decimals(amount, decimals);
                token_client.transfer(&env.current_contract_address(), &winner, &adjusted_amount);

                distributed += amount; // Track using original amount for calculation purposes
            }
        }

        // Return remaining funds to owner (if any)
        let remaining = net - distributed;
        if remaining > 0 {
            // Adjust remaining amount for token decimals
            let adjusted_remaining = adjust_for_decimals(remaining, decimals);
            token_client.transfer(
                &env.current_contract_address(),
                &bounty.owner,
                &adjusted_remaining,
            );
        }

        // Transfer platform fee to fee account
        let adjusted_fee = adjust_for_decimals(fee, decimals);
        token_client.transfer(&env.current_contract_address(), &fee_account, &adjusted_fee);

        bounty.status = Status::Completed;
        bounty.winners = winners.clone();
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_winners_selected(&env, bounty_id, winners);

        Ok(())
    }

    // Check and auto-distribute if judging deadline passed
    pub fn check_judging(env: Env, bounty_id: u32) -> Result<(), Error> {
        let storage = env.storage().persistent();

        let bounty: Option<Bounty> = storage.get(&bounty_key(bounty_id));
        if bounty.is_none() {
            return Err(Error::BountyNotFound);
        }

        let mut bounty = bounty.unwrap();

        let now = env.ledger().timestamp();
        if now <= bounty.judging_deadline || bounty.status != Status::Active {
            return Ok(());
        }

        // Get token decimals for adjustment
        let token_client = get_token_client(&env, bounty.token.clone());
        let decimals = get_token_decimals(&env, &bounty.token);

        // auto-distribute equally to all applicants
        let fee = calculate_fee(bounty.reward);
        let net = bounty.reward - fee;
        let count = bounty.applicants.len() as i128;

        if count == 0 {
            // Return full reward to owner if no applicants
            let adjusted_reward = adjust_for_decimals(bounty.reward, decimals);
            token_client.transfer(
                &env.current_contract_address(),
                &bounty.owner,
                &adjusted_reward,
            );
            return Ok(());
        }

        // Calculate share for each applicant
        let share = net / count;
        let fee_account = Self::get_fee_account(&env);

        // Adjust share amount for token decimals
        let adjusted_share = adjust_for_decimals(share, decimals);

        // Distribute to each applicant
        for applicant in bounty.applicants.iter() {
            token_client.transfer(&env.current_contract_address(), &applicant, &adjusted_share);
        }

        // Transfer fee to fee account
        let adjusted_fee = adjust_for_decimals(fee, decimals);
        token_client.transfer(&env.current_contract_address(), &fee_account, &adjusted_fee);

        bounty.status = Status::Completed;
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_auto_distributed(&env, bounty_id);

        Ok(())
    }
}

mod test;
