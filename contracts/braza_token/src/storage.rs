use crate::types::{BrazaError, TokenMetadata, VestingSchedule};
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

// ---------------------------
// CONSTANTES
// ---------------------------
pub const MAX_SUPPLY: i128 = 2_100_000_000_000_000; // 210 milhões BRZ
pub const INITIAL_SUPPLY: i128 = 1_000_000_000_000_000; // 100 milhões BRZ
pub const MAX_VESTING_SCHEDULES: u32 = 50;
pub const MAX_GLOBAL_VESTING_SCHEDULES: u32 = 10_000;
pub const VESTING_STORAGE_FEE: i128 = 1_000_000;
pub const MIN_VESTING_AMOUNT: i128 = 10_000_000;
pub const VESTING_CREATION_COOLDOWN_LEDGERS: u32 = 1_440;
pub const CRITICAL_STORAGE_TTL: u32 = 6_307_200;
pub const CRITICAL_STORAGE_THRESHOLD: u32 = 518_400;
pub const LEDGER_THRESHOLD_SHARED: u32 = 518_400;
pub const LEDGER_BUMP_SHARED: u32 = 6_307_200;

// ---------------------------
// TTL FUNCTIONS
// ---------------------------
pub fn bump_critical_storage(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(CRITICAL_STORAGE_THRESHOLD, CRITICAL_STORAGE_TTL);
}

const ADMIN: Symbol = symbol_short!("admin");
const PAUSED: Symbol = symbol_short!("paused");
const SUPPLY: Symbol = symbol_short!("supply");
const METADATA: Symbol = symbol_short!("metadat");
const BALANCE: Symbol = symbol_short!("balance");
const BLACKLIST: Symbol = symbol_short!("blklst");
const VEST_CNT: Symbol = symbol_short!("vst_cnt");
const VESTING: Symbol = symbol_short!("vesting");
const REENT_LOCK: Symbol = symbol_short!("reentlk");
const LAST_MINT_TIME: Symbol = symbol_short!("lastmnt");
const LAST_BURN_TIME: Symbol = symbol_short!("lastbrn");
const GLOBAL_VEST_COUNT: Symbol = symbol_short!("g_vstct");
const STORAGE_FEE_POOL: Symbol = symbol_short!("strpfee");
const LAST_VEST_TIME: Symbol = symbol_short!("lstvest");
const ALLOWANCE: Symbol = symbol_short!("allow");
const LOCKED_BALANCE: Symbol = symbol_short!("locked");

// ---------------------------
// BALANCE TTL
// ---------------------------
pub fn bump_balance(env: &Env, addr: &Address) {
    let key = (BALANCE, addr);
    env.storage()
        .persistent()
        .extend_ttl(&key, CRITICAL_STORAGE_THRESHOLD, CRITICAL_STORAGE_TTL);
}

// METADATA
pub fn get_metadata(env: &Env) -> TokenMetadata {
    env.storage().instance().get(&METADATA).unwrap()
}
pub fn set_metadata(env: &Env, meta: &TokenMetadata) {
    env.storage().instance().set(&METADATA, meta);
}

// BLACKLIST
pub fn is_blacklisted(env: &Env, addr: &Address) -> bool {
    let key = (BLACKLIST, addr);
    env.storage().persistent().get(&key).unwrap_or(false)
}
pub fn set_blacklisted(env: &Env, addr: &Address, val: bool) {
    let key = (BLACKLIST, addr);
    env.storage().persistent().set(&key, &val);
}

// VESTING COUNT
pub fn get_vesting_count(env: &Env, beneficiary: &Address) -> u32 {
    let key = (VEST_CNT, beneficiary);
    env.storage().persistent().get(&key).unwrap_or(0)
}

// GET SINGLE VESTING
pub fn get_vesting_schedule(env: &Env, beneficiary: &Address, id: u32) -> Option<VestingSchedule> {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().get(&key)
}

// SET VESTING
pub fn set_vesting_schedule(env: &Env, beneficiary: &Address, id: u32, schedule: &VestingSchedule) {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().set(&key, schedule);
}

