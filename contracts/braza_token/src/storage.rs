use soroban_sdk::{Address, Env, Vec, symbol_short};
use crate::types::{TokenMetadata, VestingSchedule, BrazaError};

// ============================================================================
// CONSTANTES
// ============================================================================

/// Supply máximo: 21 milhões BRZ com 7 decimais
pub const MAX_SUPPLY: i128 = 210_000_000_000_000; // 21M × 10^7

/// Supply inicial liberado: 10 milhões BRZ com 7 decimais
pub const INITIAL_SUPPLY: i128 = 100_000_000_000_000; // 10M × 10^7

/// Limite máximo de vesting schedules por beneficiário
pub const MAX_VESTING_SCHEDULES: u32 = 10;

/// TTL para storage crítico (1 ano em ledgers ~= 6.3M ledgers)
const CRITICAL_STORAGE_TTL: u32 = 6_307_200;

/// TTL threshold para bump (30 dias ~= 518K ledgers)
const CRITICAL_STORAGE_THRESHOLD: u32 = 518_400;

// ============================================================================
// FUNÇÕES DE BUMP (TTL)
// ============================================================================

/// Faz bump do TTL de storage crítico (admin, paused, supply, metadata)
pub fn bump_critical_storage(env: &Env) {
    env.storage().instance().extend_ttl(
        CRITICAL_STORAGE_THRESHOLD,
        CRITICAL_STORAGE_TTL,
    );
}

/// Faz bump do TTL de balance de um endereço
pub fn bump_balance(env: &Env, addr: &Address) {
    let key = (symbol_short!("balance"), addr);
    env.storage().persistent().extend_ttl(
        &key,
        CRITICAL_STORAGE_THRESHOLD,
        CRITICAL_STORAGE_TTL,
    );
}

/// Faz bump do TTL de vesting schedules de um beneficiário
pub fn bump_vesting_schedules(env: &Env, addr: &Address, schedule_ids: &Vec<u32>) {
    for id in schedule_ids.iter() {
        let key = (symbol_short!("vesting"), addr, id);
        env.storage().persistent().extend_ttl(
            &key,
            CRITICAL_STORAGE_THRESHOLD,
            CRITICAL_STORAGE_TTL,
        );
    }
}

// ============================================================================
// ADMIN
// ============================================================================

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&symbol_short!("admin")).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&symbol_short!("admin"), admin);
}

// ============================================================================
// PAUSED
// ============================================================================

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&symbol_short!("paused"))
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&symbol_short!("paused"), &paused);
}

// ============================================================================
// TOTAL SUPPLY
// ============================================================================

pub fn get_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&symbol_short!("supply"))
        .unwrap_or(0)
}

pub fn set_total_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&symbol_short!("supply"), &amount);
}

// ============================================================================
// BALANCE
// ============================================================================

pub fn get_balance(env: &Env, addr: &Address) -> i128 {
    let key = (symbol_short!("balance"), addr);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_balance(env: &Env, addr: &Address, amount: i128) {
    let key = (symbol_short!("balance"), addr);
    env.storage().persistent().set(&key, &amount);
}

// ============================================================================
// METADATA
// ============================================================================

pub fn get_metadata(env: &Env) -> TokenMetadata {
    env.storage().instance().get(&symbol_short!("metadata")).unwrap()
}

pub fn set_metadata(env: &Env, metadata: &TokenMetadata) {
    env.storage().instance().set(&symbol_short!("metadata"), metadata);
}

// ============================================================================
// BLACKLIST
// ============================================================================

pub fn is_blacklisted(env: &Env, addr: &Address) -> bool {
    let key = (symbol_short!("blacklist"), addr);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(false)
}

pub fn set_blacklisted(env: &Env, addr: &Address, blacklisted: bool) {
    let key = (symbol_short!("blacklist"), addr);
    env.storage()
        .persistent()
        .set(&key, &blacklisted);
}

// ============================================================================
// VESTING
// ============================================================================

pub fn get_vesting_count(env: &Env, beneficiary: &Address) -> u32 {
    let key = (symbol_short!("vest_cnt"), beneficiary);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(0)
}

pub fn increment_vesting_count(env: &Env, beneficiary: &Address) -> Result<u32, BrazaError> {
    let current_count = get_vesting_count(env, beneficiary);
    
    if current_count >= MAX_VESTING_SCHEDULES {
        return Err(BrazaError::MaxVestingSchedulesExceeded);
    }
    
    let new_count = current_count + 1;
    let key = (symbol_short!("vest_cnt"), beneficiary);
    env.storage()
        .persistent()
        .set(&key, &new_count);
    
    Ok(new_count)
}

pub fn get_vesting_schedule(env: &Env, beneficiary: &Address, id: u32) -> Option<VestingSchedule> {
    let key = (symbol_short!("vesting"), beneficiary, id);
    env.storage().persistent().get(&key)
}

pub fn set_vesting_schedule(env: &Env, beneficiary: &Address, id: u32, schedule: &VestingSchedule) {
    let key = (symbol_short!("vesting"), beneficiary, id);
    env.storage().persistent().set(&key, schedule);
}

pub fn get_all_vesting_schedules(env: &Env, beneficiary: &Address) -> Vec<VestingSchedule> {
    let count = get_vesting_count(env, beneficiary);
    let mut schedules = Vec::new(env);
    
    for id in 0..count {
        if let Some(schedule) = get_vesting_schedule(env, beneficiary, id) {
            schedules.push_back(schedule);
        }
    }
    
    schedules
}
