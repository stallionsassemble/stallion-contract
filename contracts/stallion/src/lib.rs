//! Soroban smart contract for Stallion decentralized bounty platform
// SPDX-License-Identifier: MIT

#![no_std]

use soroban_sdk::{
    Address, Env, Map, String, Symbol, Vec, contract, contracterror, contractimpl, contracttype,
    symbol_short,
};

// Error enumeration
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    OnlyOwner = 1,
    InactiveBounty = 2,
    BountyDeadlinePassed = 3,
    JudgingDeadlinePassed = 4,
    DistributionMustSumTo100 = 5,
    JudgingDeadlineMustBeAfterSubmissionDeadline = 6,
    NotEnoughWinners = 7,
    NotEnoughApplicants = 8,
    InternalError = 9,
}

// Bounty status enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum Status {
    Active,
    Judging,
    WinnersSelected,
}

// Bounty data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct Bounty {
    owner: Address,
    reward: i128,                // total reward in stroops
    distribution: Map<u32, u32>, // rank -> percent (parts per hundred)
    submission_deadline: u64,    // ledger timestamp
    judging_deadline: u64,       // ledger timestamp
    description: String,
    status: Status,
    applicants: Vec<Address>,
    submissions: Map<Address, Symbol>, // applicant -> link
    winners: Vec<Address>,
}

// Storage keys
const NEXT_ID: Symbol = symbol_short!("NEXT_ID");
const BOUNTY: Symbol = symbol_short!("BOUNTY");

#[contract]
pub struct StallionContract;

#[contractimpl]
impl StallionContract {
    // Create a new bounty
    pub fn create_bounty(
        env: Env,
        owner: Address,
        reward: i128,
        distribution: Vec<(u32, u32)>, // (rank, percent)
        submission_deadline: u64,
        judging_deadline: u64,
        description: String,
    ) -> Result<u32, Error> {
        let storage = env.storage().persistent();

        // Validate distribution sums to 100
        let mut total: u32 = 0;
        let mut map = Map::new(&env);
        for pair in distribution.iter() {
            let (rank, pct) = pair;
            total += pct;
            map.set(rank, pct);
        }
        if total != 100 {
            return Err(Error::DistributionMustSumTo100);
        }
        // Validate deadlines
        if judging_deadline <= submission_deadline {
            return Err(Error::JudgingDeadlineMustBeAfterSubmissionDeadline);
        }

        // Transfer reward to contract
        owner.require_auth();
        // TODO: Transfer to contract

        // Assign new bounty ID
        let id: u32 = storage
            .get::<Symbol, Result<u32, soroban_sdk::Error>>(&NEXT_ID)
            .unwrap_or(Ok(0))
            .unwrap();
        let next = id + 1;
        storage.set(&NEXT_ID, &next);

        // Initialize bounty
        let bounty = Bounty {
            owner: owner.clone(),
            reward,
            distribution: map.clone(),
            submission_deadline,
            judging_deadline,
            description: description.clone(),
            status: Status::Active,
            applicants: Vec::new(&env),
            submissions: Map::new(&env),
            winners: Vec::new(&env),
        };
        storage.set(&(BOUNTY, id), &bounty);
        // Emit event
        env.events()
            .publish((Symbol::new(&env, "BountyCreated"),), id);

        Ok(id)
    }

    // Get bounty details
    pub fn get_bounty(env: Env, bounty_id: u32) -> Result<Bounty, Error> {
        let storage = env.storage().persistent();
        let bounty: Bounty = storage
            .get::<(Symbol, u32), Bounty>(&(BOUNTY, bounty_id))
            .unwrap();
        Ok(bounty)
    }

    // Apply to an active bounty
    pub fn apply(
        env: Env,
        applicant: Address,
        bounty_id: u32,
        submission_link: Symbol,
    ) -> Result<(), Error> {
        applicant.require_auth();

        let storage = env.storage().persistent();
        let mut bounty: Bounty = storage
            .get::<(Symbol, u32), Bounty>(&(BOUNTY, bounty_id))
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
        storage.set(&(BOUNTY, bounty_id), &bounty);
        env.events().publish(
            (Symbol::new(&env, "SubmissionAdded"),),
            (bounty_id, applicant),
        );

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
            .get::<(Symbol, u32), Bounty>(&(BOUNTY, bounty_id))
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
        if winners.len() as u32 > bounty.applicants.len() {
            return Err(Error::NotEnoughApplicants); // TODO: Make this so that an error is not thrown but instead rewards are distributed to applicants and the rest returned to owner
        }
        // Distribute rewards
        let fee = bounty.reward / 10; // 10%
        let net = bounty.reward - fee;
        let mut idx = 0u32;
        for rank in bounty.distribution.keys() {
            let pct = bounty.distribution.get(rank).unwrap();
            let amount = net * (pct as i128) / 100;
            let winner = winners.get(idx).unwrap();
            // TODO: Transfer to winner
            // env.transfer(&Address::Contract, winner, amount);
            idx += 1;
        }
        // TODO: Fee transfer
        // env.transfer(&Address::Contract, &bounty.owner, fee);

        bounty.status = Status::WinnersSelected;
        bounty.winners = winners.clone();
        storage.set(&(BOUNTY, bounty_id), &bounty);

        let winners_selected_key: Symbol = Symbol::new(&env, "WinnersSelected");
        env.events()
            .publish((winners_selected_key,), (bounty_id, winners));

        Ok(())
    }

    // Check and auto-distribute if judging deadline passed
    pub fn check_judging(env: Env, bounty_id: u32) -> Result<(), Error> {
        let storage = env.storage().persistent();
        let mut bounty: Bounty = storage
            .get::<(Symbol, u32), Bounty>(&(BOUNTY, bounty_id))
            .unwrap();
        let now = env.ledger().timestamp();
        if now <= bounty.judging_deadline || bounty.status != Status::Active {
            return Ok(());
        }
        // auto-distribute equally to all applicants
        let fee = bounty.reward / 10;
        let net = bounty.reward - fee;
        let count = bounty.applicants.len() as i128;
        if count == 0 {
            // TODO: Return reward to owner
            return Ok(());
        }
        let share = net / count;
        for applicant in bounty.applicants.iter() {
            // TODO: Transfer to applicant
            // env.transfer(&Address::Contract, &applicant, share);
        }
        // TODO: Fee transfer
        // env.transfer(&Address::Contract, &bounty.owner, fee);

        bounty.status = Status::WinnersSelected;
        storage.set(&(BOUNTY, bounty_id), &bounty);
        env.events()
            .publish((Symbol::new(&env, "AutoDistributed"),), bounty_id);

        Ok(())
    }
}

mod test;
