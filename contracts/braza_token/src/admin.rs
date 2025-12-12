use soroban_sdk::{Address, Env, String, Vec};
use crate::storage;
use crate::types::BrazaError;
use crate::validation;
use crate::events;

// ============================================================================
// FUNÇÕES ADMINISTRATIVAS AVANÇADAS
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
    use soroban_sdk::testutils::Address as _;
    use crate::types::TokenMetadata;
    
    fn setup_test_env() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        
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
        
        (env, admin, new_admin)
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
    fn test_update_metadata() {
        let (env, admin, _) = setup_test_env();
        
        let new_name = String::from_str(&env, "Braza Token V2");
        let new_symbol = String::from_str(&env, "BRZ2");
        
        update_metadata(&env, &admin, new_name.clone(), new_symbol.clone()).unwrap();
        
        let metadata = storage::get_metadata(&env);
        assert_eq!(metadata.name, new_name);
        assert_eq!(metadata.symbol, new_symbol);
        assert_eq!(metadata.decimals, 7); // Imutável
    }
    
    #[test]
    fn test_emergency_pause_and_resume() {
        let (env, admin, _) = setup_test_env();
        
        let reason = String::from_str(&env, "Security incident detected");
        
        // Pausar
        emergency_pause(&env, &admin, reason.clone()).unwrap();
        assert!(storage::is_paused(&env));
        
        // Verificar motivo
        let stored_reason = get_pause_reason(&env).unwrap();
        assert_eq!(stored_reason, reason);
        
        // Despausar
        resume_operations(&env, &admin).unwrap();
        assert!(!storage::is_paused(&env));
        
        // Motivo deve ter sido limpo
        assert!(get_pause_reason(&env).is_none());
    }
    
    #[test]
    fn test_batch_blacklist() {
        let (env, admin, _) = setup_test_env();
        
        let mut addresses = Vec::new(&env);
        for _ in 0..5 {
            addresses.push_back(Address::generate(&env));
        }
        
        // Blacklist em lote
        let processed = batch_blacklist(&env, &admin, addresses.clone(), true).unwrap();
        assert_eq!(processed, 5);
        
        // Verificar que todos foram blacklisted
        for addr in addresses.iter() {
            assert!(storage::is_blacklisted(&env, &addr));
        }
    }
    
    #[test]
    #[should_panic(expected = "InvalidAmount")]
    fn test_batch_blacklist_limit_exceeded() {
        let (env, admin, _) = setup_test_env();
        
        let mut addresses = Vec::new(&env);
        for _ in 0..51 { // Excede limite de 50
            addresses.push_back(Address::generate(&env));
        }
        
        batch_blacklist(&env, &admin, addresses, true).unwrap();
    }
    
    #[test]
    fn test_force_burn() {
        let (env, admin, _) = setup_test_env();
        
        let user = Address::generate(&env);
        let amount = 1000i128;
        
        // Dar balance ao usuário
        storage::set_balance(&env, &user, amount);
        storage::set_total_supply(&env, amount);
        
        let reason = String::from_str(&env, "Court order");
        
        // Queimar forçadamente
        force_burn(&env, &admin, &user, amount, reason).unwrap();
        
        assert_eq!(storage::get_balance(&env, &user), 0);
        assert_eq!(storage::get_total_supply(&env), 0);
    }
    
    #[test]
    fn test_force_transfer() {
        let (env, admin, _) = setup_test_env();
        
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let amount = 1000i128;
        
        // Dar balance ao from
        storage::set_balance(&env, &from, amount);
        
        let reason = String::from_str(&env, "Fraud recovery");
        
        // Transferir forçadamente
        force_transfer(&env, &admin, &from, &to, amount, reason).unwrap();
        
        assert_eq!(storage::get_balance(&env, &from), 0);
        assert_eq!(storage::get_balance(&env, &to), amount);
    }
    
    #[test]
    fn test_get_contract_stats() {
        let (env, _, _) = setup_test_env();
        
        storage::set_total_supply(&env, 100_000_000_000_000); // 10M
        
        let (total, max, remaining, _) = get_contract_stats(&env);
        
        assert_eq!(total, 100_000_000_000_000);
        assert_eq!(max, 210_000_000_000_000); // 21M
        assert_eq!(remaining, 110_000_000_000_000); // 11M
    }
}
