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
    DistributionMustSumTo100 = 5,
    InvalidDeadlineUpdate = 6,
    TooManyWinners = 7,
    NotAdmin = 8,
    AdminCannotBeZero = 9,
    FeeAccountCannotBeZero = 10,
    SameFeeAccount = 11,
    InvalidReward = 12,
    InternalError = 13,
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
    pub status: Status,
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
