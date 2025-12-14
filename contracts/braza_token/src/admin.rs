use soroban_sdk::{Address, Env, String, Vec};
use crate::storage;
use crate::types::BrazaError;
use crate::validation;
use crate::events;

// ============================================================================
// CONSTANTES DE SEGURANÇA
// ============================================================================

const MINT_TIMELOCK_LEDGERS: u32 = 17280;
const BURN_TIMELOCK_LEDGERS: u32 = 17280;
const MAX_MINT_PER_TX: i128 = 100_000_000_000_000;
const MAX_BURN_PER_TX: i128 = 100_000_000_000_000;

// ============================================================================
// FUNÇÕES DE MINT E BURN COM PROTEÇÃO TIMELOCK
// ============================================================================

pub fn mint(
    env: &Env,
    admin: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;
    validation::require_positive_amount(amount)?;

    // COMPLIANCE — integração TOTAL
    validation::require_not_blacklisted(env, to)?;
    validation::require_country_allowed(env, to)?;
    validation::require_kyc_level(env, to, 0)?;
    validation::require_acceptable_risk(env, to, amount.try_into().unwrap())?;
    

    let last_mint_time = storage::get_last_mint_time(env);
    let current_ledger = env.ledger().sequence();

    if let Some(last_time) = last_mint_time {
        let elapsed = current_ledger.saturating_sub(last_time);
        if elapsed < MINT_TIMELOCK_LEDGERS {
            let rest = MINT_TIMELOCK_LEDGERS.saturating_sub(elapsed);
            env.events().publish(
                (soroban_sdk::symbol_short!("mint_blck"),),
                (rest, amount, to),
            );
            return Err(BrazaError::TimelockNotExpired);
        }
    }

    if amount > MAX_MINT_PER_TX {
        env.events().publish(
            (soroban_sdk::symbol_short!("mint_lmt"),),
            (amount, MAX_MINT_PER_TX),
        );
        return Err(BrazaError::InvalidAmount);
    }

    let current_supply = storage::get_total_supply(env);
    let new_supply = current_supply
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    if new_supply > storage::MAX_SUPPLY {
        env.events().publish(
            (soroban_sdk::symbol_short!("mint_cap"),),
            (new_supply, storage::MAX_SUPPLY),
        );
        return Err(BrazaError::InvalidAmount);
    }

    let current_balance = storage::get_balance(env, to);
    let new_balance = current_balance
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_balance(env, to, new_balance);
    storage::set_total_supply(env, new_supply);
    storage::set_last_mint_time(env, current_ledger);

    events::emit_mint(env, to, amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("mint_ok"), to, admin),
        (amount, current_ledger, new_supply),
    );

    Ok(())
}

pub fn burn(
    env: &Env,
    admin: &Address,
    from: &Address,
    amount: i128,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;
    validation::require_positive_amount(amount)?;
    validation::require_sufficient_balance(env, from, amount)?;

    // COMPLIANCE — agora obrigatório
    validation::require_not_blacklisted(env, from)?;
    validation::require_country_allowed(env, from)?;
    validation::require_kyc_level(env, from, 0)?;
    validation::require_acceptable_risk(env, from, amount.try_into().unwrap())?;
    

    let last_burn_time = storage::get_last_burn_time(env);
    let current_ledger = env.ledger().sequence();

    if let Some(last_time) = last_burn_time {
        let elapsed = current_ledger.saturating_sub(last_time);
        if elapsed < BURN_TIMELOCK_LEDGERS {
            let rest = BURN_TIMELOCK_LEDGERS.saturating_sub(elapsed);
            env.events().publish(
                (soroban_sdk::symbol_short!("burn_blck"),),
                (rest, amount, from),
            );
            return Err(BrazaError::TimelockNotExpired);
        }
    }

    if amount > MAX_BURN_PER_TX {
        env.events().publish(
            (soroban_sdk::symbol_short!("burn_lmt"),),
            (amount, MAX_BURN_PER_TX),
        );
        return Err(BrazaError::InvalidAmount);
    }

    let current_balance = storage::get_balance(env, from);
    let new_balance = current_balance
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;

    let current_supply = storage::get_total_supply(env);
    let new_supply = current_supply
        .checked_sub(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_balance(env, from, new_balance);
    storage::set_total_supply(env, new_supply);
    storage::set_last_burn_time(env, current_ledger);

    events::emit_burn(env, from, amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("burn_ok"), from, admin),
        (amount, current_ledger, new_supply),
    );

    Ok(())
}

