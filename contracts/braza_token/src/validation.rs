use soroban_sdk::{Address, Env};
use crate::storage;
use crate::types::BrazaError;

// ============================================================================
// VALIDAÇÕES - CRITICAL-02 CORRIGIDO (CEI Pattern)
// ============================================================================

/// Valida se o caller é o admin
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), BrazaError> {
    let admin = storage::get_admin(env);
    if caller != &admin {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

/// Valida se o contrato não está pausado
pub fn require_not_paused(env: &Env) -> Result<(), BrazaError> {
    if storage::is_paused(env) {
        return Err(BrazaError::Paused);
    }
    Ok(())
}

/// Valida se o endereço não está na blacklist
pub fn require_not_blacklisted(env: &Env, addr: &Address) -> Result<(), BrazaError> {
    if storage::is_blacklisted(env, addr) {
        return Err(BrazaError::Blacklisted);
    }
    Ok(())
}

/// Valida se o amount é válido (> 0)
pub fn require_positive_amount(amount: i128) -> Result<(), BrazaError> {
    if amount <= 0 {
        return Err(BrazaError::InvalidAmount);
    }
    Ok(())
}

/// Valida se o balance é suficiente
pub fn require_sufficient_balance(env: &Env, addr: &Address, required: i128) -> Result<(), BrazaError> {
    let balance = storage::get_balance(env, addr);
    if balance < required {
        return Err(BrazaError::InsufficientBalance);
    }
    Ok(())
}

/// Valida se o supply máximo não será excedido
pub fn require_max_supply_not_exceeded(env: &Env, additional_amount: i128) -> Result<(), BrazaError> {
    let current_supply = storage::get_total_supply(env);
    let new_supply = current_supply
        .checked_add(additional_amount)
        .ok_or(BrazaError::MaxSupplyExceeded)?;
    
    if new_supply > storage::MAX_SUPPLY {
        return Err(BrazaError::MaxSupplyExceeded);
    }
    
    Ok(())
}

/// Valida parâmetros de vesting
pub fn require_valid_vesting_params(
    total_amount: i128,
    cliff_ledgers: u32,
    duration_ledgers: u32,
) -> Result<(), BrazaError> {
    if total_amount <= 0 {
        return Err(BrazaError::InvalidAmount);
    }
    
    if duration_ledgers == 0 {
        return Err(BrazaError::InvalidVestingParams);
    }
    
    if cliff_ledgers > duration_ledgers {
        return Err(BrazaError::InvalidVestingParams);
    }
    
    Ok(())
}
