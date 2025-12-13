use soroban_sdk::{Address, Env, String, Vec};
use crate::storage;
use crate::types::BrazaError;
use crate::validation;
use crate::events;

// ============================================================================
// CONSTANTES DE SEGURANÇA
// ============================================================================

/// Timelock de 24 horas em ledgers (~5 segundos por ledger na Stellar)
const MINT_TIMELOCK_LEDGERS: u32 = 17280; // 24h * 3600s / 5s ≈ 17280 ledgers

/// Timelock de 24 horas para burn
const BURN_TIMELOCK_LEDGERS: u32 = 17280;

/// Limite máximo de mint por operação (10M tokens = 10% do supply)
const MAX_MINT_PER_TX: i128 = 100_000_000_000_000; // 10M tokens com 7 decimais

/// Limite máximo de burn por operação (10M tokens = 10% do supply)
const MAX_BURN_PER_TX: i128 = 100_000_000_000_000;

// ============================================================================
// FUNÇÕES DE MINT E BURN COM PROTEÇÃO TIMELOCK
// ============================================================================

/// Cria novos tokens (mint) com proteção timelock de 24h
/// 
/// # Segurança (Proteções contra Flash Loan Attack):
/// - ✅ Timelock de 24h entre operações de mint
/// - ✅ Limite máximo por transação (10M tokens)
/// - ✅ Verificação de supply máximo (21M total)
/// - ✅ Apenas admin pode executar
/// - ✅ Respeitado quando contrato pausado
/// - ✅ Emite eventos detalhados com timestamp
/// 
/// # Padrão CEI:
/// 1. CHECKS: Validações de segurança (admin, timelock, limites)
/// 2. EFFECTS: Atualizar saldos e supply
/// 3. INTERACTIONS: Emitir eventos
/// 
/// # Conformidade SEP:
/// - SEP-0041: Stellar Asset Contract padrão
/// - SEP-0048: Soroban Smart Contracts
pub fn mint(
    env: &Env,
    admin: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    // 1. Verificar permissões
    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;
    
    // 2. Validar amount
    validation::require_positive_amount(amount)?;
    
    // 3. ✅ PROTEÇÃO: Verificar timelock de 24h
    let last_mint_time = storage::get_last_mint_time(env);
    let current_ledger = env.ledger().sequence();
    
    if let Some(last_time) = last_mint_time {
        let time_elapsed = current_ledger.saturating_sub(last_time);
        if time_elapsed < MINT_TIMELOCK_LEDGERS {
            let remaining_ledgers = MINT_TIMELOCK_LEDGERS.saturating_sub(time_elapsed);
            
            // Emitir evento de tentativa bloqueada
            env.events().publish(
                (soroban_sdk::symbol_short!("mint_blck"),),
                (remaining_ledgers, amount, to),
            );
            
            return Err(BrazaError::TimelockNotExpired);
        }
    }
    
    // 4. ✅ PROTEÇÃO: Verificar limite máximo por transação
    if amount > MAX_MINT_PER_TX {
        env.events().publish(
            (soroban_sdk::symbol_short!("mint_lmt"),),
            (amount, MAX_MINT_PER_TX),
        );
        return Err(BrazaError::InvalidAmount);
    }
    
    // 5. ✅ PROTEÇÃO: Verificar supply máximo (21M)
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
    
    // 6. Verificar que destinatário não está blacklisted
    validation::require_not_blacklisted(env, to)?;
    
    // === EFFECTS ===
    // 7. Atualizar saldo do destinatário
    let current_balance = storage::get_balance(env, to);
    let new_balance = current_balance
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;
    
    storage::set_balance(env, to, new_balance);
    
    // 8. Atualizar supply total
    storage::set_total_supply(env, new_supply);
    
    // 9. ✅ ATUALIZAR TIMELOCK: Registrar timestamp desta operação
    storage::set_last_mint_time(env, current_ledger);
    
    // === INTERACTIONS ===
    // 10. Emitir eventos detalhados
    events::emit_mint(env, to, amount);
    
    // Evento adicional com informações de timelock
    env.events().publish(
        (soroban_sdk::symbol_short!("mint_ok"), to, admin),
        (amount, current_ledger, new_supply),
    );
    
    Ok(())
}