// ============================================================================
// FUNÇÕES AUXILIARES DE CONSULTA
// ============================================================================

pub fn get_next_mint_available(env: &Env) -> Option<u32> {
    storage::bump_critical_storage(env);

    let last = storage::get_last_mint_time(env)?;
    let now = env.ledger().sequence();
    let elapsed = now.saturating_sub(last);

    if elapsed < MINT_TIMELOCK_LEDGERS {
        Some(MINT_TIMELOCK_LEDGERS.saturating_sub(elapsed))
    } else {
        None
    }
}

pub fn get_next_burn_available(env: &Env) -> Option<u32> {
    storage::bump_critical_storage(env);

    let last = storage::get_last_burn_time(env)?;
    let now = env.ledger().sequence();
    let elapsed = now.saturating_sub(last);

    if elapsed < BURN_TIMELOCK_LEDGERS {
        Some(BURN_TIMELOCK_LEDGERS.saturating_sub(elapsed))
    } else {
        None
    }
}

pub fn get_mint_burn_stats(env: &Env) -> (Option<u32>, Option<u32>, i128, i128) {
    storage::bump_critical_storage(env);

    (
        storage::get_last_mint_time(env),
        storage::get_last_burn_time(env),
        MAX_MINT_PER_TX,
        MAX_BURN_PER_TX,
    )
}

// ============================================================================
// FUNÇÕES ADMINISTRATIVAS AVANÇADAS
// ============================================================================

pub fn transfer_ownership(
    env: &Env,
    current_admin: &Address,
    new_admin: &Address,
) -> Result<(), BrazaError> {

    current_admin.require_auth();
    new_admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, current_admin)?;
    validation::require_not_blacklisted(env, new_admin)?;
    validation::require_country_allowed(env, new_admin)?;
    validation::require_kyc_level(env, new_admin, 0)?;

    if new_admin == current_admin {
        return Err(BrazaError::InvalidAmount);
    }

    let old = storage::get_admin(env);
    storage::set_admin(env, new_admin);

    env.events().publish(
        (soroban_sdk::symbol_short!("owner_chg"), old, new_admin),
        true,
    );

    Ok(())
}

pub fn update_metadata(
    env: &Env,
    admin: &Address,
    new_name: String,
    new_symbol: String,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;

    let mut metadata = storage::get_metadata(env);
    metadata.name = new_name.clone();
    metadata.symbol = new_symbol.clone();

    storage::set_metadata(env, &metadata);

    env.events().publish(
        (soroban_sdk::symbol_short!("meta_upd"),),
        (new_name, new_symbol),
    );

    Ok(())
}

pub fn recover_tokens(
    env: &Env,
    admin: &Address,
    token: &Address,
    amount: i128,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;
    validation::require_positive_amount(amount)?;
    validation::require_not_paused(env)?;

    env.events().publish(
        (soroban_sdk::symbol_short!("recover"), token, admin),
        amount,
    );

    Ok(())
}

pub fn emergency_pause(
    env: &Env,
    admin: &Address,
    reason: String,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    storage::set_paused(env, true);

    env.storage().instance().set(
        &soroban_sdk::symbol_short!("pause_rsn"),
        &reason,
    );

    env.storage().instance().set(
        &soroban_sdk::symbol_short!("pause_ts"),
        &env.ledger().sequence(),
    );

    events::emit_pause(env);

    env.events().publish(
        (soroban_sdk::symbol_short!("emerg_pse"),),
        reason,
    );

    Ok(())
}

pub fn resume_operations(env: &Env, admin: &Address) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    let start: Option<u32> = env.storage().instance().get(&soroban_sdk::symbol_short!("pause_ts"));

    storage::set_paused(env, false);
    env.storage().instance().remove(&soroban_sdk::symbol_short!("pause_rsn"));
    env.storage().instance().remove(&soroban_sdk::symbol_short!("pause_ts"));

    events::emit_unpause(env);

    if let Some(s) = start {
        let dur = env.ledger().sequence().saturating_sub(s);
        env.events().publish(
            (soroban_sdk::symbol_short!("resume"),),
            dur,
        );
    }

    Ok(())
}

