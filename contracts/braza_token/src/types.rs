#![allow(dead_code)]
use soroban_sdk::{contracttype, contracterror, String};

// ============================================================================
// ERROS DO CONTRATO
// ============================================================================
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

    // Allowance insuficiente para transfer_from
    InsufficientAllowance = 19,
}

// ============================================================================
// METADADOS DO TOKEN
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

// ============================================================================
// VESTING SCHEDULE
// ============================================================================

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

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, Address, String};  // <- Imports necessários para o teste

    #[test]
    fn test_error_ordering() {
        assert!(BrazaError::AlreadyInitialized < BrazaError::Unauthorized);
        assert!(BrazaError::Unauthorized < BrazaError::InsufficientBalance);
        assert!(BrazaError::VestingAmountTooLow < BrazaError::InsufficientAllowance);
    }

    #[test]
    fn test_error_values() {
        assert_eq!(BrazaError::AlreadyInitialized as u32, 1);
        assert_eq!(BrazaError::InsufficientBalance as u32, 3);
        assert_eq!(BrazaError::InsufficientAllowance as u32, 19);
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(BrazaError::InsufficientAllowance, BrazaError::InsufficientAllowance);
        assert_ne!(BrazaError::InsufficientAllowance, BrazaError::InsufficientBalance);
    }

    #[test]
    fn test_error_clone() {
        let a = BrazaError::InsufficientAllowance;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_vesting_schedule_clone() {
        let env = Env::default();
        let addr = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));  // <- Atribuído a addr

        let s = VestingSchedule {
            beneficiary: addr.clone(),  // <- Agora addr existe e pode ser clonado
            total_amount: 1000,
            released_amount: 0,
            start_ledger: 0,
            cliff_ledgers: 100,
            duration_ledgers: 1000,
            revocable: true,
            revoked: false,
        };

        assert_eq!(s.clone(), s);
    }
}
