use crate::storage;
use crate::types::{BrazaError, VestingSchedule};
use soroban_sdk::{symbol_short, Address, Env};

// ============================================================================
// VESTING — LÓGICA MATEMÁTICA E DE ESTADO
// ============================================================================

/// Calcula quanto já deveria estar liberado até o ledger atual.
/// Usa cálculo proporcional linear após o cliff.
pub fn calculate_vested_amount(env: &Env, schedule: &VestingSchedule) -> i128 {
    // Vesting revogado → não libera mais do que já foi liberado (congela no momento da revogação)
    if schedule.revoked {
        return schedule.released_amount;
    }

    let now = env.ledger().sequence();

    // Proteção contra underflow se o ledger atual for menor que o start (raro, mas possível em testes)
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

    num.checked_div(schedule.duration_ledgers as i128)
        .unwrap_or(0)
}

/// Calcula quanto está disponível para release AGORA (Vested - Já Liberado)
pub fn calculate_releasable_amount(env: &Env, schedule: &VestingSchedule) -> i128 {
    let vested = calculate_vested_amount(env, schedule);
    vested.saturating_sub(schedule.released_amount).max(0)
}

/// Cria um vesting schedule.
/// Nota: Token.rs é responsável por debitar o saldo do criador e travar no locked_balance.
pub fn create_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    total_amount: i128,
    cliff_ledgers: u32,
    duration_ledgers: u32,
    revocable: bool,
) -> Result<u32, BrazaError> {
    // Incrementa contador e valida limites globais/usuário
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

    // Emite evento específico de criação interna
    env.events().publish(
        (symbol_short!("v_new"), beneficiary, schedule_id),
        total_amount,
    );

    Ok(schedule_id)
}

/// Libera tokens vestidos.
/// Retorna o valor liberado para que o Token.rs faça a transferência efetiva.
pub fn release_vested_tokens(
    env: &Env,
    beneficiary: &Address,
    schedule_id: u32,
) -> Result<i128, BrazaError> {
    let mut schedule = storage::get_vesting_schedule(env, beneficiary, schedule_id)
        .ok_or(BrazaError::VestingNotFound)?;

    // Se já foi revogado, não pode liberar mais nada além do que já foi pago
    if schedule.revoked {
        return Err(BrazaError::VestingNotFound);
    }

    let releasable = calculate_releasable_amount(env, &schedule);

    if releasable <= 0 {
        return Err(BrazaError::NoTokensToRelease);
    }

    // Atualizar schedule com o novo valor liberado
    schedule.released_amount = schedule
        .released_amount
        .checked_add(releasable)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);

    Ok(releasable)
}

/// Revoga um vesting schedule.
/// Retorna o valor "unvested" (que volta para o admin).
pub fn revoke_vesting_schedule(
    env: &Env,
    beneficiary: &Address,
    schedule_id: u32,
) -> Result<i128, BrazaError> {
    let mut schedule = storage::get_vesting_schedule(env, beneficiary, schedule_id)
        .ok_or(BrazaError::VestingNotFound)?;

    if !schedule.revocable {
        return Err(BrazaError::Unauthorized); // Ou NotRevocable
    }

    if schedule.revoked {
        return Err(BrazaError::VestingNotFound); // Já revogado
    }

    // Calcula o que o usuário tem direito até agora
    let vested = calculate_vested_amount(env, &schedule);

    // O restante (Total - Vested) é o que será devolvido ao admin
    let unvested = schedule.total_amount.saturating_sub(vested);

    // Marca como revogado
    schedule.revoked = true;
    // Opcional: Atualiza released_amount para igualar vested, para travar o estado visualmente
    // schedule.released_amount = vested;

    storage::set_vesting_schedule(env, beneficiary, schedule_id, &schedule);

    Ok(unvested)
}

// ============================================================================
// MÓDULO DE TESTES (EXPORTADO)
// ============================================================================

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub mod test_utils {

    // Re-exportar funções internas para testes de integração
    pub use super::{
        calculate_releasable_amount, calculate_vested_amount, create_vesting_schedule,
        release_vested_tokens, revoke_vesting_schedule,
    };
}