pub fn batch_blacklist(
    env: &Env,
    admin: &Address,
    addresses: Vec<Address>,
    blacklisted: bool,
) -> Result<u32, BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;

    let count = addresses.len();
    if count > 50 {
        return Err(BrazaError::InvalidAmount);
    }

    let mut processed = 0u32;

    for addr in addresses.iter() {
        validation::require_country_allowed(env, &addr)?;
        storage::set_blacklisted(env, &addr, blacklisted);
        events::emit_blacklist(env, &addr, blacklisted);
        processed += 1;
    }

    env.events().publish(
        (soroban_sdk::symbol_short!("batch_bl"),),
        (processed, blacklisted),
    );

    Ok(processed)
}

pub fn force_burn(
    env: &Env,
    admin: &Address,
    from: &Address,
    amount: i128,
    reason: String,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;
    validation::require_positive_amount(amount)?;
    validation::require_sufficient_balance(env, from, amount)?;

    // COMPLIANCE obrigatório
    validation::require_not_blacklisted(env, from)?;
    validation::require_country_allowed(env, from)?;
    validation::require_kyc_level(env, from, 0)?;
    validation::require_acceptable_risk(env, from, amount.try_into().unwrap())?;
    

    let bal = storage::get_balance(env, from);
    let new_bal = bal
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;

    let supply = storage::get_total_supply(env);
    let new_supply = supply
        .checked_sub(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_balance(env, from, new_bal);
    storage::set_total_supply(env, new_supply);

    events::emit_burn(env, from, amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("force_brn"), from),
        (amount, reason),
    );

    Ok(())
}

pub fn force_transfer(
    env: &Env,
    admin: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
    reason: String,
) -> Result<(), BrazaError> {

    admin.require_auth();
    storage::bump_critical_storage(env);

    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;
    validation::require_positive_amount(amount)?;
    validation::require_sufficient_balance(env, from, amount)?;

    // COMPLIANCE COMPLETO nos dois lados
    validation::require_not_blacklisted(env, from)?;
    validation::require_not_blacklisted(env, to)?;
    validation::require_country_allowed(env, from)?;
    validation::require_country_allowed(env, to)?;
    validation::require_kyc_level(env, from, 0)?;
    validation::require_kyc_level(env, to, 0)?;
    validation::require_acceptable_risk(env, from, amount.try_into().unwrap())?;

    let bal_from = storage::get_balance(env, from);
    let bal_to = storage::get_balance(env, to);

    let new_from = bal_from
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;

    let new_to = bal_to
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_balance(env, from, new_from);
    storage::set_balance(env, to, new_to);

    events::emit_transfer(env, from, to, amount);

    env.events().publish(
        (soroban_sdk::symbol_short!("force_txf"), from, to),
        (amount, reason),
    );

    Ok(())
}

pub fn get_admin_info(env: &Env) -> (Address, bool, u32) {

    storage::bump_critical_storage(env);

    let admin = storage::get_admin(env);
    let paused = storage::is_paused(env);
    let supply = storage::get_total_supply(env);

    let supply_u32 = if supply <= 0 {
        0
    } else if supply > u32::MAX as i128 {
        u32::MAX
    } else {
        supply as u32
    };

    (admin, paused, supply_u32)
}

pub fn get_admin_info_full(env: &Env) -> (Address, bool, i128) {
    storage::bump_critical_storage(env);
    (storage::get_admin(env), storage::is_paused(env), storage::get_total_supply(env))
}

pub fn get_pause_reason(env: &Env) -> Option<String> {
    storage::bump_critical_storage(env);
    env.storage().instance().get(&soroban_sdk::symbol_short!("pause_rsn"))
}

pub fn get_contract_stats(env: &Env) -> (i128, i128, i128, u32) {
    storage::bump_critical_storage(env);

    let supply = storage::get_total_supply(env);
    let max = storage::MAX_SUPPLY;

    (supply, max, max.saturating_sub(supply), env.ledger().sequence())
}