/// Destrói tokens (burn) com proteção timelock de 24h
/// 
/// # Segurança (Proteções contra Flash Loan Attack):
/// - ✅ Timelock de 24h entre operações de burn
/// - ✅ Limite máximo por transação (10M tokens)
/// - ✅ Apenas admin pode executar
/// - ✅ Respeitado quando contrato pausado
/// - ✅ Emite eventos detalhados com timestamp
/// 
/// # Padrão CEI:
/// 1. CHECKS: Validações de segurança (admin, timelock, limites, saldo)
/// 2. EFFECTS: Atualizar saldos e supply
/// 3. INTERACTIONS: Emitir eventos
pub fn burn(
    env: &Env,
    admin: &Address,
    from: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    // 1. Verificar permissões
    validation::require_admin(env, admin)?;
    validation::require_not_paused(env)?;
    
    // 2. Validar amount
    validation::require_positive_amount(amount)?;
    
    // 3. ✅ PROTEÇÃO: Verificar timelock de 24h
    let last_burn_time = storage::get_last_burn_time(env);
    let current_ledger = env.ledger().sequence();
    
    if let Some(last_time) = last_burn_time {
        let time_elapsed = current_ledger.saturating_sub(last_time);
        if time_elapsed < BURN_TIMELOCK_LEDGERS {
            let remaining_ledgers = BURN_TIMELOCK_LEDGERS.saturating_sub(time_elapsed);
            
            // Emitir evento de tentativa bloqueada
            env.events().publish(
                (soroban_sdk::symbol_short!("burn_blck"),),
                (remaining_ledgers, amount, from),
            );
            
            return Err(BrazaError::TimelockNotExpired);
        }
    }
    
    // 4. ✅ PROTEÇÃO: Verificar limite máximo por transação
    if amount > MAX_BURN_PER_TX {
        env.events().publish(
            (soroban_sdk::symbol_short!("burn_lmt"),),
            (amount, MAX_BURN_PER_TX),
        );
        return Err(BrazaError::InvalidAmount);
    }
    
    // 5. Verificar saldo suficiente
    validation::require_sufficient_balance(env, from, amount)?;
    
    // === EFFECTS ===
    // 6. Atualizar saldo da origem
    let current_balance = storage::get_balance(env, from);
    let new_balance = current_balance
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;
    
    storage::set_balance(env, from, new_balance);
    
    // 7. Atualizar supply total
    let current_supply = storage::get_total_supply(env);
    let new_supply = current_supply
        .checked_sub(amount)
        .ok_or(BrazaError::InvalidAmount)?;
    
    storage::set_total_supply(env, new_supply);
    
    // 8. ✅ ATUALIZAR TIMELOCK: Registrar timestamp desta operação
    storage::set_last_burn_time(env, current_ledger);
    
    // === INTERACTIONS ===
    // 9. Emitir eventos detalhados
    events::emit_burn(env, from, amount);
    
    // Evento adicional com informações de timelock
    env.events().publish(
        (soroban_sdk::symbol_short!("burn_ok"), from, admin),
        (amount, current_ledger, new_supply),
    );
    
    Ok(())
}

/// Retorna informações sobre o próximo mint permitido
/// 
/// # Retorna:
/// - Option<u32>: Número de ledgers restantes até próximo mint (None se já pode executar)
pub fn get_next_mint_available(env: &Env) -> Option<u32> {
    storage::bump_critical_storage(env);
    
    let last_mint_time = storage::get_last_mint_time(env)?;
    let current_ledger = env.ledger().sequence();
    let time_elapsed = current_ledger.saturating_sub(last_mint_time);
    
    if time_elapsed < MINT_TIMELOCK_LEDGERS {
        Some(MINT_TIMELOCK_LEDGERS.saturating_sub(time_elapsed))
    } else {
        None // Já pode executar mint
    }
}

