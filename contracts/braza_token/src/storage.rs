use soroban_sdk::{Address, Env, Vec, symbol_short, Symbol};
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

/// ✅ NOVO: Limite global de vesting schedules ativos no contrato
pub const MAX_GLOBAL_VESTING_SCHEDULES: u32 = 10_000;

/// ✅ NOVO: Taxa de storage por vesting schedule (0.1 BRZ)
pub const VESTING_STORAGE_FEE: i128 = 1_000_000; // 0.1 BRZ com 7 decimais

/// ✅ NOVO: Taxa mínima de vesting (evita spam com valores baixos)
pub const MIN_VESTING_AMOUNT: i128 = 10_000_000; // 1 BRZ mínimo

/// ✅ NOVO: Cooldown entre criações de vesting (anti-spam)
pub const VESTING_CREATION_COOLDOWN_LEDGERS: u32 = 1440; // ~2 horas

/// TTL para storage crítico (1 ano em ledgers ~= 6.3M ledgers)
pub const CRITICAL_STORAGE_TTL: u32 = 6_307_200;

/// TTL threshold para bump (30 dias ~= 518K ledgers)
pub const CRITICAL_STORAGE_THRESHOLD: u32 = 518_400;

/// TTL para storage compartilhado (usado em timelock)
pub const LEDGER_THRESHOLD_SHARED: u32 = 518_400;
pub const LEDGER_BUMP_SHARED: u32 = 6_307_200;

// ============================================================================
// SÍMBOLOS DE STORAGE
// ============================================================================

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

/// ✅ NOVOS SÍMBOLOS
const GLOBAL_VEST_COUNT: Symbol = symbol_short!("g_vest_c");
const STORAGE_FEE_POOL: Symbol = symbol_short!("stor_pol");
const LAST_VEST_TIME: Symbol = symbol_short!("lst_vest");

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
    let key = (BALANCE, addr);
    env.storage().persistent().extend_ttl(
        &key,
        CRITICAL_STORAGE_THRESHOLD,
        CRITICAL_STORAGE_TTL,
    );
}

/// Faz bump do TTL de vesting schedules de um beneficiário
pub fn bump_vesting_schedules(env: &Env, addr: &Address, schedule_ids: &Vec<u32>) {
    for id in schedule_ids.iter() {
        let key = (VESTING, addr, id);
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
    env.storage().instance().get(&ADMIN).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

// ============================================================================
// PAUSED
// ============================================================================

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&PAUSED)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED, &paused);
}

// ============================================================================
// TOTAL SUPPLY
// ============================================================================

pub fn get_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&SUPPLY)
        .unwrap_or(0)
}

pub fn set_total_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&SUPPLY, &amount);
}

// ============================================================================
// BALANCE
// ============================================================================

