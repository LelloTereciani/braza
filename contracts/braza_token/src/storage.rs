use soroban_sdk::{Address, Env, Vec, symbol_short, Symbol};
use crate::types::{TokenMetadata, VestingSchedule, BrazaError};

//
// CONSTANTES
//

pub const MAX_SUPPLY: i128 = 210_000_000_000_000;
pub const INITIAL_SUPPLY: i128 = 100_000_000_000_000;
pub const MAX_VESTING_SCHEDULES: u32 = 10;
pub const MAX_GLOBAL_VESTING_SCHEDULES: u32 = 10_000;
pub const VESTING_STORAGE_FEE: i128 = 1_000_000;
pub const MIN_VESTING_AMOUNT: i128 = 10_000_000;
pub const VESTING_CREATION_COOLDOWN_LEDGERS: u32 = 1_440;

pub const CRITICAL_STORAGE_TTL: u32 = 6_307_200;
pub const CRITICAL_STORAGE_THRESHOLD: u32 = 518_400;

pub const LEDGER_THRESHOLD_SHARED: u32 = 518_400;
pub const LEDGER_BUMP_SHARED: u32 = 6_307_200;

//
// Símbolos de Storage
//

const ADMIN: Symbol = symbol_short!("admin");
const PAUSED: Symbol = symbol_short!("paused");
const SUPPLY: Symbol = symbol_short!("supply");
const METADATA: Symbol = symbol_short!("metadata");
const BALANCE: Symbol = symbol_short!("balance");
const BLACKLIST: Symbol = symbol_short!("blacklist");
const VEST_CNT: Symbol = symbol_short!("vest_cnt");
const VESTING: Symbol = symbol_short!("vesting");
const REENT_LOCK: Symbol = symbol_short!("reent_lock");
const LAST_MINT_TIME: Symbol = symbol_short!("last_mnt");
const LAST_BURN_TIME: Symbol = symbol_short!("last_brn");

const GLOBAL_VEST_COUNT: Symbol = symbol_short!("g_vest_c");
const STORAGE_FEE_POOL: Symbol = symbol_short!("stor_pol");
const LAST_VEST_TIME: Symbol = symbol_short!("lst_vest");
const ALLOWANCE: Symbol = symbol_short!("allow");
const LOCKED_BALANCE: Symbol = symbol_short!("locked");

//
// TTL FUNCTIONS
//

pub fn bump_critical_storage(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(CRITICAL_STORAGE_THRESHOLD, CRITICAL_STORAGE_TTL);
}

pub fn bump_balance(env: &Env, addr: &Address) {
    let key = (BALANCE, addr);
    env.storage()
        .persistent()
        .extend_ttl(&key, CRITICAL_STORAGE_THRESHOLD, CRITICAL_STORAGE_TTL);
}

pub fn bump_vesting_schedules(env: &Env, addr: &Address, ids: &Vec<u32>) {
    for id in ids.iter() {
        let key = (VESTING, addr, id);
        env.storage()
            .persistent()
            .extend_ttl(&key, CRITICAL_STORAGE_THRESHOLD, CRITICAL_STORAGE_TTL);
    }
}

//
// ADMIN
//

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&ADMIN).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

//
// PAUSE
//

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&PAUSED)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED, &paused);
}

//
// SUPPLY
//

pub fn get_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&SUPPLY)
        .unwrap_or(0)
}

pub fn set_total_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&SUPPLY, &amount);
}

//
// BALANCE
//

pub fn get_balance(env: &Env, addr: &Address) -> i128 {
    let key = (BALANCE, addr);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_balance(env: &Env, addr: &Address, amount: i128) {
    let key = (BALANCE, addr);
    env.storage().persistent().set(&key, &amount);
}

//
// METADATA
//

pub fn get_metadata(env: &Env) -> TokenMetadata {
    env.storage().instance().get(&METADATA).unwrap()
}

pub fn set_metadata(env: &Env, meta: &TokenMetadata) {
    env.storage().instance().set(&METADATA, meta);
}

//
// BLACKLIST
//

pub fn is_blacklisted(env: &Env, addr: &Address) -> bool {
    let key = (BLACKLIST, addr);
    env.storage().persistent().get(&key).unwrap_or(false)
}

pub fn set_blacklisted(env: &Env, addr: &Address, val: bool) {
    let key = (BLACKLIST, addr);
    env.storage().persistent().set(&key, &val);
}
//
// VESTING — Funções Principais
//

pub fn get_vesting_count(env: &Env, beneficiary: &Address) -> u32 {
    let key = (VEST_CNT, beneficiary);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn get_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    id: u32,
) -> Option<VestingSchedule> {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().get(&key)
}

pub fn set_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    id: u32,
    schedule: &VestingSchedule,
) {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().set(&key, schedule);
}

