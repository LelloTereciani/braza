use soroban_sdk::{Address, Env};
use crate::storage;
use crate::types::{BrazaError, VestingSchedule};

// ============================================================================
// VESTING - CRITICAL-01 CORRIGIDO
// ============================================================================

/// Calcula a quantidade de tokens liberados para um vesting schedule
/// CORREÇÃO: Usa ledger.sequence() ao invés de timestamp
pub fn calculate_vested_amount(env: &Env, schedule: &VestingSchedule) -> i128 {
    // Se revogado, não libera mais tokens
    if schedule.revoked {
        return schedule.released_amount;
    }
    
    let current_ledger = env.ledger().sequence();
    let elapsed_ledgers = current_ledger.saturating_sub(schedule.start_ledger);
    
    // Se ainda não passou o cliff, nenhum token é liberado
    if elapsed_ledgers < schedule.cliff_ledgers {
        return 0;
    }
    
    // Se já passou a duração total, libera tudo
    if elapsed_ledgers >= schedule.duration_ledgers {
        return schedule.total_amount;
    }
    
    // Cálculo proporcional com aritmética segura
    // vested = (total_amount * elapsed_ledgers) / duration_ledgers
    let numerator = schedule.total_amount
        .checked_mul(elapsed_ledgers as i128)
        .unwrap_or(0);
    
    let vested = numerator
        .checked_div(schedule.duration_ledgers as i128)
        .unwrap_or(0);
    
    vested
}

/// Calcula a quantidade de tokens disponíveis para release
pub fn calculate_releasable_amount(env: &Env, schedule: &VestingSchedule) -> i128 {
    let vested = calculate_vested_amount(env, schedule);
    vested.saturating_sub(schedule.released_amount).max(0)
}

/// Cria um novo vesting schedule
pub fn create_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    total_amount: i128,
    cliff_ledgers: u32,
    duration_ledgers: u32,
    revocable: bool,
) -> Result<u32, BrazaError> {
    // CRITICAL-05: Incrementa e valida limite
    let schedule_id = storage::increment_vesting_count(env, beneficiary)?;
    
    let schedule = VestingSchedule {
        beneficiary: beneficiary.clone(),
        total_amount,
        released_amount: 0,
        start_ledger: env.ledger().sequence(),
        cliff_ledgers,
        duration_ledgers,
        revocable,
        revoked: false,
    };
    
    storage::set_vesting_schedule(env, beneficiary, schedule_id - 1, &schedule);
    
    Ok(schedule_id - 1)
}

/// Libera tokens de um vesting schedule
pub fn release_vested_tokens(
    env: &Env,
    beneficiary: &Address,
    schedule_id: u32,
) -> Result<i128, BrazaError> {
    let mut schedule = storage::get_vesting_schedule(env, beneficiary, schedule_id)
        .ok_or(BrazaError::VestingNotFound)?;
    
    let releasable = calculate_releasable_amount(env, &schedule);
    
    if releasable == 0 {
        return Err(BrazaError::NoTokensToRelease);
    }
    
    // Atualiza o schedule
    schedule.released_amount = schedule.released_amount
        .checked_add(releasable)
        .ok_or(BrazaError::InvalidAmount)?;
    
    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);
    
    Ok(releasable)
}

/// Revoga um vesting schedule (apenas se revocable)
pub fn revoke_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    schedule_id: u32,
) -> Result<i128, BrazaError> {
    let mut schedule = storage::get_vesting_schedule(env, beneficiary, schedule_id)
        .ok_or(BrazaError::VestingNotFound)?;
    
    if !schedule.revocable {
        return Err(BrazaError::Unauthorized);
    }
    
    if schedule.revoked {
        return Err(BrazaError::VestingNotFound);
    }
    
    // Calcula tokens não vestidos que serão devolvidos
    let vested = calculate_vested_amount(env, &schedule);
    let unvested = schedule.total_amount.saturating_sub(vested);
    
    schedule.revoked = true;
    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);
    
    Ok(unvested)
}
