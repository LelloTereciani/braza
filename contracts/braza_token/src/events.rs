use soroban_sdk::{Address, Env, symbol_short};

// ============================================================================
// EVENTOS DO TOKEN
// ============================================================================

/// Emite evento de transferência
/// 
/// # Conformidade SEP:
/// - SEP-0041: Evento padrão de transferência de tokens
/// 
/// # Formato:
/// - Topics: ("transfer", from, to)
/// - Data: amount
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("transfer"), from, to),
        amount,
    );
}

/// Emite evento de mint (criação de tokens)
/// 
/// # Conformidade SEP:
/// - SEP-0041: Evento de criação de novos tokens
/// 
/// # Formato:
/// - Topics: ("mint", to)
/// - Data: amount
pub fn emit_mint(env: &Env, to: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("mint"), to),
        amount,
    );
}

/// Emite evento de burn (destruição de tokens)
/// 
/// # Conformidade SEP:
/// - SEP-0041: Evento de destruição de tokens
/// 
/// # Formato:
/// - Topics: ("burn", from)
/// - Data: amount
pub fn emit_burn(env: &Env, from: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("burn"), from),
        amount,
    );
}

/// ✅ NOVO: Emite evento de aprovação de allowance
/// 
/// # Conformidade SEP:
/// - SEP-0041: Evento padrão de aprovação de allowance
/// - Compatível com ERC-20 Approval event
/// 
/// # Formato:
/// - Topics: ("approve", owner, spender)
/// - Data: amount
/// 
/// # Uso:
/// - Chamado após approve(), increase_allowance(), decrease_allowance()
/// - Permite que frontends e indexadores monitorem aprovações
/// - Essencial para auditoria de segurança
pub fn emit_approval(env: &Env, from: &Address, spender: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("approve"), from, spender),
        amount,
    );
}

/// Emite evento de pausa do contrato
/// 
/// # Formato:
/// - Topics: ("pause",)
/// - Data: true
pub fn emit_pause(env: &Env) {
    env.events().publish(
        (symbol_short!("pause"),),
        true,
    );
}

/// Emite evento de despausa do contrato
/// 
/// # Formato:
/// - Topics: ("unpause",)
/// - Data: true
pub fn emit_unpause(env: &Env) {
    env.events().publish(
        (symbol_short!("unpause"),),
        true,
    );
}

/// Emite evento de blacklist (adicionar/remover)
/// 
/// # Formato:
/// - Topics: ("blacklist", addr)
/// - Data: blacklisted (true = adicionado, false = removido)
pub fn emit_blacklist(env: &Env, addr: &Address, blacklisted: bool) {
    env.events().publish(
        (symbol_short!("blacklist"), addr),
        blacklisted,
    );
}

/// Emite evento de criação de vesting schedule
/// 
/// # Formato:
/// - Topics: ("vest_new", beneficiary, schedule_id)
/// - Data: amount
pub fn emit_vesting_created(env: &Env, beneficiary: &Address, schedule_id: u32, amount: i128) {
    env.events().publish(
        (symbol_short!("vest_new"), beneficiary, schedule_id),
        amount,
    );
}

/// Emite evento de release de tokens vestidos
/// 
/// # Formato:
/// - Topics: ("vest_rel", beneficiary, schedule_id)
/// - Data: amount
pub fn emit_vesting_released(env: &Env, beneficiary: &Address, schedule_id: u32, amount: i128) {
    env.events().publish(
        (symbol_short!("vest_rel"), beneficiary, schedule_id),
        amount,
    );
}

/// Emite evento de revogação de vesting schedule
/// 
/// # Formato:
/// - Topics: ("vest_rev", beneficiary, schedule_id)
/// - Data: true
pub fn emit_vesting_revoked(env: &Env, beneficiary: &Address, schedule_id: u32) {
    env.events().publish(
        (symbol_short!("vest_rev"), beneficiary, schedule_id),
        true,
    );
}

// ============================================================================
// ✅ TESTES UNITÁRIOS - EVENTOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};
    
    #[test]
    fn test_emit_transfer() {
        let env = Env::default();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        
        emit_transfer(&env, &from, &to, 1000);
        
        // Verificar que evento foi emitido
        let events = env.events().all();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_emit_approval() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);
        
        emit_approval(&env, &owner, &spender, 5000);
        
        // Verificar que evento foi emitido
        let events = env.events().all();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_emit_mint() {
        let env = Env::default();
        let to = Address::generate(&env);
        
        emit_mint(&env, &to, 10000);
        
        let events = env.events().all();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_emit_burn() {
        let env = Env::default();
        let from = Address::generate(&env);
        
        emit_burn(&env, &from, 500);
        
        let events = env.events().all();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_emit_pause_unpause() {
        let env = Env::default();
        
        emit_pause(&env);
        emit_unpause(&env);
        
        let events = env.events().all();
        assert_eq!(events.len(), 2);
    }
    
    #[test]
    fn test_emit_blacklist() {
        let env = Env::default();
        let addr = Address::generate(&env);
        
        // Adicionar à blacklist
        emit_blacklist(&env, &addr, true);
        
        // Remover da blacklist
        emit_blacklist(&env, &addr, false);
        
        let events = env.events().all();
        assert_eq!(events.len(), 2);
    }
    
    #[test]
    fn test_emit_vesting_events() {
        let env = Env::default();
        let beneficiary = Address::generate(&env);
        let schedule_id = 0u32;
        
        // Criar vesting
        emit_vesting_created(&env, &beneficiary, schedule_id, 10000);
        
        // Release vesting
        emit_vesting_released(&env, &beneficiary, schedule_id, 5000);
        
        // Revogar vesting
        emit_vesting_revoked(&env, &beneficiary, schedule_id);
        
        let events = env.events().all();
        assert_eq!(events.len(), 3);
    }
    
    #[test]
    fn test_multiple_approvals() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let spender1 = Address::generate(&env);
        let spender2 = Address::generate(&env);
        
        // Múltiplas aprovações
        emit_approval(&env, &owner, &spender1, 1000);
        emit_approval(&env, &owner, &spender2, 2000);
        emit_approval(&env, &owner, &spender1, 0); // Revogar
        
        let events = env.events().all();
        assert_eq!(events.len(), 3);
    }
}