pub fn get_balance(env: &Env, addr: &Address) -> i128 {
    let key = (BALANCE, addr);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_balance(env: &Env, addr: &Address, amount: i128) {
    let key = (BALANCE, addr);
    env.storage().persistent().set(&key, &amount);
}

// ============================================================================
// METADATA
// ============================================================================

pub fn get_metadata(env: &Env) -> TokenMetadata {
    env.storage().instance().get(&METADATA).unwrap()
}

pub fn set_metadata(env: &Env, metadata: &TokenMetadata) {
    env.storage().instance().set(&METADATA, metadata);
}

// ============================================================================
// BLACKLIST
// ============================================================================

pub fn is_blacklisted(env: &Env, addr: &Address) -> bool {
    let key = (BLACKLIST, addr);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(false)
}

pub fn set_blacklisted(env: &Env, addr: &Address, blacklisted: bool) {
    let key = (BLACKLIST, addr);
    env.storage()
        .persistent()
        .set(&key, &blacklisted);
}

// ============================================================================
// VESTING - FUNÇÕES PRINCIPAIS
// ============================================================================

pub fn get_vesting_count(env: &Env, beneficiary: &Address) -> u32 {
    let key = (VEST_CNT, beneficiary);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(0)
}

/// ✅ CORRIGIDO: Incrementa contador com verificações de limite global e cooldown
/// 
/// # Proteções contra Storage DoS:
/// - Limite por beneficiário (10 schedules)
/// - Limite global do contrato (10.000 schedules)
/// - Cooldown de 2h entre criações
/// - Taxa de storage obrigatória
/// 
/// # Retorna:
/// - Ok(new_count): Novo contador de schedules
/// - Err: Se exceder limites ou cooldown não expirado
pub fn increment_vesting_count(env: &Env, beneficiary: &Address) -> Result<u32, BrazaError> {
    // 1. ✅ PROTEÇÃO: Verificar limite por beneficiário
    let current_count = get_vesting_count(env, beneficiary);
    
    if current_count >= MAX_VESTING_SCHEDULES {
        // Emitir evento de tentativa bloqueada
        env.events().publish(
            (symbol_short!("vest_lmt"), beneficiary),
            (current_count, MAX_VESTING_SCHEDULES),
        );
        return Err(BrazaError::MaxVestingSchedulesExceeded);
    }
    
    // 2. ✅ PROTEÇÃO: Verificar limite global do contrato
    let global_count = get_global_vesting_count(env);
    
    if global_count >= MAX_GLOBAL_VESTING_SCHEDULES {
        // Emitir evento de limite global atingido
        env.events().publish(
            (symbol_short!("g_vst_lm"),),
            (global_count, MAX_GLOBAL_VESTING_SCHEDULES),
        );
        return Err(BrazaError::GlobalVestingLimitExceeded);
    }
    
    // 3. ✅ PROTEÇÃO: Verificar cooldown de criação (anti-spam)
    let last_creation_time = get_last_vesting_creation_time(env, beneficiary);
    let current_ledger = env.ledger().sequence();
    
    if let Some(last_time) = last_creation_time {
        let time_elapsed = current_ledger.saturating_sub(last_time);
        
        if time_elapsed < VESTING_CREATION_COOLDOWN_LEDGERS {
            let remaining_ledgers = VESTING_CREATION_COOLDOWN_LEDGERS.saturating_sub(time_elapsed);
            
            // Emitir evento de cooldown ativo
            env.events().publish(
                (symbol_short!("vest_cool"), beneficiary),
                remaining_ledgers,
            );
            
            return Err(BrazaError::VestingCooldownActive);
        }
    }
    
    // 4. ✅ ATUALIZAR CONTADORES
    let new_count = current_count + 1;
    let key = (VEST_CNT, beneficiary);
    env.storage()
        .persistent()
        .set(&key, &new_count);
    
    // Incrementar contador global
    increment_global_vesting_count(env);
    
    // Registrar timestamp desta criação
    set_last_vesting_creation_time(env, beneficiary, current_ledger);
    
    // 5. ✅ EMITIR EVENTO DE SUCESSO
    env.events().publish(
        (symbol_short!("vest_inc"), beneficiary),
        (new_count, global_count + 1),
    );
    
    Ok(new_count)
}

/// ✅ NOVO: Decrementa contador de vesting (quando schedule é completado/cancelado)
pub fn decrement_vesting_count(env: &Env, beneficiary: &Address) -> Result<(), BrazaError> {
    let current_count = get_vesting_count(env, beneficiary);
    
    if current_count == 0 {
        return Err(BrazaError::InvalidAmount);
    }
    
    let new_count = current_count - 1;
    let key = (VEST_CNT, beneficiary);
    env.storage()
        .persistent()
        .set(&key, &new_count);
    
    // Decrementar contador global
    decrement_global_vesting_count(env);
    
    Ok(())
}

pub fn get_vesting_schedule(env: &Env, beneficiary: &Address, id: u32) -> Option<VestingSchedule> {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().get(&key)
}

pub fn set_vesting_schedule(env: &Env, beneficiary: &Address, id: u32, schedule: &VestingSchedule) {
    let key = (VESTING, beneficiary, id);
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

/// ✅ NOVO: Remove vesting schedule (libera storage)
pub fn remove_vesting_schedule(env: &Env, beneficiary: &Address, id: u32) {
    let key = (VESTING, beneficiary, id);
    env.storage().persistent().remove(&key);
}

// ============================================================================
// VESTING - CONTADOR GLOBAL
// ============================================================================

/// ✅ NOVO: Obtém contador global de vesting schedules ativos
pub fn get_global_vesting_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&GLOBAL_VEST_COUNT)
        .unwrap_or(0)
}

