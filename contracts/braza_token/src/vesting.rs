use soroban_sdk::{Address, Env, symbol_short};
use crate::storage;
use crate::types::{BrazaError, VestingSchedule};

// ============================================================================
// VESTING — CORRIGIDO, COMPATÍVEL COM O CONTRATO FINAL
// ============================================================================

/// Calcula quanto já deveria estar liberado até o ledger atual.
/// Usa cálculo proporcional linear após o cliff.
pub fn calculate_vested_amount(env: &Env, schedule: &VestingSchedule) -> i128 {

    // Vesting revogado → não libera mais do que já foi liberado
    if schedule.revoked {
        return schedule.released_amount;
    }

    let now = env.ledger().sequence();

    let elapsed = now.saturating_sub(schedule.start_ledger);

    // Antes do cliff → 0 tokens
    if elapsed < schedule.cliff_ledgers {
        return 0;
    }

    // Após duration → 100% liberado
    if elapsed >= schedule.duration_ledgers {
        return schedule.total_amount;
    }

    // Cálculo linear proporcional
    // vested = (total_amount * elapsed) / duration
    let num = schedule
        .total_amount
        .checked_mul(elapsed as i128)
        .unwrap_or(0);

    let vested = num
        .checked_div(schedule.duration_ledgers as i128)
        .unwrap_or(0);

    vested
}

/// Calcula quanto está disponível para release agora
pub fn calculate_releasable_amount(env: &Env, schedule: &VestingSchedule) -> i128 {
    let vested = calculate_vested_amount(env, schedule);
    vested.saturating_sub(schedule.released_amount).max(0)
}

/// Cria um vesting schedule corretamente alinhado ao contrato.
/// Token.rs já faz todas as validações e locked_balance.
pub fn create_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    total_amount: i128,
    cliff_ledgers: u32,
    duration_ledgers: u32,
    revocable: bool,
) -> Result<u32, BrazaError> {

    // CRÍTICO: usar incremento seguro que valida limites.
    let schedule_id = storage::increment_vesting_count(env, beneficiary)?;

    let start_ledger = env.ledger().sequence();

    let schedule = VestingSchedule {
        beneficiary: beneficiary.clone(),
        total_amount,
        released_amount: 0,
        start_ledger,
        cliff_ledgers,
        duration_ledgers,
        revocable,
        revoked: false,
    };

    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);

    // Evento padrão (token.rs emite o evento principal)
    env.events().publish(
        (symbol_short!("v_new"), beneficiary, schedule_id),
        total_amount,
    );

    Ok(schedule_id)
}

/// Libera tokens vestidos.
/// Token.rs se encarrega de transferir saldo e atualizar locked_balance.
pub fn release_vested_tokens(
    env: &Env,
    beneficiary: &Address,
    schedule_id: u32,
) -> Result<i128, BrazaError> {

    let mut schedule = storage
        ::get_vesting_schedule(env, beneficiary, schedule_id)
        .ok_or(BrazaError::VestingNotFound)?;

    if schedule.revoked {
        return Err(BrazaError::VestingNotFound);
    }

    let releasable = calculate_releasable_amount(env, &schedule);

    if releasable <= 0 {
        return Err(BrazaError::NoTokensToRelease);
    }

    // Atualizar schedule
    schedule.released_amount = schedule
        .released_amount
        .checked_add(releasable)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);

    Ok(releasable)
}

/// Revoga um vesting schedule (apenas se revocable).
pub fn revoke_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    schedule_id: u32,
) -> Result<i128, BrazaError> {

    let mut schedule = storage
        ::get_vesting_schedule(env, beneficiary, schedule_id)
        .ok_or(BrazaError::VestingNotFound)?;

    if !schedule.revocable {
        return Err(BrazaError::Unauthorized);
    }

    if schedule.revoked {
        return Err(BrazaError::VestingNotFound);
    }

    // Cálculo dos tokens não-vestidos
    let vested = calculate_vested_amount(env, &schedule);
    let unvested = schedule.total_amount.saturating_sub(vested);

    // Atualizar estado
    schedule.revoked = true;

    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);

    Ok(unvested)
}
