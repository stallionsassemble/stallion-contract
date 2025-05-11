// SPDX-License-Identifier: Boost Software License, Version 1.0.0
#![no_std]

use soroban_sdk::{
    Address, Env, Map, String, Symbol, Vec, contract, contractimpl, contractmeta, token,
};

mod events;
mod types;
mod utils;

use events::Events;
use types::{Bounty, DataKey, Error, Status};
use utils::{calculate_fee, get_token_client, validate_distribution_sum};

contractmeta!(
    key = "Description",
    val = "Soroban smart contract for Stallion decentralized bounty platform"
);

#[contract]
pub struct StallionContract;

#[contractimpl]
impl StallionContract {
    // Initialize contract with token
    pub fn __constructor(env: Env, token: Address) {
        let storage = env.storage().persistent();
        storage.set(&DataKey::Token, &token);
    }

    fn token_client(env: &Env) -> token::Client {
        let token: Address = env.storage().persistent().get(&DataKey::Token).unwrap();
        get_token_client(env, token)
    }

    pub fn get_bounty(env: Env, bounty_id: u32) -> Result<Bounty, Error> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage
            .get::<(DataKey, u32), Bounty>(&(DataKey::Bounty, bounty_id))
            .unwrap();
        Ok(bounty)
    }

    pub fn get_bounty_submissions(env: Env, bounty_id: u32) -> Map<Address, Symbol> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&(DataKey::Bounty, bounty_id)).unwrap();
        bounty.submissions
    }

    pub fn get_bounty_applicants(env: Env, bounty_id: u32) -> Vec<Address> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&(DataKey::Bounty, bounty_id)).unwrap();
        bounty.applicants
    }

    pub fn get_bounty_winners(env: Env, bounty_id: u32) -> Vec<Address> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&(DataKey::Bounty, bounty_id)).unwrap();
        bounty.winners
    }

    pub fn get_bounty_status(env: Env, bounty_id: u32) -> Status {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage.get(&(DataKey::Bounty, bounty_id)).unwrap();
        bounty.status
    }

    pub fn create_bounty(
        env: Env,
        owner: Address,
        reward: i128,
        distribution: Vec<(u32, u32)>,
        submission_deadline: u64,
        judging_deadline: u64,
        description: String,
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
        let token_client = Self::token_client(&env);
        token_client.transfer(&owner, &env.current_contract_address(), &reward);

        // Assign new bounty ID
        let id: u32 = storage
            .get::<DataKey, Result<u32, soroban_sdk::Error>>(&DataKey::NextId)
            .unwrap_or(Ok(0))
            .unwrap();
        let next = id + 1;
        storage.set(&DataKey::NextId, &next);

        // Initialize bounty
        let mut distribution_map = Map::new(&env);
        for (rank, percent) in distribution.iter() {
            distribution_map.set(rank, percent);
        }
        let bounty = Bounty {
            owner: owner.clone(),
            reward,
            distribution: distribution_map,
            submission_deadline,
            judging_deadline,
            description: description.clone(),
            status: Status::Active,
            applicants: Vec::new(&env),
            submissions: Map::new(&env),
            winners: Vec::new(&env),
        };
        storage.set(&(DataKey::Bounty, id), &bounty);
        Events::bounty_created(&env, id);

        Ok(id)
    }

    // Apply to an active bounty
    pub fn apply_to_bounty(
        env: Env,
        applicant: Address,
        bounty_id: u32,
        submission_link: Symbol,
    ) -> Result<(), Error> {
        applicant.require_auth();

        let storage = env.storage().persistent();
        let mut bounty: Bounty = storage
            .get::<(DataKey, u32), Bounty>(&(DataKey::Bounty, bounty_id))
            .unwrap();
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
        storage.set(&(DataKey::Bounty, bounty_id), &bounty);
        Events::submission_added(&env, bounty_id, applicant);

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
        let mut bounty: Bounty = storage
            .get::<(DataKey, u32), Bounty>(&(DataKey::Bounty, bounty_id))
            .unwrap();
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
        let token_client = Self::token_client(&env);

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

        // Transfer platform fee
        token_client.transfer(&env.current_contract_address(), &bounty.owner, &fee);

        bounty.status = Status::WinnersSelected;
        bounty.winners = winners.clone();
        storage.set(&(DataKey::Bounty, bounty_id), &bounty);
        Events::winners_selected(&env, bounty_id, winners);

        Ok(())
    }

    // Check and auto-distribute if judging deadline passed
    pub fn check_judging(env: Env, bounty_id: u32) -> Result<(), Error> {
        let storage = env.storage().persistent();
        let mut bounty: Bounty = storage
            .get::<(DataKey, u32), Bounty>(&(DataKey::Bounty, bounty_id))
            .unwrap();
        let now = env.ledger().timestamp();
        if now <= bounty.judging_deadline || bounty.status != Status::Active {
            return Ok(());
        }
        // auto-distribute equally to all applicants
        let fee = calculate_fee(bounty.reward);
        let net = bounty.reward - fee;
        let count = bounty.applicants.len() as i128;
        if count == 0 {
            let token_client = Self::token_client(&env);
            token_client.transfer(
                &env.current_contract_address(),
                &bounty.owner,
                &bounty.reward,
            );
            return Ok(());
        }
        let share = net / count;
        let token_client = Self::token_client(&env);
        for applicant in bounty.applicants.iter() {
            token_client.transfer(&env.current_contract_address(), &applicant, &share);
        }
        token_client.transfer(&env.current_contract_address(), &bounty.owner, &fee);

        bounty.status = Status::WinnersSelected;
        storage.set(&(DataKey::Bounty, bounty_id), &bounty);
        Events::auto_distributed(&env, bounty_id);

        Ok(())
    }

    pub fn get_active_bounties(env: Env) -> Vec<u32> {
        let storage = env.storage().persistent();
        let next_id: u32 = storage.get::<DataKey, u32>(&DataKey::NextId).unwrap_or(0);
        let mut active = Vec::new(&env);

        for id in 0..next_id {
            let bounty = storage
                .get::<(DataKey, u32), Bounty>(&(DataKey::Bounty, id))
                .unwrap();
            if bounty.status == Status::Active {
                active.push_back(id);
            }
        }
        active
    }
}

mod test;