/// ✅ NOVO: Incrementa contador global
fn increment_global_vesting_count(env: &Env) {
    let current = get_global_vesting_count(env);
    env.storage()
        .instance()
        .set(&GLOBAL_VEST_COUNT, &(current + 1));
}

/// ✅ NOVO: Decrementa contador global
fn decrement_global_vesting_count(env: &Env) {
    let current = get_global_vesting_count(env);
    if current > 0 {
        env.storage()
            .instance()
            .set(&GLOBAL_VEST_COUNT, &(current - 1));
    }
}

// ============================================================================
// VESTING - COOLDOWN E RATE LIMITING
// ============================================================================

/// ✅ NOVO: Obtém timestamp da última criação de vesting
pub fn get_last_vesting_creation_time(env: &Env, beneficiary: &Address) -> Option<u32> {
    let key = (LAST_VEST_TIME, beneficiary);
    env.storage().persistent().get(&key)
}

/// ✅ NOVO: Define timestamp da última criação de vesting
fn set_last_vesting_creation_time(env: &Env, beneficiary: &Address, ledger: u32) {
    let key = (LAST_VEST_TIME, beneficiary);
    env.storage().persistent().set(&key, &ledger);
}

/// ✅ NOVO: Verifica se cooldown expirou
pub fn is_vesting_cooldown_expired(env: &Env, beneficiary: &Address) -> bool {
    let last_time = get_last_vesting_creation_time(env, beneficiary);
    
    match last_time {
        None => true, // Nunca criou vesting
        Some(last) => {
            let current = env.ledger().sequence();
            let elapsed = current.saturating_sub(last);
            elapsed >= VESTING_CREATION_COOLDOWN_LEDGERS
        }
    }
}

/// ✅ NOVO: Retorna ledgers restantes até próxima criação permitida
pub fn get_vesting_cooldown_remaining(env: &Env, beneficiary: &Address) -> Option<u32> {
    let last_time = get_last_vesting_creation_time(env, beneficiary)?;
    let current = env.ledger().sequence();
    let elapsed = current.saturating_sub(last_time);
    
    if elapsed < VESTING_CREATION_COOLDOWN_LEDGERS {
        Some(VESTING_CREATION_COOLDOWN_LEDGERS.saturating_sub(elapsed))
    } else {
        None
    }
}

// ============================================================================
// STORAGE FEE POOL
// ============================================================================

/// ✅ NOVO: Obtém saldo do pool de taxas de storage
pub fn get_storage_fee_pool(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&STORAGE_FEE_POOL)
        .unwrap_or(0)
}

/// ✅ NOVO: Adiciona taxa ao pool de storage
pub fn add_to_storage_fee_pool(env: &Env, amount: i128) {
    let current = get_storage_fee_pool(env);
    let new_amount = current
        .checked_add(amount)
        .unwrap_or(current);
    
    env.storage()
        .instance()
        .set(&STORAGE_FEE_POOL, &new_amount);
    
    // Emitir evento
    env.events().publish(
        (symbol_short!("stor_add"),),
        (amount, new_amount),
    );
}

/// ✅ NOVO: Remove taxa do pool (apenas admin, para manutenção)
pub fn withdraw_from_storage_fee_pool(env: &Env, amount: i128) -> Result<(), BrazaError> {
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
    
    // Emitir evento
    env.events().publish(
        (symbol_short!("stor_wdr"),),
        (amount, new_amount),
    );
    
    Ok(())
}

