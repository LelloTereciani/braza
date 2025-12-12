use soroban_sdk::{Address, Env, String, symbol_short};
use crate::storage;
use crate::types::BrazaError;
use crate::validation;

// ============================================================================
// FUNÇÕES DE CONFORMIDADE REGULATÓRIA
// ============================================================================

/// Níveis de KYC (Know Your Customer)
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum KYCLevel {
    None = 0,
    Basic = 1,
    Intermediate = 2,
    Advanced = 3,
}

// ============================================================================
// GESTÃO DE KYC
// ============================================================================

pub fn set_kyc_level(
    env: &Env,
    admin: &Address,
    user: &Address,
    level: u32,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    if level > 3 {
        return Err(BrazaError::InvalidAmount);
    }
    
    let key = (symbol_short!("kyc"), user);
    env.storage().persistent().set(&key, &level);
    
    let ts_key = (symbol_short!("kyc_ts"), user);
    env.storage().persistent().set(&ts_key, &env.ledger().sequence());
    
    env.events().publish(
        (symbol_short!("kyc_set"), user),
        level,
    );
    
    Ok(())
}

pub fn get_kyc_level(env: &Env, user: &Address) -> u32 {
    storage::bump_critical_storage(env);
    
    let key = (symbol_short!("kyc"), user);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(0)
}

pub fn require_kyc_level(
    env: &Env,
    user: &Address,
    min_level: u32,
) -> Result<(), BrazaError> {
    let user_level = get_kyc_level(env, user);
    
    if user_level < min_level {
        return Err(BrazaError::Unauthorized);
    }
    
    Ok(())
}

// ============================================================================
// GESTÃO DE INVESTIDORES CREDENCIADOS
// ============================================================================

pub fn set_accredited_investor(
    env: &Env,
    admin: &Address,
    investor: &Address,
    accredited: bool,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    if accredited {
        require_kyc_level(env, investor, 3)?;
    }
    
    let key = (symbol_short!("accred"), investor);
    env.storage().persistent().set(&key, &accredited);
    
    let ts_key = (symbol_short!("accr_ts"), investor);
    env.storage().persistent().set(&ts_key, &env.ledger().sequence());
    
    env.events().publish(
        (symbol_short!("accr_set"), investor),
        accredited,
    );
    
    Ok(())
}

pub fn is_accredited_investor(env: &Env, investor: &Address) -> bool {
    storage::bump_critical_storage(env);
    
    let key = (symbol_short!("accred"), investor);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(false)
}

// ============================================================================
// GESTÃO DE PAÍSES E RESTRIÇÕES GEOGRÁFICAS
// ============================================================================

pub fn set_country_code(
    env: &Env,
    admin: &Address,
    user: &Address,
    country_code: String,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    let key = (symbol_short!("country"), user);
    env.storage().persistent().set(&key, &country_code);
    
    env.events().publish(
        (symbol_short!("ctry_set"), user),
        country_code,
    );
    
    Ok(())
}

pub fn get_country_code(env: &Env, user: &Address) -> Option<String> {
    storage::bump_critical_storage(env);
    
    let key = (symbol_short!("country"), user);
    env.storage().persistent().get(&key)
}

pub fn add_blocked_country(
    env: &Env,
    admin: &Address,
    country_code: String,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    let key = (symbol_short!("blk_ctry"), country_code.clone());
    env.storage().persistent().set(&key, &true);
    
    env.events().publish(
        (symbol_short!("ctry_blk"),),
        country_code,
    );
    
    Ok(())
}

pub fn remove_blocked_country(
    env: &Env,
    admin: &Address,
    country_code: String,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    let key = (symbol_short!("blk_ctry"), country_code.clone());
    env.storage().persistent().remove(&key);
    
    env.events().publish(
        (symbol_short!("ctry_unb"),),
        country_code,
    );
    
    Ok(())
}

pub fn is_country_blocked(env: &Env, country_code: String) -> bool {
    storage::bump_critical_storage(env);
    
    let key = (symbol_short!("blk_ctry"), country_code);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(false)
}

