// SPDX-License-Identifier: Boost Software License, Version 1.0.0
#![no_std]

use soroban_sdk::{Address, Env, Map, String, Symbol, Vec, contract, contractimpl, contractmeta};

mod events;
mod storage;
mod types;
mod utils;

use events::Events;
use storage::{admin_key, bounty_key, fee_account_key, next_id_key};
use types::{Bounty, Error, Status};
use utils::{calculate_fee, get_token_client, is_zero_address, validate_distribution_sum};

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
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut bounties = Vec::new(&env);
        for id in 0..next_id {
            bounties.push_back(id);
        }
        bounties
    }

    pub fn get_user_bounties(env: Env, user: Address) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut bounties = Vec::new(&env);
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.submissions.contains_key(user.clone()) {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_user_bounties_count(env: Env, user: Address) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut count = 0;
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.submissions.contains_key(user.clone()) {
                count += 1;
            }
        }
        count
    }

    pub fn get_owner_bounties(env: Env, owner: Address) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut bounties = Vec::new(&env);
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.owner == owner {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_owner_bounties_count(env: Env, owner: Address) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut count = 0;
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.owner == owner {
                count += 1;
            }
        }
        count
    }

    pub fn get_bounties_by_token(env: Env, token: Address) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut bounties = Vec::new(&env);
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.token == token {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_bounties_by_token_count(env: Env, token: Address) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut count = 0;
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.token == token {
                count += 1;
            }
        }
        count
    }

    pub fn get_active_bounties(env: Env) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut active = Vec::new(&env);

        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.status == Status::Active {
                active.push_back(id);
            }
        }
        active
    }

    pub fn get_bounties_count(env: Env) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        next_id
    }

    pub fn get_bounties_by_status(env: Env, status: Status) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut bounties = Vec::new(&env);
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.status == status {
                bounties.push_back(id);
            }
        }
        bounties
    }

    pub fn get_bounties_by_status_count(env: Env, status: Status) -> u32 {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let mut count = 0;
        for id in 0..next_id {
            let bounty: Bounty = storage.get(&bounty_key(id)).unwrap();
            if bounty.status == status {
                count += 1;
            }
        }
        count
    }

    pub fn get_bounty(env: Env, bounty_id: u32) -> Result<Bounty, Error> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        Ok(bounty)
    }

    pub fn get_bounty_submissions(env: Env, bounty_id: u32) -> Map<Address, String> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        bounty.submissions
    }

    pub fn get_bounty_applicants(env: Env, bounty_id: u32) -> Vec<Address> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        bounty.applicants
    }

    pub fn get_bounty_winners(env: Env, bounty_id: u32) -> Vec<Address> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        bounty.winners
    }

    pub fn get_bounty_status(env: Env, bounty_id: u32) -> Status {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        bounty.status
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
        token_client.transfer(&owner, &env.current_contract_address(), &reward);

        // Assign new bounty ID
        let id: u32 = storage.get(&next_id_key()).unwrap_or(0);
        let next = id + 1;
        storage.set(&next_id_key(), &next);

        // Initialize bounty
        let mut distribution_map = Map::new(&env);
        for (rank, percent) in distribution.iter() {
            distribution_map.set(rank, percent);
        }
        let bounty = Bounty {
            owner: owner.clone(),
            token: token.clone(),
            reward,
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
        let mut bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();

        // Only the bounty owner can update the bounty
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
        let bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();

        // Only the bounty owner can delete the bounty
        if bounty.owner != owner {
            return Err(Error::OnlyOwner);
        }

        // Can't delete if there are submissions
        if !bounty.submissions.is_empty() {
            return Err(Error::BountyHasSubmissions);
        }

        // Remove the bounty from storage
        storage.remove(&bounty_key(bounty_id));

        // Transfer remaining funds back to the owner
        let token_client = get_token_client(&env, bounty.token);
        token_client.transfer(
            &env.current_contract_address(),
            &bounty.owner,
            &bounty.reward,
        );

        Events::emit_bounty_deleted(&env, bounty_id);
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
        let mut bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
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
        let mut bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
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
            return Err(Error::InternalError);
        }
        
        // Update the submission
        bounty.submissions.set(applicant.clone(), new_submission_link);
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
        let mut bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        if bounty.owner != owner {
            return Err(Error::OnlyOwner);
        }
        let now = env.ledger().timestamp();
        if now > bounty.judging_deadline {
            return Err(Error::JudgingDeadlinePassed);
        }
        let num_spec = bounty.distribution.len();
        if winners.len() < num_spec {
            return Err(Error::NotEnoughWinners);
        }

        // Distribute rewards
        let fee = calculate_fee(bounty.reward);
        let net = bounty.reward - fee;
        let token_client = get_token_client(&env, bounty.token.clone());
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
                token_client.transfer(&env.current_contract_address(), &winner, &amount);
                distributed += amount;
            }
        }

        // Return remaining funds to owner (if any)
        let remaining = net - distributed;
        if remaining > 0 {
            token_client.transfer(&env.current_contract_address(), &bounty.owner, &remaining);
        }

        // Transfer platform fee to fee account instead of owner
        token_client.transfer(&env.current_contract_address(), &fee_account, &fee);

        bounty.status = Status::Completed;
        bounty.winners = winners.clone();
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_winners_selected(&env, bounty_id, winners);

        Ok(())
    }

    // Check and auto-distribute if judging deadline passed
    pub fn check_judging(env: Env, bounty_id: u32) -> Result<(), Error> {
        let storage = env.storage().persistent();
        let mut bounty: Bounty = storage.get(&bounty_key(bounty_id)).unwrap();
        let now = env.ledger().timestamp();
        if now <= bounty.judging_deadline || bounty.status != Status::Active {
            return Ok(());
        }
        // auto-distribute equally to all applicants
        let fee = calculate_fee(bounty.reward);
        let net = bounty.reward - fee;
        let count = bounty.applicants.len() as i128;
        if count == 0 {
            let token_client = get_token_client(&env, bounty.token.clone());
            token_client.transfer(
                &env.current_contract_address(),
                &bounty.owner,
                &bounty.reward,
            );
            return Ok(());
        }
        let share = net / count;
        let token_client = get_token_client(&env, bounty.token.clone());
        let fee_account = Self::get_fee_account(&env);
        for applicant in bounty.applicants.iter() {
            token_client.transfer(&env.current_contract_address(), &applicant, &share);
        }
        token_client.transfer(&env.current_contract_address(), &fee_account, &fee);

        bounty.status = Status::Completed;
        storage.set(&bounty_key(bounty_id), &bounty);
        Events::emit_auto_distributed(&env, bounty_id);

        Ok(())
    }
}

mod test;
