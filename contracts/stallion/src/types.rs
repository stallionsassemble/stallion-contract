use soroban_sdk::{
    Address, ConversionError, Env, Map, String, TryFromVal, Val, Vec, contracterror, contracttype,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    // Admin-related errors
    NotAdmin = 1,
    AdminCannotBeZero = 2,
    FeeAccountCannotBeZero = 3,
    SameFeeAccount = 4,
    
    // Authorization errors
    OnlyOwner = 5,
    Unauthorized = 6,
    
    // Bounty-related errors
    BountyNotFound = 7,
    InactiveBounty = 8,
    BountyDeadlinePassed = 9,
    JudgingDeadlinePassed = 10,
    BountyHasSubmissions = 11,
    CannotSelectWinnersBeforeSubmissionDeadline = 12,
    JudgingDeadlineMustBeAfterSubmissionDeadline = 13,
    NotEnoughWinners = 14,
    DistributionMustSumTo100 = 15,
    InvalidDeadlineUpdate = 16,
    
    // Submission-related errors
    SubmissionNotFound = 17,
    
    // Project-related errors
    ProjectNotFound = 18,
    InvalidProjectType = 19,
    ProjectNotActive = 20,
    InvalidMilestones = 21,
    
    // Milestone-related errors
    MilestoneNotFound = 22,
    MilestoneAlreadyPaid = 23,
    InsufficientEscrow = 24,
    
    // Validation errors
    InvalidReward = 25,
    InvalidAmount = 26,
    DeadlinePassed = 27,
    
    // System errors
    InternalError = 28,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Active,
    Completed,
    Closed
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectType {
    Gig,
    Job,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectStatus {
    Active,
    Completed,
    Cancelled,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneData {
    pub amount: i128,
    pub order: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneInfo {
    pub amount: i128,
    pub order: u32,
    pub is_paid: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub owner: Address,
    pub project_type: ProjectType,
    pub token: Address,
    pub total_reward: i128,
    pub remaining_escrow: i128,
    pub deadline: u64,
    pub status: ProjectStatus,
    pub milestones: Vec<MilestoneInfo>,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Token = 1,
    NextId = 2,
    Bounty = 3,
    Admin = 4,
    FeeAccount = 5,
    NextProjectId = 6,
    Project = 7,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}