pub fn remove_vesting_schedule(env: &Env, beneficiary: &Address, id: u32) {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().remove(&key);
}

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

//
// Incremento com segurança
//

pub fn increment_vesting_count(env: &Env, beneficiary: &Address) -> Result<u32, BrazaError> {
    let current = get_vesting_count(env, beneficiary);

    if current >= MAX_VESTING_SCHEDULES {
        env.events().publish(
            (symbol_short!("vest_lmt"), beneficiary),
            (current, MAX_VESTING_SCHEDULES),
        );
        return Err(BrazaError::MaxVestingSchedulesExceeded);
    }

    // LIMITE GLOBAL
    let global = get_global_vesting_count(env);
    if global >= MAX_GLOBAL_VESTING_SCHEDULES {
        env.events().publish(
            (symbol_short!("g_vst_lm"),),
            (global, MAX_GLOBAL_VESTING_SCHEDULES),
        );
        return Err(BrazaError::GlobalVestingLimitExceeded);
    }

    // COOLDOWN
    let last = get_last_vesting_creation_time(env, beneficiary);
    let ledger = env.ledger().sequence();

    if let Some(last) = last {
        let elapsed = ledger.saturating_sub(last);

        if elapsed < VESTING_CREATION_COOLDOWN_LEDGERS {
            let remain = VESTING_CREATION_COOLDOWN_LEDGERS.saturating_sub(elapsed);

            env.events().publish(
                (symbol_short!("vest_cool"), beneficiary),
                remain,
            );

            return Err(BrazaError::VestingCooldownActive);
        }
    }

    let new_count = current + 1;
    let key = (VEST_CNT, beneficiary);
    env.storage().persistent().set(&key, &new_count);

    increment_global_vesting_count(env);
    set_last_vesting_creation_time(env, beneficiary, ledger);

    env.events().publish(
        (symbol_short!("vest_inc"), beneficiary),
        (new_count, global + 1),
    );

    Ok(new_count)
}

pub fn decrement_vesting_count(env: &Env, beneficiary: &Address) -> Result<(), BrazaError> {
    let current = get_vesting_count(env, beneficiary);

    if current == 0 {
        return Err(BrazaError::InvalidAmount); // não pode diminuir abaixo de 0
    }

    let new_count = current - 1;
    let key = (VEST_CNT, beneficiary);
    env.storage().persistent().set(&key, &new_count);

    decrement_global_vesting_count(env);

    Ok(())
}

//
// Contador Global
//

pub fn get_global_vesting_count(env: &Env) -> u32 {
    env.storage().instance().get(&GLOBAL_VEST_COUNT).unwrap_or(0)
}

fn increment_global_vesting_count(env: &Env) {
    let c = get_global_vesting_count(env);
    env.storage()
        .instance()
        .set(&GLOBAL_VEST_COUNT, &(c + 1));
}

fn decrement_global_vesting_count(env: &Env) {
    let c = get_global_vesting_count(env);
    if c > 0 {
        env.storage()
            .instance()
            .set(&GLOBAL_VEST_COUNT, &(c - 1));
    }
}

//
// Vesting Cooldown
//

pub fn get_last_vesting_creation_time(env: &Env, beneficiary: &Address) -> Option<u32> {
    let key = (LAST_VEST_TIME, beneficiary);
    env.storage().persistent().get(&key)
}

fn set_last_vesting_creation_time(
    env: &Env,
    beneficiary: &Address,
    ledger: u32,
) {
    let key = (LAST_VEST_TIME, beneficiary);
    env.storage().persistent().set(&key, &ledger);
}

