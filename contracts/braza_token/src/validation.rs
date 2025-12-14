use soroban_sdk::{Address, Env};
use crate::storage;
use crate::types::BrazaError;

// ============================================================================
// VALIDAÇÕES — CORRIGIDO E REFORÇADO (CEI + COMPLIANCE)
// ============================================================================

/// Admin somente
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), BrazaError> {
    let admin = storage::get_admin(env);
    if caller != &admin {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

/// Contrato não pausado
pub fn require_not_paused(env: &Env) -> Result<(), BrazaError> {
    if storage::is_paused(env) {
        return Err(BrazaError::Paused);
    }
    Ok(())
}

/// Endereço não está na blacklist
pub fn require_not_blacklisted(env: &Env, addr: &Address) -> Result<(), BrazaError> {
    if storage::is_blacklisted(env, addr) {
        return Err(BrazaError::Blacklisted);
    }
    Ok(())
}

/// valor > 0
pub fn require_positive_amount(amount: i128) -> Result<(), BrazaError> {
    if amount <= 0 {
        return Err(BrazaError::InvalidAmount);
    }
    Ok(())
}

/// Balance suficiente
pub fn require_sufficient_balance(env: &Env, addr: &Address, required: i128) -> Result<(), BrazaError> {
    let bal = storage::get_balance(env, addr);
    if bal < required {
        return Err(BrazaError::InsufficientBalance);
    }
    Ok(())
}

/// Supply máximo não ultrapassado
pub fn require_max_supply_not_exceeded(env: &Env, add: i128) -> Result<(), BrazaError> {
    let current = storage::get_total_supply(env);
    let new_sup = current.checked_add(add).ok_or(BrazaError::MaxSupplyExceeded)?;
    if new_sup > storage::MAX_SUPPLY {
        return Err(BrazaError::MaxSupplyExceeded);
    }
    Ok(())
}

/// Parâmetros de vesting válidos
pub fn require_valid_vesting_params(
    total: i128,
    cliff: u32,
    duration: u32,
) -> Result<(), BrazaError> {

    if total <= 0 {
        return Err(BrazaError::InvalidAmount);
    }

    if duration == 0 {
        return Err(BrazaError::InvalidVestingParams);
    }

    if cliff > duration {
        return Err(BrazaError::InvalidVestingParams);
    }

    // Mínimo de 1 BRZ (1e7)
    if total < storage::MIN_VESTING_AMOUNT {
        return Err(BrazaError::VestingAmountTooLow);
    }

    Ok(())
}

// ============================================================================
//  COMPLIANCE — INTEGRADO (NOVO)
//  Estas funções já existiam nos módulos, mas o validation.rs PRECISA expor
//  wrappers uniformes para manter o padrão CEI centralizado.
// ============================================================================

/// País permitido
pub fn require_country_allowed(env: &Env, user: &Address) -> Result<(), BrazaError> {
    if let Some(country) = storage::get_country_code(env, user) {
        if country.len() == 0 {
            return Err(BrazaError::Unauthorized);
        }
        if storage::is_country_blocked(env, country) {
            return Err(BrazaError::Unauthorized);
        }
    }
    Ok(())
}

/// KYC mínimo (0-3)
pub fn require_kyc_level(env: &Env, user: &Address, min_level: u32) -> Result<(), BrazaError> {
    let lvl = storage::get_kyc_level(env, user);
    if lvl < min_level {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

/// Risco (AML)
pub fn require_acceptable_risk(env: &Env, user: &Address, max_allowed: u32) -> Result<(), BrazaError> {
    let risk = storage::get_risk_score(env, user);
    if risk > max_allowed {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}