// REMOVE VESTING
pub fn remove_vesting_schedule(env: &Env, beneficiary: &Address, id: u32) {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().remove(&key);
}

// GET ALL VESTINGS
pub fn get_all_vesting_schedules(env: &Env, beneficiary: &Address) -> Vec<VestingSchedule> {
    let count = get_vesting_count(env, beneficiary);
    let mut v = Vec::new(env);

    for id in 0..count {
        if let Some(s) = get_vesting_schedule(env, beneficiary, id) {
            v.push_back(s);
        }
    }
    v
}

// GLOBAL COUNT
pub fn get_global_vesting_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&GLOBAL_VEST_COUNT)
        .unwrap_or(0)
}

fn increment_global_vesting_count(env: &Env) {
    let c = get_global_vesting_count(env);
    env.storage().instance().set(&GLOBAL_VEST_COUNT, &(c + 1));
}

#[allow(dead_code)]
fn decrement_global_vesting_count(env: &Env) {
    let c = get_global_vesting_count(env);
    if c > 0 {
        env.storage().instance().set(&GLOBAL_VEST_COUNT, &(c - 1));
    }
}

// STORAGE FEE POOL
pub fn get_storage_fee_pool(env: &Env) -> i128 {
    env.storage().instance().get(&STORAGE_FEE_POOL).unwrap_or(0)
}

pub fn add_to_storage_fee_pool(env: &Env, amount: i128) {
    let cur = get_storage_fee_pool(env);
    let new = cur.checked_add(amount).unwrap_or(cur);
    env.storage().instance().set(&STORAGE_FEE_POOL, &new);
}

pub fn withdraw_from_storage_fee_pool(env: &Env, amount: i128) -> Result<(), BrazaError> {
    let cur = get_storage_fee_pool(env);
    if amount > cur {
        return Err(BrazaError::InsufficientBalance);
    }
    env.storage()
        .instance()
        .set(&STORAGE_FEE_POOL, &(cur - amount));
    Ok(())
}

// ALLOWANCE — GET
pub fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    let key = (ALLOWANCE, from, spender);
    env.storage().persistent().get(&key).unwrap_or(0)
}

// ALLOWANCE — EXISTS
pub fn has_allowance(env: &Env, from: &Address, spender: &Address) -> bool {
    let key = (ALLOWANCE, from, spender);
    env.storage().persistent().has(&key)
}

// SET ALLOWANCE
pub fn set_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
    let key = (ALLOWANCE, from, spender);
    if amount == 0 {
        env.storage().persistent().remove(&key);
    } else {
        env.storage().persistent().set(&key, &amount);
        env.storage().persistent().extend_ttl(
            &key,
            CRITICAL_STORAGE_THRESHOLD,
            CRITICAL_STORAGE_TTL,
        );
    }
}

// BUMP & REMOVE
pub fn bump_allowance(env: &Env, from: &Address, spender: &Address) {
    let key = (ALLOWANCE, from, spender);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            CRITICAL_STORAGE_THRESHOLD,
            CRITICAL_STORAGE_TTL,
        );
    }
}

// LOCKED BALANCE
pub fn get_locked_balance(env: &Env) -> i128 {
    env.storage().instance().get(&LOCKED_BALANCE).unwrap_or(0)
}

pub fn set_locked_balance(env: &Env, amt: i128) {
    env.storage().instance().set(&LOCKED_BALANCE, &amt);
}

pub fn increment_locked_balance(env: &Env, amt: i128) -> Result<i128, BrazaError> {
    let cur = get_locked_balance(env);
    let new = cur.checked_add(amt).ok_or(BrazaError::InvalidAmount)?;
    set_locked_balance(env, new);
    Ok(new)
}

pub fn decrement_locked_balance(env: &Env, amt: i128) -> Result<i128, BrazaError> {
    let cur = get_locked_balance(env);
    if cur < amt {
        return Err(BrazaError::InsufficientBalance);
    }
    set_locked_balance(env, cur - amt);
    Ok(cur - amt)
}