pub fn is_vesting_cooldown_expired(env: &Env, beneficiary: &Address) -> bool {
    match get_last_vesting_creation_time(env, beneficiary) {
        None => true,
        Some(last) => {
            let now = env.ledger().sequence();
            now.saturating_sub(last) >= VESTING_CREATION_COOLDOWN_LEDGERS
        }
    }
}

pub fn get_vesting_cooldown_remaining(env: &Env, beneficiary: &Address) -> Option<u32> {
    let last = get_last_vesting_creation_time(env, beneficiary)?;
    let now = env.ledger().sequence();
    let elapsed = now.saturating_sub(last);

    if elapsed < VESTING_CREATION_COOLDOWN_LEDGERS {
        Some(VESTING_CREATION_COOLDOWN_LEDGERS.saturating_sub(elapsed))
    } else {
        None
    }
}
//
// STORAGE FEE POOL
//

pub fn get_storage_fee_pool(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&STORAGE_FEE_POOL)
        .unwrap_or(0)
}

pub fn add_to_storage_fee_pool(env: &Env, amount: i128) {
    let current = get_storage_fee_pool(env);
    let new_amount = current
        .checked_add(amount)
        .unwrap_or(current);

    env.storage()
        .instance()
        .set(&STORAGE_FEE_POOL, &new_amount);

    env.events().publish(
        (symbol_short!("stor_add"),),
        (amount, new_amount),
    );
}

pub fn withdraw_from_storage_fee_pool(
    env: &Env,
    amount: i128,
) -> Result<(), BrazaError> {
    let current = get_storage_fee_pool(env);

    if amount > current {
        return Err(BrazaError::InsufficientBalance);
    }

    let new_amount = current
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;

    env.storage()
        .instance()
        .set(&STORAGE_FEE_POOL, &new_amount);

    env.events().publish(
        (symbol_short!("stor_wdr"),),
        (amount, new_amount),
    );

    Ok(())
}

//
// ALLOWANCE
//

pub fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    let key = (ALLOWANCE, from, spender);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(0)
}

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

pub fn remove_allowance(env: &Env, from: &Address, spender: &Address) {
    let key = (ALLOWANCE, from, spender);
    env.storage().persistent().remove(&key);
}

pub fn has_allowance(env: &Env, from: &Address, spender: &Address) -> bool {
    let key = (ALLOWANCE, from, spender);
    env.storage().persistent().has(&key)
}

//
// LOCKED BALANCE
//

pub fn get_locked_balance(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&LOCKED_BALANCE)
        .unwrap_or(0)
}

pub fn set_locked_balance(env: &Env, amount: i128) {
    env.storage().instance().set(&LOCKED_BALANCE, &amount);
    bump_critical_storage(env);
}

pub fn increment_locked_balance(env: &Env, amount: i128) -> Result<i128, BrazaError> {
    let current = get_locked_balance(env);
    let new_locked = current
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    set_locked_balance(env, new_locked);

    env.events().publish(
        (symbol_short!("lock_inc"),),
        (amount, new_locked),
    );

    Ok(new_locked)
}

pub fn decrement_locked_balance(env: &Env, amount: i128) -> Result<i128, BrazaError> {
    let current = get_locked_balance(env);

    if current < amount {
        env.events().publish(
            (symbol_short!("lock_err"),),
            (amount, current),
        );
        return Err(BrazaError::InsufficientBalance);
    }

    let new_locked = current
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;

    set_locked_balance(env, new_locked);

    env.events().publish(
        (symbol_short!("lock_dec"),),
        (amount, new_locked),
    );

    Ok(new_locked)
}

//
// CIRCULATING SUPPLY
//

pub fn get_circulating_supply(env: &Env) -> i128 {
    let total = get_total_supply(env);
    let locked = get_locked_balance(env);
    total.saturating_sub(locked)
}

pub fn validate_burn_not_locked(
    env: &Env,
    burn_amount: i128,
) -> Result<(), BrazaError> {
    let total = get_total_supply(env);
    let locked = get_locked_balance(env);
    let circulating = total.saturating_sub(locked);

    if burn_amount > circulating {
        env.events().publish(
            (symbol_short!("brn_lock"),),
            (burn_amount, circulating, locked),
        );
        return Err(BrazaError::InsufficientBalance);
    }

    Ok(())
}