pub fn require_country_allowed(env: &Env, user: &Address) -> Result<(), BrazaError> {
    if let Some(country) = get_country_code(env, user) {
        if is_country_blocked(env, country) {
            return Err(BrazaError::Unauthorized);
        }
    }
    
    Ok(())
}

// ============================================================================
// GESTÃO DE RISCO (AML)
// ============================================================================

pub fn set_risk_score(
    env: &Env,
    admin: &Address,
    user: &Address,
    score: u32,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    if score > 100 {
        return Err(BrazaError::InvalidAmount);
    }
    
    let key = (symbol_short!("risk"), user);
    env.storage().persistent().set(&key, &score);
    
    let ts_key = (symbol_short!("risk_ts"), user);
    env.storage().persistent().set(&ts_key, &env.ledger().sequence());
    
    if score > 80 {
        storage::set_blacklisted(env, user, true);
    }
    
    env.events().publish(
        (symbol_short!("risk_set"), user),
        score,
    );
    
    Ok(())
}

pub fn get_risk_score(env: &Env, user: &Address) -> u32 {
    storage::bump_critical_storage(env);
    
    let key = (symbol_short!("risk"), user);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(0)
}

pub fn require_acceptable_risk(
    env: &Env,
    user: &Address,
    max_risk: u32,
) -> Result<(), BrazaError> {
    let risk = get_risk_score(env, user);
    
    if risk > max_risk {
        return Err(BrazaError::Unauthorized);
    }
    
    Ok(())
}

// ============================================================================
// LIMITES DE TRANSAÇÃO
// ============================================================================

pub fn set_daily_limit(
    env: &Env,
    admin: &Address,
    user: &Address,
    limit: i128,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    if limit <= 0 {
        return Err(BrazaError::InvalidAmount);
    }
    
    let key = (symbol_short!("day_lim"), user);
    env.storage().persistent().set(&key, &limit);
    
    env.events().publish(
        (symbol_short!("lim_set"), user),
        limit,
    );
    
    Ok(())
}

pub fn get_daily_limit(env: &Env, user: &Address) -> i128 {
    storage::bump_critical_storage(env);
    
    let key = (symbol_short!("day_lim"), user);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(i128::MAX)
}

pub fn check_and_update_daily_volume(
    env: &Env,
    user: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    storage::bump_critical_storage(env);
    
    let current_ledger = env.ledger().sequence();
    let current_day = current_ledger / 17_280;
    
    let day_key = (symbol_short!("vol_day"), user);
    let last_day: u32 = env.storage()
        .persistent()
        .get(&day_key)
        .unwrap_or(0);
    
    let amt_key = (symbol_short!("vol_amt"), user);
    let mut current_volume: i128 = if last_day == current_day {
        env.storage()
            .persistent()
            .get(&amt_key)
            .unwrap_or(0)
    } else {
        0
    };
    
    current_volume = current_volume
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;
    
    let limit = get_daily_limit(env, user);
    if current_volume > limit {
        return Err(BrazaError::Unauthorized);
    }
    
    env.storage().persistent().set(&day_key, &current_day);
    env.storage().persistent().set(&amt_key, &current_volume);
    
    Ok(())
}

pub fn get_daily_volume(env: &Env, user: &Address) -> i128 {
    storage::bump_critical_storage(env);
    
    let current_ledger = env.ledger().sequence();
    let current_day = current_ledger / 17_280;
    
    let day_key = (symbol_short!("vol_day"), user);
    let last_day: u32 = env.storage()
        .persistent()
        .get(&day_key)
        .unwrap_or(0);
    
    if last_day == current_day {
        let amt_key = (symbol_short!("vol_amt"), user);
        env.storage()
            .persistent()
            .get(&amt_key)
            .unwrap_or(0)
    } else {
        0
    }
}

pub fn is_fully_compliant(env: &Env, user: &Address) -> bool {
    storage::bump_critical_storage(env);
    
    if storage::is_blacklisted(env, user) {
        return false;
    }
    
    if get_kyc_level(env, user) < 2 {
        return false;
    }
    
    if let Some(country) = get_country_code(env, user) {
        if is_country_blocked(env, country) {
            return false;
        }
    }
    
    if get_risk_score(env, user) >= 50 {
        return false;
    }
    
    true
}