// ============================================================================
// REENTRANCY GUARD
// ============================================================================

pub fn set_reentrancy_guard(env: &Env, locked: bool) {
    env.storage().instance().set(&REENT_LOCK, &locked);
}

pub fn is_reentrancy_locked(env: &Env) -> bool {
    env.storage().instance().get(&REENT_LOCK).unwrap_or(false)
}

// ============================================================================
// FUNÇÕES DE TIMELOCK (MINT/BURN)
// ============================================================================

/// Obtém o timestamp do último mint
pub fn get_last_mint_time(env: &Env) -> Option<u32> {
    env.storage().instance().get(&LAST_MINT_TIME)
}

/// Define o timestamp do último mint
pub fn set_last_mint_time(env: &Env, ledger: u32) {
    env.storage().instance().set(&LAST_MINT_TIME, &ledger);
    env.storage().instance().extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Obtém o timestamp do último burn
pub fn get_last_burn_time(env: &Env) -> Option<u32> {
    env.storage().instance().get(&LAST_BURN_TIME)
}

/// Define o timestamp do último burn
pub fn set_last_burn_time(env: &Env, ledger: u32) {
    env.storage().instance().set(&LAST_BURN_TIME, &ledger);
    env.storage().instance().extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

// ============================================================================
// FUNÇÕES DE ESTATÍSTICAS E MONITORAMENTO
// ============================================================================

/// ✅ NOVO: Retorna estatísticas completas de vesting
pub fn get_vesting_stats(env: &Env) -> (u32, u32, i128, u32) {
    let global_count = get_global_vesting_count(env);
    let max_global = MAX_GLOBAL_VESTING_SCHEDULES;
    let storage_pool = get_storage_fee_pool(env);
    let max_per_user = MAX_VESTING_SCHEDULES;
    
    (global_count, max_global, storage_pool, max_per_user)
}

/// ✅ NOVO: Verifica se contrato está próximo do limite global
pub fn is_near_global_vesting_limit(env: &Env) -> bool {
    let current = get_global_vesting_count(env);
    let threshold = (MAX_GLOBAL_VESTING_SCHEDULES * 90) / 100; // 90% do limite
    
    current >= threshold
}

/// ✅ NOVO: Retorna porcentagem de uso do limite global
pub fn get_global_vesting_usage_percentage(env: &Env) -> u32 {
    let current = get_global_vesting_count(env);
    ((current as u64 * 100) / MAX_GLOBAL_VESTING_SCHEDULES as u64) as u32
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
    fn setup_test_env() -> (Env, Address) {
        let env = Env::default();
        let beneficiary = Address::generate(&env);
        
        // Inicializar storage
        env.storage().instance().set(&GLOBAL_VEST_COUNT, &0u32);
        env.storage().instance().set(&STORAGE_FEE_POOL, &0i128);
        
        (env, beneficiary)
    }
    
    #[test]
    fn test_increment_vesting_count_success() {
        let (env, beneficiary) = setup_test_env();
        
        let result = increment_vesting_count(&env, &beneficiary);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(get_vesting_count(&env, &beneficiary), 1);
        assert_eq!(get_global_vesting_count(&env), 1);
    }
    
    #[test]
    fn test_increment_vesting_count_max_per_user() {
        let (env, beneficiary) = setup_test_env();
        
        // Criar 10 schedules (máximo)
        for i in 0..MAX_VESTING_SCHEDULES {
            let result = increment_vesting_count(&env, &beneficiary);
            assert!(result.is_ok());
            
            // Avançar ledger para passar cooldown
            env.ledger().with_mut(|li| {
                li.sequence_number += VESTING_CREATION_COOLDOWN_LEDGERS;
            });
        }
        
        // 11º deve falhar
        let result = increment_vesting_count(&env, &beneficiary);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::MaxVestingSchedulesExceeded);
    }
    
    #[test]
    fn test_vesting_cooldown_enforcement() {
        let (env, beneficiary) = setup_test_env();
        
        // Primeira criação
        increment_vesting_count(&env, &beneficiary).unwrap();
        
        // Tentativa imediata (deve falhar)
        let result = increment_vesting_count(&env, &beneficiary);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::VestingCooldownActive);
        
        // Verificar cooldown ativo
        assert!(!is_vesting_cooldown_expired(&env, &beneficiary));
        assert!(get_vesting_cooldown_remaining(&env, &beneficiary).is_some());
    }
    
    #[test]
    fn test_vesting_cooldown_expiration() {
        let (env, beneficiary) = setup_test_env();
        
        increment_vesting_count(&env, &beneficiary).unwrap();
        
        // Avançar tempo além do cooldown
        env.ledger().with_mut(|li| {
            li.sequence_number += VESTING_CREATION_COOLDOWN_LEDGERS;
        });
        
        // Deve permitir nova criação
        assert!(is_vesting_cooldown_expired(&env, &beneficiary));
        assert!(get_vesting_cooldown_remaining(&env, &beneficiary).is_none());
        
        let result = increment_vesting_count(&env, &beneficiary);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_global_vesting_limit() {
        let env = Env::default();
        
        // Simular limite global atingido
        env.storage().instance().set(&GLOBAL_VEST_COUNT, &MAX_GLOBAL_VESTING_SCHEDULES);
        
        let beneficiary = Address::generate(&env);
        
        let result = increment_vesting_count(&env, &beneficiary);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::GlobalVestingLimitExceeded);
    }
    
    #[test]
    fn test_storage_fee_pool() {
        let (env, _) = setup_test_env();
        
        // Adicionar taxa
        add_to_storage_fee_pool(&env, VESTING_STORAGE_FEE);
        assert_eq!(get_storage_fee_pool(&env), VESTING_STORAGE_FEE);
        
        // Adicionar mais
        add_to_storage_fee_pool(&env, VESTING_STORAGE_FEE);
        assert_eq!(get_storage_fee_pool(&env), VESTING_STORAGE_FEE * 2);
        
        // Retirar
        withdraw_from_storage_fee_pool(&env, VESTING_STORAGE_FEE).unwrap();
        assert_eq!(get_storage_fee_pool(&env), VESTING_STORAGE_FEE);
    }
    
    #[test]
    fn test_decrement_vesting_count() {
        let (env, beneficiary) = setup_test_env();
        
        increment_vesting_count(&env, &beneficiary).unwrap();
        assert_eq!(get_vesting_count(&env, &beneficiary), 1);
        assert_eq!(get_global_vesting_count(&env), 1);
        
        decrement_vesting_count(&env, &beneficiary).unwrap();
        assert_eq!(get_vesting_count(&env, &beneficiary), 0);
        assert_eq!(get_global_vesting_count(&env), 0);
    }
    
    #[test]
    fn test_vesting_stats() {
        let (env, beneficiary) = setup_test_env();
        
        increment_vesting_count(&env, &beneficiary).unwrap();
        add_to_storage_fee_pool(&env, VESTING_STORAGE_FEE);
        
        let (global, max_global, pool, max_user) = get_vesting_stats(&env);
        
        assert_eq!(global, 1);
        assert_eq!(max_global, MAX_GLOBAL_VESTING_SCHEDULES);
        assert_eq!(pool, VESTING_STORAGE_FEE);
        assert_eq!(max_user, MAX_VESTING_SCHEDULES);
    }
    
    #[test]
    fn test_global_usage_percentage() {
        let env = Env::default();
        
        // 50% de uso
        env.storage().instance().set(&GLOBAL_VEST_COUNT, &(MAX_GLOBAL_VESTING_SCHEDULES / 2));
        assert_eq!(get_global_vesting_usage_percentage(&env), 50);
        
        // 90% de uso (próximo do limite)
        env.storage().instance().set(&GLOBAL_VEST_COUNT, &((MAX_GLOBAL_VESTING_SCHEDULES * 90) / 100));
        assert!(is_near_global_vesting_limit(&env));
    }
}