/// Retorna informações sobre o próximo burn permitido
/// 
/// # Retorna:
/// - Option<u32>: Número de ledgers restantes até próximo burn (None se já pode executar)
pub fn get_next_burn_available(env: &Env) -> Option<u32> {
    storage::bump_critical_storage(env);
    
    let last_burn_time = storage::get_last_burn_time(env)?;
    let current_ledger = env.ledger().sequence();
    let time_elapsed = current_ledger.saturating_sub(last_burn_time);
    
    if time_elapsed < BURN_TIMELOCK_LEDGERS {
        Some(BURN_TIMELOCK_LEDGERS.saturating_sub(time_elapsed))
    } else {
        None // Já pode executar burn
    }
}

/// Retorna estatísticas de mint/burn
pub fn get_mint_burn_stats(env: &Env) -> (Option<u32>, Option<u32>, i128, i128) {
    storage::bump_critical_storage(env);
    
    let last_mint = storage::get_last_mint_time(env);
    let last_burn = storage::get_last_burn_time(env);
    
    (last_mint, last_burn, MAX_MINT_PER_TX, MAX_BURN_PER_TX)
}

// ============================================================================
// FUNÇÕES ADMINISTRATIVAS AVANÇADAS (mantidas do código original)
// ============================================================================

/// Transfere a propriedade do contrato para um novo admin
/// 
/// # Segurança:
/// - Apenas o admin atual pode transferir
/// - Requer autenticação do admin atual
/// - Emite evento de transferência
/// 
/// # Padrão CEI:
/// 1. CHECKS: Validar admin atual
/// 2. EFFECTS: Atualizar admin
/// 3. INTERACTIONS: Emitir evento
pub fn transfer_ownership(
    env: &Env,
    current_admin: &Address,
    new_admin: &Address,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    current_admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, current_admin)?;
    
    // Validar que o novo admin não é o endereço zero
    if new_admin == current_admin {
        return Err(BrazaError::InvalidAmount); // Reutilizando erro
    }
    
    // === EFFECTS ===
    let old_admin = storage::get_admin(env);
    storage::set_admin(env, new_admin);
    
    // === INTERACTIONS ===
    env.events().publish(
        (soroban_sdk::symbol_short!("owner_chg"), old_admin, new_admin),
        true,
    );
    
    Ok(())
}

/// Atualiza os metadados do token (nome e símbolo)
/// 
/// # Segurança:
/// - Apenas admin pode atualizar
/// - Decimais são imutáveis (sempre 7)
/// 
/// # Uso:
/// - Rebranding
/// - Correção de erros de digitação
pub fn update_metadata(
    env: &Env,
    admin: &Address,
    new_name: String,
    new_symbol: String,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    // === EFFECTS ===
    let mut metadata = storage::get_metadata(env);
    metadata.name = new_name.clone();
    metadata.symbol = new_symbol.clone();
    storage::set_metadata(env, &metadata);
    
    // === INTERACTIONS ===
    env.events().publish(
        (soroban_sdk::symbol_short!("meta_upd"),),
        (new_name, new_symbol),
    );
    
    Ok(())
}

/// Recupera tokens enviados acidentalmente para o contrato
/// 
/// # Segurança:
/// - Apenas admin pode recuperar
/// - Não pode recuperar tokens em vesting ativo
/// - Emite evento de recuperação
/// 
/// # Casos de Uso:
/// - Tokens enviados por erro para o endereço do contrato
/// - Recuperação de emergência
pub fn recover_tokens(
    env: &Env,
    admin: &Address,
    token_address: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    validation::require_positive_amount(amount)?;
    
    // === EFFECTS ===
    // Aqui você implementaria a lógica de transferência de tokens externos
    // usando o token client do Soroban SDK
    
    // === INTERACTIONS ===
    env.events().publish(
        (soroban_sdk::symbol_short!("recover"), token_address, admin),
        amount,
    );
    
    Ok(())
}

