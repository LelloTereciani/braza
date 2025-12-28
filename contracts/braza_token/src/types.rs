#![allow(dead_code)]
use soroban_sdk::{contracterror, contracttype, String};

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
    InsufficientAllowance = 19,
    OverflowError = 20, // ← ADICIONAR ESTA LINHA
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

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_error_ordering() {
        assert!(BrazaError::AlreadyInitialized < BrazaError::Unauthorized);
        assert!(BrazaError::Unauthorized < BrazaError::InsufficientBalance);
        assert!(BrazaError::VestingAmountTooLow < BrazaError::InsufficientAllowance);
        assert!(BrazaError::InsufficientAllowance < BrazaError::OverflowError);
    }

    #[test]
    fn test_error_values() {
        assert_eq!(BrazaError::AlreadyInitialized as u32, 1);
        assert_eq!(BrazaError::InsufficientBalance as u32, 3);
        assert_eq!(BrazaError::InsufficientAllowance as u32, 19);
        assert_eq!(BrazaError::OverflowError as u32, 20);
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(
            BrazaError::InsufficientAllowance,
            BrazaError::InsufficientAllowance
        );
        assert_ne!(
            BrazaError::InsufficientAllowance,
            BrazaError::InsufficientBalance
        );
        assert_ne!(BrazaError::OverflowError, BrazaError::InvalidAmount);
    }

    #[test]
    fn test_error_clone() {
        let a = BrazaError::InsufficientAllowance;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_overflow_error_exists() {
        let err = BrazaError::OverflowError;
        assert_eq!(err as u32, 20);
    }

    #[test]
    fn test_vesting_schedule_clone() {
        let env = Env::default();
        let addr = soroban_sdk::Address::generate(&env);

        let s = VestingSchedule {
            beneficiary: addr.clone(),
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