// CIRCULATING SUPPLY
pub fn get_circulating_supply(env: &Env) -> i128 {
    let total = get_total_supply(env);
    let locked = get_locked_balance(env);
    total.saturating_sub(locked)
}

// VALIDATE BURN
pub fn validate_burn_not_locked(env: &Env, burn: i128) -> Result<(), BrazaError> {
    let circ = get_circulating_supply(env);
    if burn > circ {
        return Err(BrazaError::InsufficientBalance);
    }
    Ok(())
}

// GLOBAL STATS
pub fn get_vesting_stats(env: &Env) -> (u32, u32, i128, u32) {
    (
        get_global_vesting_count(env),
        MAX_GLOBAL_VESTING_SCHEDULES,
        get_storage_fee_pool(env),
        MAX_VESTING_SCHEDULES,
    )
}

pub fn is_near_global_vesting_limit(env: &Env) -> bool {
    let cur = get_global_vesting_count(env);
    let thr = (MAX_GLOBAL_VESTING_SCHEDULES * 90) / 100;
    cur >= thr
}

// REENTRANCY GUARD
pub fn is_reentrancy_locked(env: &Env) -> bool {
    env.storage().instance().get(&REENT_LOCK).unwrap_or(false)
}

pub fn set_reentrancy_guard(env: &Env, locked: bool) {
    env.storage().instance().set(&REENT_LOCK, &locked);
}

pub fn assert_no_reentrancy(env: &Env) -> Result<(), BrazaError> {
    if is_reentrancy_locked(env) {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

// MINT TIME
pub fn get_last_mint_time(env: &Env) -> Option<u32> {
    env.storage().instance().get(&LAST_MINT_TIME)
}
pub fn set_last_mint_time(env: &Env, ledger: u32) {
    env.storage().instance().set(&LAST_MINT_TIME, &ledger);
}

// BURN TIME
pub fn get_last_burn_time(env: &Env) -> Option<u32> {
    env.storage().instance().get(&LAST_BURN_TIME)
}
pub fn set_last_burn_time(env: &Env, ledger: u32) {
    env.storage().instance().set(&LAST_BURN_TIME, &ledger);
}

// LAST VEST CREATION
pub fn get_last_vesting_creation_time(env: &Env, user: &Address) -> Option<u32> {
    let key = (LAST_VEST_TIME, user);
    env.storage().persistent().get(&key)
}

pub fn set_last_vesting_creation_time(env: &Env, user: &Address, seq: u32) {
    let key = (LAST_VEST_TIME, user);
    env.storage().persistent().set(&key, &seq);
}

// VESTING COOLDOWN
pub fn is_vesting_cooldown_expired(env: &Env, user: &Address) -> bool {
    match get_last_vesting_creation_time(env, user) {
        None => true,
        Some(last) => {
            let now = env.ledger().sequence();
            now.saturating_sub(last) >= VESTING_CREATION_COOLDOWN_LEDGERS
        }
    }
}

pub fn get_vesting_cooldown_remaining(env: &Env, user: &Address) -> Option<u32> {
    let last = get_last_vesting_creation_time(env, user)?;
    let now = env.ledger().sequence();
    let el = now.saturating_sub(last);
    if el < VESTING_CREATION_COOLDOWN_LEDGERS {
        Some(VESTING_CREATION_COOLDOWN_LEDGERS - el)
    } else {
        None
    }
}

// EVENT HELPERS
pub fn emit_balance_event(env: &Env, user: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("bal_evt"), user), amount);
}

pub fn emit_admin_event(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("adm_set"),), admin.clone());
}

