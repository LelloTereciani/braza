use soroban_sdk::{contracttype, contracterror, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BrazaError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    InsufficientBalance = 3,
    InvalidAmount = 4,
    Paused = 5,
    Blacklisted = 6,
    MaxSupplyExceeded = 7,
    VestingNotFound = 8,
    VestingAlreadyReleased = 9,
    CliffNotReached = 10,
    NotRevocable = 11,
    MaxVestingSchedulesExceeded = 12,
    InvalidVestingParams = 13,
    NoTokensToRelease = 14,
    TimelockNotExpired = 15,
    GlobalVestingLimitExceeded = 16,
    VestingCooldownActive = 17,    
    VestingAmountTooLow = 18,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub beneficiary: soroban_sdk::Address,
    pub total_amount: i128,
    pub released_amount: i128,
    pub start_ledger: u32,
    pub cliff_ledgers: u32,
    pub duration_ledgers: u32,
    pub revocable: bool,
    pub revoked: bool,
}
