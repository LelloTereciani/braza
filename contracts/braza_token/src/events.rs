use soroban_sdk::{symbol_short, Address, Env};

// ============================================================================
// EVENTOS DO TOKEN (SEP-41 + Custom)
// ============================================================================

// Transferência padrão SEP‑0041
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("transfer"), from, to), amount);
}

// Mint padrão SEP‑0041
pub fn emit_mint(env: &Env, to: &Address, amount: i128) {
    env.events().publish((symbol_short!("mint"), to), amount);
}

// Burn padrão SEP‑0041
pub fn emit_burn(env: &Env, from: &Address, amount: i128) {
    env.events().publish((symbol_short!("burn"), from), amount);
}

// Evento de aprovação (compatível SEP‑41 + ERC‑20)
pub fn emit_approval(env: &Env, owner: &Address, spender: &Address, amount: i128) {
    env.events()
        .publish((symbol_short!("approve"), owner, spender), amount);
}

// Pausa
pub fn emit_pause(env: &Env) {
    env.events().publish((symbol_short!("pause"),), true);
}

// Despausa
pub fn emit_unpause(env: &Env) {
    env.events().publish((symbol_short!("unpause"),), true);
}

// Blacklist / unblacklist
pub fn emit_blacklist(env: &Env, addr: &Address, blacklisted: bool) {
    env.events()
        .publish((symbol_short!("blklst"), addr), blacklisted);
}

// Vesting criado
pub fn emit_vesting_created(env: &Env, beneficiary: &Address, schedule_id: u32, amount: i128) {
    env.events()
        .publish((symbol_short!("v_new"), beneficiary, schedule_id), amount);
}

// Vesting liberado
pub fn emit_vesting_released(env: &Env, beneficiary: &Address, schedule_id: u32, amount: i128) {
    env.events()
        .publish((symbol_short!("v_rel"), beneficiary, schedule_id), amount);
}

// Vesting revogado
pub fn emit_vesting_revoked(env: &Env, beneficiary: &Address, schedule_id: u32) {
    env.events()
        .publish((symbol_short!("v_rev"), beneficiary, schedule_id), true);
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events as _},
        Env,
    };
    // Importa a struct real do token para registro
    use crate::BrazaToken;

    #[test]
    fn test_emit_transfer() {
        let env = Env::default();
        // Usa a struct BrazaToken diretamente
        let contract_id = env.register_contract(None, BrazaToken);

        env.as_contract(&contract_id, || {
            let a = Address::generate(&env);
            let b = Address::generate(&env);
            emit_transfer(&env, &a, &b, 1000);
        });

        assert_eq!(env.events().all().len(), 1);
    }

    #[test]
    fn test_approval() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);

        env.as_contract(&contract_id, || {
            let o = Address::generate(&env);
            let s = Address::generate(&env);
            emit_approval(&env, &o, &s, 50);
        });

        assert_eq!(env.events().all().len(), 1);
    }

    #[test]
    fn test_burn() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);

        env.as_contract(&contract_id, || {
            let a = Address::generate(&env);
            emit_burn(&env, &a, 33);
        });

        assert_eq!(env.events().all().len(), 1);
    }

    #[test]
    fn test_pause_unpause() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);

        env.as_contract(&contract_id, || {
            emit_pause(&env);
            emit_unpause(&env);
        });

        assert_eq!(env.events().all().len(), 2);
    }

    #[test]
    fn test_blacklist() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);

        env.as_contract(&contract_id, || {
            let a = Address::generate(&env);
            emit_blacklist(&env, &a, true);
            emit_blacklist(&env, &a, false);
        });

        assert_eq!(env.events().all().len(), 2);
    }

    #[test]
    fn test_vesting() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);

        env.as_contract(&contract_id, || {
            let b = Address::generate(&env);

            emit_vesting_created(&env, &b, 1, 1000);
            emit_vesting_released(&env, &b, 1, 500);
            emit_vesting_revoked(&env, &b, 1);
        });

        assert_eq!(env.events().all().len(), 3);
    }
}

// ============================================================================
// MÓDULO DE TESTES (EXPORTADO)
// ============================================================================

// Permite uso em 'cargo test' E quando a feature 'testutils' estiver ativa
#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub mod test_utils {

    // Re-exportar funções internas para uso em testes de integração
    pub use super::{
        emit_approval, emit_blacklist, emit_burn, emit_mint, emit_pause, emit_transfer,
        emit_unpause, emit_vesting_created, emit_vesting_released, emit_vesting_revoked,
    };
}