// ADMIN HELPERS
pub fn ensure_admin(env: &Env, caller: &Address) -> Result<(), BrazaError> {
    let admin = get_admin(env);
    if &admin != caller {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

// PAUSE CHECK
pub fn assert_not_paused(env: &Env) -> Result<(), BrazaError> {
    if is_paused(env) {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

// ADD BALANCE
pub fn add_balance(env: &Env, user: &Address, amt: i128) {
    let cur = get_balance(env, user);
    let new = cur.saturating_add(amt);
    set_balance(env, user, new);
}

// SUB BALANCE
pub fn sub_balance(env: &Env, user: &Address, amt: i128) -> Result<(), BrazaError> {
    let cur = get_balance(env, user);
    if cur < amt {
        return Err(BrazaError::InsufficientBalance);
    }
    set_balance(env, user, cur - amt);
    Ok(())
}

// RESET GLOBAL (somente admin deve usar)
#[allow(dead_code)]
pub fn reset_global_state(env: &Env) {
    env.storage().instance().set(&GLOBAL_VEST_COUNT, &0u32);
    env.storage().instance().set(&STORAGE_FEE_POOL, &0i128);
    env.storage().instance().set(&LOCKED_BALANCE, &0i128);
    env.storage().instance().remove(&LAST_MINT_TIME);
    env.storage().instance().remove(&LAST_BURN_TIME);
}

// CLEANUP PERSISTENT KEYS
#[allow(dead_code)]
pub fn cleanup_balance(env: &Env, addr: &Address) {
    let key = (BALANCE, addr);
    env.storage().persistent().remove(&key);
}

#[allow(dead_code)]
pub fn cleanup_vesting_user(env: &Env, addr: &Address) {
    let cnt = get_vesting_count(env, addr);
    for id in 0..cnt {
        let key = (VESTING, addr, id);
        env.storage().persistent().remove(&key);
    }
    let cnt_key = (VEST_CNT, addr);
    env.storage().persistent().remove(&cnt_key);
}

// SAFE GET BOOL
#[allow(dead_code)]
pub fn get_bool(env: &Env, key: Symbol) -> bool {
    env.storage().instance().get(&key).unwrap_or(false)
}

// SAFE SET
#[allow(dead_code)]
pub fn set_bool(env: &Env, key: Symbol, val: bool) {
    env.storage().instance().set(&key, &val);
}

// LEDGER SEQ GETTER
#[allow(dead_code)]
pub fn ledger_seq(env: &Env) -> u32 {
    env.ledger().sequence()
}

// ADMIN
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&ADMIN).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

// PAUSE
pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED).unwrap_or(false)
}

pub fn set_paused(env: &Env, val: bool) {
    env.storage().instance().set(&PAUSED, &val);
}

// SUPPLY
pub fn get_total_supply(env: &Env) -> i128 {
    env.storage().instance().get(&SUPPLY).unwrap_or(0)
}

pub fn set_total_supply(env: &Env, amt: i128) {
    env.storage().instance().set(&SUPPLY, &amt);
}

// BALANCE
pub fn get_balance(env: &Env, user: &Address) -> i128 {
    let key = (BALANCE, user);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_balance(env: &Env, user: &Address, amt: i128) {
    let key = (BALANCE, user);
    env.storage().persistent().set(&key, &amt);
}

// REMOVE ALLOWANCE
pub fn remove_allowance(env: &Env, from: &Address, spender: &Address) {
    let key = (ALLOWANCE, from, spender);
    env.storage().persistent().remove(&key);
}

// increment_vesting_count
pub fn increment_vesting_count(env: &Env, user: &Address) -> Result<u32, BrazaError> {
    let cur = get_vesting_count(env, user);
    if cur >= MAX_VESTING_SCHEDULES {
        return Err(BrazaError::MaxVestingSchedulesExceeded);
    }

    let global = get_global_vesting_count(env);
    if global >= MAX_GLOBAL_VESTING_SCHEDULES {
        return Err(BrazaError::GlobalVestingLimitExceeded);
    }

    let new = cur + 1;
    env.storage().persistent().set(&(VEST_CNT, user), &new);
    increment_global_vesting_count(env);
    Ok(new)
}

// ============================================================================
// FUNÇÕES DE TESTE (EXPOSTAS APENAS PARA TESTES)
// ============================================================================

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn set_admin_test(env: &Env, admin: &Address) {
    set_admin(env, admin);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn is_blacklisted_test(env: &Env, addr: &Address) -> bool {
    is_blacklisted(env, addr)
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn set_blacklisted_test(env: &Env, addr: &Address, val: bool) {
    set_blacklisted(env, addr, val);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn set_balance_test(env: &Env, user: &Address, amt: i128) {
    set_balance(env, user, amt);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn get_balance_test(env: &Env, user: &Address) -> i128 {
    get_balance(env, user)
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn set_total_supply_test(env: &Env, amt: i128) {
    set_total_supply(env, amt);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn set_locked_balance_test(env: &Env, amt: i128) {
    set_locked_balance(env, amt);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn set_paused_test(env: &Env, val: bool) {
    set_paused(env, val);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn create_test_vesting(
    env: &Env,
    beneficiary: &Address,
    amount: i128,
    start: u32,
    cliff: u32,
    duration: u32,
) -> Result<u32, BrazaError> {
    let id = increment_vesting_count(env, beneficiary)?;

    let schedule = VestingSchedule {
        beneficiary: beneficiary.clone(),
        total_amount: amount,
        released_amount: 0,
        start_ledger: start,
        cliff_ledgers: cliff,
        duration_ledgers: duration,
        revocable: false,
        revoked: false,
    };

    set_vesting_schedule(env, beneficiary, id - 1, &schedule);
    Ok(id - 1)
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn clear_all_vestings(env: &Env, beneficiary: &Address) {
    let count = get_vesting_count(env, beneficiary);
    for id in 0..count {
        remove_vesting_schedule(env, beneficiary, id);
    }

    let key = (VEST_CNT, beneficiary);
    env.storage().persistent().set(&key, &0u32);
}

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub fn reset_contract_state(env: &Env) {
    reset_global_state(env);
}

// ============================================================================
// TESTES UNITÁRIOS INTERNOS
// ============================================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod circulating_tests {
    use super::*;

    #[test]
    fn test_circulating_simple() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_total_supply(&env, 1000);
            set_locked_balance(&env, 300);
            assert_eq!(get_circulating_supply(&env), 700);
        });
    }

    #[test]
    fn test_burn_guard() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_total_supply(&env, 1000);
            set_locked_balance(&env, 900);
            assert!(validate_burn_not_locked(&env, 200).is_err());
        });
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod locked_extra_tests {
    use super::*;

    #[test]
    fn test_locked_increment_overflow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_locked_balance(&env, i128::MAX - 5);
            let r = increment_locked_balance(&env, 100);
            assert!(r.is_err());
        });
    }

    #[test]
    fn test_locked_decrement_full() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_locked_balance(&env, 200);
            decrement_locked_balance(&env, 200).unwrap();
            assert_eq!(get_locked_balance(&env), 0);
        });
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod reentrancy_tests {
    use super::*;

    #[test]
    fn test_reentrancy_default_unlocked() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            assert!(!is_reentrancy_locked(&env));
        });
    }

    #[test]
    fn test_reentrancy_set_and_get() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_reentrancy_guard(&env, true);
            assert!(is_reentrancy_locked(&env));
        });
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod timelock_tests {
    use super::*;

    #[test]
    fn test_set_and_get_last_mint() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_last_mint_time(&env, 100);
            assert_eq!(get_last_mint_time(&env), Some(100));
        });
    }

    #[test]
    fn test_set_and_get_last_burn() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            set_last_burn_time(&env, 55);
            assert_eq!(get_last_burn_time(&env), Some(55));
        });
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod allowance_tests_extra {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_allowance_cycle() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::BrazaToken);
        env.as_contract(&contract_id, || {
            let a = Address::generate(&env);
            let b = Address::generate(&env);

            set_allowance(&env, &a, &b, 500);
            bump_allowance(&env, &a, &b);
            remove_allowance(&env, &a, &b);

            assert_eq!(get_allowance(&env, &a, &b), 0);
        });
    }
}
