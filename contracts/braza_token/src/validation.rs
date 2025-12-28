#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::storage;
use crate::types::BrazaError;
use soroban_sdk::{Address, Env};
// Importamos o módulo inteiro para delegar as verificações
use crate::compliance;

//
// VALIDAÇÕES BÁSICAS (ADMIN, PAUSA, SALDO)
//

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
pub fn require_sufficient_balance(
    env: &Env,
    addr: &Address,
    required: i128,
) -> Result<(), BrazaError> {
    let bal = storage::get_balance(env, addr);
    if bal < required {
        return Err(BrazaError::InsufficientBalance);
    }
    Ok(())
}

/// Supply máximo não ultrapassado
pub fn require_max_supply_not_exceeded(env: &Env, add: i128) -> Result<(), BrazaError> {
    let current = storage::get_total_supply(env);
    let new_sup = current
        .checked_add(add)
        .ok_or(BrazaError::MaxSupplyExceeded)?;
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

//
// NOVAS FUNÇÕES DE VALIDAÇÃO
//

/// Valida que KYC level está no intervalo [1, 3]
pub fn validate_kyc_level_value(kyc_level: u32) -> Result<(), BrazaError> {
    if kyc_level == 0 || kyc_level > 3 {
        return Err(BrazaError::InvalidAmount); // Usar InvalidAmount, pois InvalidKycLevel não existe
    }
    Ok(())
}

/// Valida daily volume limit
pub fn require_daily_volume_limit(
    env: &Env,
    user: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    let daily_limit = compliance::get_daily_limit(env, user);

    // Se não tem limite (0 ou negativo), passa
    if daily_limit <= 0 {
        return Ok(());
    }

    let daily_volume = compliance::get_daily_volume(env, user);
    let new_volume = daily_volume
        .checked_add(amount)
        .ok_or(BrazaError::OverflowError)?; // Evita overflow ao somar

    if new_volume > daily_limit {
        return Err(BrazaError::InvalidAmount); // Ou criar um erro DailyLimitExceeded
    }

    Ok(())
}

//
//  COMPLIANCE — WRAPPERS (Single Source of Truth)
//  Delega para compliance.rs para garantir que a lógica estrita seja usada.
//

/// País permitido (Delega para compliance.rs que tem a lógica estrita)
pub fn require_country_allowed(env: &Env, user: &Address) -> Result<(), BrazaError> {
    compliance::require_country_allowed(env, user)
}

/// KYC mínimo (0-3)
pub fn require_kyc_level(env: &Env, user: &Address, min_level: u32) -> Result<(), BrazaError> {
    compliance::require_kyc_level(env, user, min_level)
}

/// Risco (AML)
pub fn require_acceptable_risk(
    env: &Env,
    user: &Address,
    max_allowed: u32,
) -> Result<(), BrazaError> {
    compliance::require_acceptable_risk(env, user, max_allowed)
}

//
// MÓDULO DE TESTES (EXPORTADO)
//

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub mod test_utils {

    // Re-exportar funções internas para testes de integração
    pub use super::{
        require_acceptable_risk, require_admin, require_country_allowed,
        require_daily_volume_limit, require_kyc_level, require_max_supply_not_exceeded,
        require_not_blacklisted, require_not_paused, require_positive_amount,
        require_sufficient_balance, require_valid_vesting_params, validate_kyc_level_value,
    };
}
