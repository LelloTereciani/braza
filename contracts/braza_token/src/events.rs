use soroban_sdk::{Address, Env, symbol_short};

// ============================================================================
// EVENTOS DO TOKEN
// ============================================================================

/// Emite evento de transferência
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("transfer"), from, to),
        amount,
    );
}

/// Emite evento de mint
pub fn emit_mint(env: &Env, to: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("mint"), to),
        amount,
    );
}

/// Emite evento de burn
pub fn emit_burn(env: &Env, from: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("burn"), from),
        amount,
    );
}

/// Emite evento de pausa
pub fn emit_pause(env: &Env) {
    env.events().publish(
        (symbol_short!("pause"),),
        true,
    );
}

/// Emite evento de despausa
pub fn emit_unpause(env: &Env) {
    env.events().publish(
        (symbol_short!("unpause"),),
        true,
    );
}

/// Emite evento de blacklist
pub fn emit_blacklist(env: &Env, addr: &Address, blacklisted: bool) {
    env.events().publish(
        (symbol_short!("blacklist"), addr),
        blacklisted,
    );
}

/// Emite evento de criação de vesting
pub fn emit_vesting_created(env: &Env, beneficiary: &Address, schedule_id: u32, amount: i128) {
    env.events().publish(
        (symbol_short!("vest_new"), beneficiary, schedule_id),
        amount,
    );
}

/// Emite evento de release de vesting
pub fn emit_vesting_released(env: &Env, beneficiary: &Address, schedule_id: u32, amount: i128) {
    env.events().publish(
        (symbol_short!("vest_rel"), beneficiary, schedule_id),
        amount,
    );
}

/// Emite evento de revogação de vesting
pub fn emit_vesting_revoked(env: &Env, beneficiary: &Address, schedule_id: u32) {
    env.events().publish(
        (symbol_short!("vest_rev"), beneficiary, schedule_id),
        true,
    );
}

/// Emite evento de mint (criação de tokens)
pub fn emit_mint(env: &Env, to: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("mint"), to),
        amount,
    );
}

/// Emite evento de burn (destruição de tokens)
pub fn emit_burn(env: &Env, from: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("burn"), from),
        amount,
    );
}
