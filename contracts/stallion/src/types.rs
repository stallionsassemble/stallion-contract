use soroban_sdk::{
    Address, ConversionError, Env, Map, String, TryFromVal, Val, Vec, contracterror, contracttype,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    OnlyOwner = 1,
    InactiveBounty = 2,
    BountyDeadlinePassed = 3,
    BountyNotFound = 4,
    SubmissionNotFound = 5,
    JudgingDeadlinePassed = 6,
    DistributionMustSumTo100 = 7,
    JudgingDeadlineMustBeAfterSubmissionDeadline = 8,
    NotEnoughWinners = 9,
    InternalError = 10,
    NotAdmin = 11,
    AdminCannotBeZero = 12,
    FeeAccountCannotBeZero = 13,
    BountyHasSubmissions = 14,
    InvalidDeadlineUpdate = 15,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Active,
    InReview,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bounty {
    pub owner: Address,
    pub title: String,
    pub reward: i128,
    pub token: Address,
    pub distribution: Map<u32, u32>,
    pub submission_deadline: u64,
    pub judging_deadline: u64,
    pub status: Status,
    pub applicants: Vec<Address>,
    pub submissions: Map<Address, String>,
    pub winners: Vec<Address>,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Token = 1,
    NextId = 2,
    Bounty = 3,
    Admin = 4,
    FeeAccount = 5,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}