//
// ESTATÍSTICAS
//

pub fn get_vesting_stats(env: &Env) -> (u32, u32, i128, u32) {
    (
        get_global_vesting_count(env),
        MAX_GLOBAL_VESTING_SCHEDULES,
        get_storage_fee_pool(env),
        MAX_VESTING_SCHEDULES,
    )
}

pub fn is_near_global_vesting_limit(env: &Env) -> bool {
    let current = get_global_vesting_count(env);
    let threshold = (MAX_GLOBAL_VESTING_SCHEDULES * 90) / 100;
    current >= threshold
}

pub fn get_global_vesting_usage_percentage(env: &Env) -> u32 {
    let current = get_global_vesting_count(env);
    ((current as u64 * 100) / MAX_GLOBAL_VESTING_SCHEDULES as u64) as u32
}

//
// TESTES UNITÁRIOS — VESTING
//

#[cfg(test)]
mod vesting_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup_test_env() -> (Env, Address) {
        let env = Env::default();
        let user = Address::generate(&env);

        env.storage().instance().set(&GLOBAL_VEST_COUNT, &0u32);
        env.storage().instance().set(&STORAGE_FEE_POOL, &0i128);

        (env, user)
    }

    #[test]
    fn test_increment_vesting_count_success() {
        let (env, user) = setup_test_env();

        let r = increment_vesting_count(&env, &user);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 1);

        assert_eq!(get_vesting_count(&env, &user), 1);
        assert_eq!(get_global_vesting_count(&env), 1);
    }

    #[test]
    fn test_increment_vesting_count_max_per_user() {
        let (env, user) = setup_test_env();

        for _ in 0..MAX_VESTING_SCHEDULES {
            let ok = increment_vesting_count(&env, &user);
            assert!(ok.is_ok());

            env.ledger().with_mut(|l| {
                l.sequence_number += VESTING_CREATION_COOLDOWN_LEDGERS;
            });
        }

        let err = increment_vesting_count(&env, &user);
        assert!(err.is_err());
        assert_eq!(err.unwrap_err(), BrazaError::MaxVestingSchedulesExceeded);
    }

    #[test]
    fn test_vesting_cooldown_enforcement() {
        let (env, user) = setup_test_env();

        increment_vesting_count(&env, &user).unwrap();

        let second = increment_vesting_count(&env, &user);
        assert!(second.is_err());
        assert_eq!(second.unwrap_err(), BrazaError::VestingCooldownActive);

        assert!(!is_vesting_cooldown_expired(&env, &user));
        assert!(get_vesting_cooldown_remaining(&env, &user).is_some());
    }

    #[test]
    fn test_vesting_cooldown_expired() {
        let (env, user) = setup_test_env();

        increment_vesting_count(&env, &user).unwrap();

        env.ledger().with_mut(|l| {
            l.sequence_number += VESTING_CREATION_COOLDOWN_LEDGERS;
        });

        assert!(is_vesting_cooldown_expired(&env, &user));
        assert!(get_vesting_cooldown_remaining(&env, &user).is_none());

        assert!(increment_vesting_count(&env, &user).is_ok());
    }

    #[test]
    fn test_global_vesting_limit() {
        let env = Env::default();
        env.storage()
            .instance()
            .set(&GLOBAL_VEST_COUNT, &MAX_GLOBAL_VESTING_SCHEDULES);

        let user = Address::generate(&env);

        let res = increment_vesting_count(&env, &user);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), BrazaError::GlobalVestingLimitExceeded);
    }
}

//
// TESTES — STORAGE FEE POOL
//

#[cfg(test)]
mod storage_fee_tests {
    use super::*;

    fn setup() -> Env {
        let env = Env::default();
        env.storage().instance().set(&STORAGE_FEE_POOL, &0i128);
        env
    }

    #[test]
    fn test_storage_fee_add() {
        let env = setup();

        add_to_storage_fee_pool(&env, VESTING_STORAGE_FEE);
        assert_eq!(get_storage_fee_pool(&env), VESTING_STORAGE_FEE);

        add_to_storage_fee_pool(&env, VESTING_STORAGE_FEE);
        assert_eq!(get_storage_fee_pool(&env), VESTING_STORAGE_FEE * 2);
    }

