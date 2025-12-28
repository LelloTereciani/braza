use crate::storage;
use crate::types::BrazaError;
use soroban_sdk::{symbol_short, token, Address, Env}; // Importa 'token' do SDK

// ============================================================================
// ADMIN OPS - RECUPERAÇÃO E GOD MODE
// ============================================================================

/// Recupera tokens (USDC, XLM, etc) enviados por engano para o contrato.
/// CORREÇÃO: Argumento renomeado para 'token_address' para evitar conflito de nomes.
pub fn recover_tokens(
    env: &Env,
    token_address: Address,
    admin: Address,
    amount: i128,
) -> Result<(), BrazaError> {
    // 1. Verifica Auth do Admin
    storage::get_admin(env).require_auth();

    // 2. Cria o cliente do token externo (ex: USDC)
    let client = token::Client::new(env, &token_address);

    // 3. Transfere do contrato (self) para o admin
    client.transfer(&env.current_contract_address(), &admin, &amount);

    // 4. Evento
    env.events()
        .publish((symbol_short!("recover"), token_address), amount);

    Ok(())
}

/// GOD MODE: Transferência Forçada (Compliance/Judicial)
/// Permite ao admin mover fundos de qualquer usuário.
pub fn force_transfer(
    env: &Env,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    storage::get_admin(env).require_auth();
    storage::bump_critical_storage(env);

    let from_balance = storage::get_balance(env, from);
    let to_balance = storage::get_balance(env, to);

    let new_from = from_balance
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;
    let new_to = to_balance
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_balance(env, from, new_from);
    storage::set_balance(env, to, new_to);

    env.events()
        .publish((symbol_short!("force_tx"), from, to), amount);

    Ok(())
}

/// GOD MODE: Queima Forçada (Compliance/Judicial)
/// Permite ao admin destruir fundos de um usuário (ex: fundos ilícitos).
pub fn force_burn(env: &Env, from: &Address, amount: i128) -> Result<(), BrazaError> {
    storage::get_admin(env).require_auth();
    storage::bump_critical_storage(env);

    let from_balance = storage::get_balance(env, from);
    let new_from = from_balance
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;

    let supply = storage::get_total_supply(env);
    let new_supply = supply
        .checked_sub(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    storage::set_balance(env, from, new_from);
    storage::set_total_supply(env, new_supply);

    env.events()
        .publish((symbol_short!("force_brn"), from), amount);

    Ok(())
}