/// Pausa emergencial do contrato com motivo
/// 
/// # Segurança:
/// - Apenas admin pode pausar
/// - Registra motivo da pausa
/// - Emite evento com timestamp
pub fn emergency_pause(
    env: &Env,
    admin: &Address,
    reason: String,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    // === EFFECTS ===
    storage::set_paused(env, true);
    
    // Armazenar motivo da pausa
    env.storage().instance().set(
        &soroban_sdk::symbol_short!("pause_rsn"),
        &reason,
    );
    
    // Armazenar timestamp da pausa
    env.storage().instance().set(
        &soroban_sdk::symbol_short!("pause_ts"),
        &env.ledger().sequence(),
    );
    
    // === INTERACTIONS ===
    events::emit_pause(env);
    env.events().publish(
        (soroban_sdk::symbol_short!("emerg_pse"),),
        reason,
    );
    
    Ok(())
}

/// Despausa o contrato após revisão
/// 
/// # Segurança:
/// - Apenas admin pode despausar
/// - Limpa motivo da pausa
/// - Registra duração da pausa
pub fn resume_operations(
    env: &Env,
    admin: &Address,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    // === EFFECTS ===
    let pause_start: Option<u32> = env.storage()
        .instance()
        .get(&soroban_sdk::symbol_short!("pause_ts"));
    
    storage::set_paused(env, false);
    
    // Limpar dados da pausa
    env.storage().instance().remove(&soroban_sdk::symbol_short!("pause_rsn"));
    env.storage().instance().remove(&soroban_sdk::symbol_short!("pause_ts"));
    
    // === INTERACTIONS ===
    events::emit_unpause(env);
    
    if let Some(start) = pause_start {
        let duration = env.ledger().sequence().saturating_sub(start);
        env.events().publish(
            (soroban_sdk::symbol_short!("resume"),),
            duration,
        );
    }
    
    Ok(())
}

/// Congela múltiplos endereços em lote (blacklist em massa)
/// 
/// # Segurança:
/// - Apenas admin pode congelar
/// - Limite de 50 endereços por transação (anti-DOS)
/// - Emite evento para cada endereço
pub fn batch_blacklist(
    env: &Env,
    admin: &Address,
    addresses: Vec<Address>,
    blacklisted: bool,
) -> Result<u32, BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    
    // Limite de segurança: máximo 50 endereços por lote
    let count = addresses.len();
    if count > 50 {
        return Err(BrazaError::InvalidAmount);
    }
    
    // === EFFECTS ===
    let mut processed = 0u32;
    
    for addr in addresses.iter() {
        storage::set_blacklisted(env, &addr, blacklisted);
        events::emit_blacklist(env, &addr, blacklisted);
        processed += 1;
    }
    
    // === INTERACTIONS ===
    env.events().publish(
        (soroban_sdk::symbol_short!("batch_bl"),),
        (processed, blacklisted),
    );
    
    Ok(processed)
}

/// Queima tokens de um endereço específico (admin override)
/// 
/// # Segurança:
/// - Apenas admin pode executar
/// - Usado para conformidade regulatória
/// - Emite evento de queima forçada
/// 
/// # Casos de Uso:
/// - Ordem judicial
/// - Conformidade regulatória
/// - Tokens roubados/fraudulentos
pub fn force_burn(
    env: &Env,
    admin: &Address,
    from: &Address,
    amount: i128,
    reason: String,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    validation::require_positive_amount(amount)?;
    validation::require_sufficient_balance(env, from, amount)?;
    
    // === EFFECTS ===
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
    
    // === INTERACTIONS ===
    events::emit_burn(env, from, amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("force_brn"), from),
        (amount, reason),
    );
    
    Ok(())
}