    #[test]
    fn test_storage_fee_withdraw() {
        let env = setup();

        add_to_storage_fee_pool(&env, 5000);

        withdraw_from_storage_fee_pool(&env, 2000).unwrap();
        assert_eq!(get_storage_fee_pool(&env), 3000);
    }
}

//
// TESTES — ALLOWANCE
//

#[cfg(test)]
mod allowance_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup_test_env() -> (Env, Address, Address) {
        let env = Env::default();
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);
        (env, owner, spender)
    }

    #[test]
    fn test_allowance_default_zero() {
        let (env, owner, spender) = setup_test_env();
        assert_eq!(get_allowance(&env, &owner, &spender), 0);
        assert!(!has_allowance(&env, &owner, &spender));
    }

    #[test]
    fn test_allowance_set_and_get() {
        let (env, owner, spender) = setup_test_env();

        set_allowance(&env, &owner, &spender, 1000);
        assert_eq!(get_allowance(&env, &owner, &spender), 1000);
        assert!(has_allowance(&env, &owner, &spender));
    }

    #[test]
    fn test_allowance_zero_removes_entry() {
        let (env, owner, spender) = setup_test_env();

        set_allowance(&env, &owner, &spender, 1000);
        assert!(has_allowance(&env, &owner, &spender));

        set_allowance(&env, &owner, &spender, 0);
        assert_eq!(get_allowance(&env, &owner, &spender), 0);
        assert!(!has_allowance(&env, &owner, &spender));
    }

    #[test]
    fn test_bump_allowance_no_fail() {
        let (env, owner, spender) = setup_test_env();
        bump_allowance(&env, &owner, &spender);
        assert_eq!(get_allowance(&env, &owner, &spender), 0);
    }
}

//
// TESTES — LOCKED BALANCE (VESTING)
//

#[cfg(test)]
mod locked_balance_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup() -> Env {
        let env = Env::default();
        env.storage().instance().set(&LOCKED_BALANCE, &0i128);
        env.storage().instance().set(&SUPPLY, &100_000_000_000_000i128);
        env
    }

    #[test]
    fn test_locked_balance_default_zero() {
        let env = setup();
        assert_eq!(get_locked_balance(&env), 0);
    }

    #[test]
    fn test_increment_locked_balance() {
        let env = setup();
        increment_locked_balance(&env, 500).unwrap();
        assert_eq!(get_locked_balance(&env), 500);
    }

    #[test]
    fn test_decrement_locked_balance() {
        let env = setup();
        set_locked_balance(&env, 1000);
        decrement_locked_balance(&env, 250).unwrap();
        assert_eq!(get_locked_balance(&env), 750);
    }

    #[test]
    fn test_decrement_locked_balance_invalid() {
        let env = setup();
        set_locked_balance(&env, 100);

        let r = decrement_locked_balance(&env, 200);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), BrazaError::InsufficientBalance);

        assert_eq!(get_locked_balance(&env), 100);
    }

    #[test]
    fn test_locked_balance_overflow_protection() {
        let env = setup();

        set_locked_balance(&env, i128::MAX - 10);

        let r = increment_locked_balance(&env, 50);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), BrazaError::InvalidAmount);
    }

    #[test]
    fn test_circulating_supply() {
        let env = setup();

        assert_eq!(get_circulating_supply(&env), 100_000_000_000_000);

        set_locked_balance(&env, 10_000_000_000_000);

        assert_eq!(get_circulating_supply(&env), 90_000_000_000_000);
    }

    #[test]
    fn test_validate_burn_not_locked_success() {
        let env = setup();

        set_locked_balance(&env, 10_000_000_000_000);
        assert!(validate_burn_not_locked(&env, 1_000_000_000_000).is_ok());
    }

    #[test]
    fn test_validate_burn_not_locked_failure() {
        let env = setup();

        set_locked_balance(&env, 95_000_000_000_000);
        let r = validate_burn_not_locked(&env, 10_000_000_000_000);
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), BrazaError::InsufficientBalance);
    }
}