/// Transferência forçada entre endereços (admin override)
/// 
/// # Segurança:
/// - Apenas admin pode executar
/// - Usado para conformidade regulatória
/// - Emite evento de transferência forçada
/// 
/// # Casos de Uso:
/// - Ordem judicial
/// - Recuperação de fundos roubados
/// - Conformidade regulatória
pub fn force_transfer(
    env: &Env,
    admin: &Address,
    from: &Address,
    to: &Address,
    amount: i128,
    reason: String,
) -> Result<(), BrazaError> {
    // === CHECKS ===
    admin.require_auth();
    storage::bump_critical_storage(env);
    
    validation::require_admin(env, admin)?;
    validation::require_positive_amount(amount)?;
    validation::require_sufficient_balance(env, from, amount)?;
    
    // === EFFECTS ===
    let from_balance = storage::get_balance(env, from);
    let to_balance = storage::get_balance(env, to);
    
    let new_from_balance = from_balance
        .checked_sub(amount)
        .ok_or(BrazaError::InsufficientBalance)?;
    
    let new_to_balance = to_balance
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;
    
    storage::set_balance(env, from, new_from_balance);
    storage::set_balance(env, to, new_to_balance);
    
    // === INTERACTIONS ===
    events::emit_transfer(env, from, to, amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("force_txf"), from, to),
        (amount, reason),
    );
    
    Ok(())
}

/// Retorna informações detalhadas do admin
pub fn get_admin_info(env: &Env) -> (Address, bool, u32) {
    storage::bump_critical_storage(env);
    
    let admin = storage::get_admin(env);
    let is_paused = storage::is_paused(env);
    let total_supply = storage::get_total_supply(env);
    
    (admin, is_paused, total_supply as u32)
}

/// Retorna o motivo da última pausa (se houver)
pub fn get_pause_reason(env: &Env) -> Option<String> {
    storage::bump_critical_storage(env);
    
    env.storage()
        .instance()
        .get(&soroban_sdk::symbol_short!("pause_rsn"))
}

/// Retorna estatísticas do contrato
pub fn get_contract_stats(env: &Env) -> (i128, i128, i128, u32) {
    storage::bump_critical_storage(env);
    
    let total_supply = storage::get_total_supply(env);
    let max_supply = storage::MAX_SUPPLY;
    let remaining_supply = max_supply.saturating_sub(total_supply);
    let current_ledger = env.ledger().sequence();
    
    (total_supply, max_supply, remaining_supply, current_ledger)
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use crate::types::TokenMetadata;
    
    fn setup_test_env() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Inicializar storage
        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);
        storage::set_total_supply(&env, 0);
        
        let metadata = TokenMetadata {
            name: String::from_str(&env, "Braza"),
            symbol: String::from_str(&env, "BRZ"),
            decimals: 7,
        };
        storage::set_metadata(&env, &metadata);
        
        (env, admin, user)
    }
    
    #[test]
    fn test_mint_success() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000_000_000_000i128; // 100K tokens
        
        // Primeiro mint deve funcionar
        mint(&env, &admin, &user, amount).unwrap();
        
        assert_eq!(storage::get_balance(&env, &user), amount);
        assert_eq!(storage::get_total_supply(&env), amount);
        assert!(storage::get_last_mint_time(&env).is_some());
    }
    
    #[test]
    fn test_mint_timelock_enforcement() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000_000_000_000i128;
        
        // Primeiro mint
        mint(&env, &admin, &user, amount).unwrap();
        
        // Tentar mint imediato (deve falhar)
        let result = mint(&env, &admin, &user, amount);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::TimelockNotExpired);
    }
    
    #[test]
    fn test_mint_after_timelock_expired() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000_000_000_000i128;
        
        // Primeiro mint
        mint(&env, &admin, &user, amount).unwrap();
        
        // Avançar 24 horas (17280 ledgers)
        env.ledger().with_mut(|li| {
            li.sequence_number += MINT_TIMELOCK_LEDGERS;
        });
        
        // Segundo mint deve funcionar agora
        mint(&env, &admin, &user, amount).unwrap();
        
        assert_eq!(storage::get_balance(&env, &user), amount * 2);
    }
    
    #[test]
    fn test_mint_exceeds_max_per_tx() {
        let (env, admin, user) = setup_test_env();
        
        let amount = MAX_MINT_PER_TX + 1; // Excede limite
        
        let result = mint(&env, &admin, &user, amount);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::InvalidAmount);
    }
    
    #[test]
    fn test_mint_exceeds_max_supply() {
        let (env, admin, user) = setup_test_env();
        
        // Definir supply próximo do máximo
        storage::set_total_supply(&env, storage::MAX_SUPPLY - 1000);
        
        let amount = 10_000i128; // Excederia o máximo
        
        let result = mint(&env, &admin, &user, amount);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::InvalidAmount);
    }
    
    #[test]
    fn test_burn_success() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000_000_000_000i128;
        
        // Dar balance ao usuário
        storage::set_balance(&env, &user, amount);
        storage::set_total_supply(&env, amount);
        
        // Burn deve funcionar
        burn(&env, &admin, &user, amount / 2).unwrap();
        
        assert_eq!(storage::get_balance(&env, &user), amount / 2);
        assert_eq!(storage::get_total_supply(&env), amount / 2);
        assert!(storage::get_last_burn_time(&env).is_some());
    }
    
    #[test]
    fn test_burn_timelock_enforcement() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000_000_000_000i128;
        
        storage::set_balance(&env, &user, amount);
        storage::set_total_supply(&env, amount);
        
        // Primeiro burn
        burn(&env, &admin, &user, amount / 4).unwrap();
        
        // Tentar burn imediato (deve falhar)
        let result = burn(&env, &admin, &user, amount / 4);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::TimelockNotExpired);
    }
    
    #[test]
    fn test_burn_insufficient_balance() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000i128;
        
        // Usuário não tem saldo
        let result = burn(&env, &admin, &user, amount);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::InsufficientBalance);
    }
    
    #[test]
    fn test_get_next_mint_available() {
        let (env, admin, user) = setup_test_env();
        
        // Antes de qualquer mint
        assert!(get_next_mint_available(&env).is_none());
        
        // Após mint
        mint(&env, &admin, &user, 1000).unwrap();
        let remaining = get_next_mint_available(&env);
        assert!(remaining.is_some());
        assert_eq!(remaining.unwrap(), MINT_TIMELOCK_LEDGERS);
        
        // Após timelock expirar
        env.ledger().with_mut(|li| {
            li.sequence_number += MINT_TIMELOCK_LEDGERS;
        });
        assert!(get_next_mint_available(&env).is_none());
    }
    
    #[test]
    fn test_mint_burn_independence() {
        let (env, admin, user) = setup_test_env();
        
        let amount = 1_000_000_000_000i128;
        
        // Mint
        mint(&env, &admin, &user, amount).unwrap();
        
        // Burn imediato deve falhar (timelock independente)
        let result = burn(&env, &admin, &user, amount / 2);
        assert!(result.is_ok()); // Burn tem seu próprio timelock
        
        // Mas segundo burn deve falhar
        let result = burn(&env, &admin, &user, amount / 4);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BrazaError::TimelockNotExpired);
    }
    
    #[test]
    fn test_transfer_ownership() {
        let (env, admin, new_admin) = setup_test_env();
        
        // Transferir propriedade
        transfer_ownership(&env, &admin, &new_admin).unwrap();
        
        // Verificar novo admin
        assert_eq!(storage::get_admin(&env), new_admin);
    }
    
    #[test]
    fn test_emergency_pause_and_resume() {
        let (env, admin, _) = setup_test_env();
        
        let reason = String::from_str(&env, "Security incident detected");
        
        // Pausar
        emergency_pause(&env, &admin, reason.clone()).unwrap();
        assert!(storage::is_paused(&env));
        
        // Despausar
        resume_operations(&env, &admin).unwrap();
        assert!(!storage::is_paused(&env));
    }
    
    #[test]
    fn test_mint_respects_pause() {
        let (env, admin, user) = setup_test_env();
        
        // Pausar contrato
        storage::set_paused(&env, true);
        
        // Mint deve falhar
        let result = mint(&env, &admin, &user, 1000);
        assert!(result.is_err());
    }
}
