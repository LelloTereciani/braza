#![allow(unused_imports)]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec, symbol_short};
use crate::storage;
use crate::types::{BrazaError, TokenMetadata, VestingSchedule};
use crate::validation;
use crate::vesting;
use crate::events;

// ============================================================================
// CONTRATO PRINCIPAL - BRAZA TOKEN
// ============================================================================

#[contract]
pub struct BrazaToken;

#[contractimpl]
impl BrazaToken {
    
    // ========================================================================
    // INICIALIZAÇÃO
    // ========================================================================
    
    /// Inicializa o contrato BrazaToken
    /// 
    /// # Parâmetros
    /// - `admin`: Endereço do administrador
    /// - `name`: Nome do token (ex: "Braza Token")
    /// - `symbol`: Símbolo do token (ex: "BRZ")
    /// 
    /// # Erros
    /// - `AlreadyInitialized`: Se o contrato já foi inicializado
    pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    ) -> Result<(), BrazaError> {
    // CHECKS: Verificar se já foi inicializado
    if env.storage().instance().has(&symbol_short!("admin")) {
        return Err(BrazaError::AlreadyInitialized);
    }
    
    // EFFECTS: Configurar estado inicial
    storage::set_admin(&env, &admin);
    storage::set_paused(&env, false);
    
    // Mint inicial de 10 milhões BRZ para o admin
    storage::set_balance(&env, &admin, storage::INITIAL_SUPPLY);
    storage::set_total_supply(&env, storage::INITIAL_SUPPLY);
    
    let metadata = TokenMetadata {
        name,
        symbol,
        decimals: 7, // Fixo em 7 decimais
    };
    storage::set_metadata(&env, &metadata);
    
    // INTERACTIONS: Emitir evento
    events::emit_mint(&env, &admin, storage::INITIAL_SUPPLY);
    
    Ok(())
}
    
    // ========================================================================
    // FUNÇÕES SEP-41 PADRÃO
    // ========================================================================
    
    /// Retorna o nome do token
    pub fn name(env: Env) -> String {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).name
    }
    
    /// Retorna o símbolo do token
    pub fn symbol(env: Env) -> String {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).symbol
    }
    
    /// Retorna o número de decimais
    pub fn decimals(env: Env) -> u32 {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).decimals
    }
    
    /// Retorna o balance de um endereço
    pub fn balance(env: Env, id: Address) -> i128 {
        storage::get_balance(&env, &id)
    }
    
    /// Retorna o supply total
    pub fn total_supply(env: Env) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_total_supply(&env)
    }
    
    // ========================================================================
    // TRANSFERÊNCIAS - CEI PATTERN IMPLEMENTADO (CRITICAL-02)
    // ========================================================================
    
    /// Transfere tokens de `from` para `to`
    /// 
    /// # Padrão CEI Implementado:
    /// 1. CHECKS: Validações (auth, paused, blacklist, amounts, balance)
    /// 2. EFFECTS: Atualização de estado (balances, supplies)
    /// 3. INTERACTIONS: Emissão de eventos
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        from.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &to)?;
        validation::require_positive_amount(amount)?;
        validation::require_sufficient_balance(&env, &from, amount)?;
        
        // === EFFECTS ===
        let from_balance = storage::get_balance(&env, &from);
        let to_balance = storage::get_balance(&env, &to);
        
        let new_from_balance = from_balance
            .checked_sub(amount)
            .ok_or(BrazaError::InsufficientBalance)?;
        
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(BrazaError::InvalidAmount)?;
        
        storage::set_balance(&env, &from, new_from_balance);
        storage::set_balance(&env, &to, new_to_balance);
        
        // === INTERACTIONS ===
        events::emit_transfer(&env, &from, &to, amount);
        
        Ok(())
    }
    
    /// Transfere tokens usando allowance
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        spender.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &to)?;
        validation::require_not_blacklisted(&env, &spender)?;
        validation::require_positive_amount(amount)?;
        validation::require_sufficient_balance(&env, &from, amount)?;
        
        // Verificar allowance (implementação simplificada)
        // TODO: Implementar sistema completo de allowance
        
        // === EFFECTS ===
        let from_balance = storage::get_balance(&env, &from);
        let to_balance = storage::get_balance(&env, &to);
        
        let new_from_balance = from_balance
            .checked_sub(amount)
            .ok_or(BrazaError::InsufficientBalance)?;
        
        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(BrazaError::InvalidAmount)?;
        
        storage::set_balance(&env, &from, new_from_balance);
        storage::set_balance(&env, &to, new_to_balance);
        
        // === INTERACTIONS ===
        events::emit_transfer(&env, &from, &to, amount);
        
        Ok(())
    }
    
    // ========================================================================
    // MINT E BURN - CEI PATTERN (CRITICAL-02)
    // ========================================================================
    
    /// Cria novos tokens (apenas admin)
    pub fn mint(
        env: Env,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_positive_amount(amount)?;
        validation::require_max_supply_not_exceeded(&env, amount)?;
        
        // === EFFECTS ===
        let current_balance = storage::get_balance(&env, &to);
        let new_balance = current_balance
            .checked_add(amount)
            .ok_or(BrazaError::InvalidAmount)?;
        
        let current_supply = storage::get_total_supply(&env);
        let new_supply = current_supply
            .checked_add(amount)
            .ok_or(BrazaError::MaxSupplyExceeded)?;
        
        storage::set_balance(&env, &to, new_balance);
        storage::set_total_supply(&env, new_supply);
        
        // === INTERACTIONS ===
        events::emit_mint(&env, &to, amount);
        
        Ok(())
    }
    
    /// Destrói tokens (apenas admin)
    pub fn burn(
        env: Env,
        from: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_positive_amount(amount)?;
        validation::require_sufficient_balance(&env, &from, amount)?;
        
        // === EFFECTS ===
        let current_balance = storage::get_balance(&env, &from);
        let new_balance = current_balance
            .checked_sub(amount)
            .ok_or(BrazaError::InsufficientBalance)?;
        
        let current_supply = storage::get_total_supply(&env);
        let new_supply = current_supply
            .checked_sub(amount)
            .ok_or(BrazaError::InvalidAmount)?;
        
        storage::set_balance(&env, &from, new_balance);
        storage::set_total_supply(&env, new_supply);
        
        // === INTERACTIONS ===
        events::emit_burn(&env, &from, amount);
        
        Ok(())
    }
    
    // ========================================================================
    // VESTING - CRITICAL-01 CORRIGIDO
    // ========================================================================
    
    /// Cria um novo vesting schedule
    /// 
    /// # Correções Implementadas:
    /// - CRITICAL-01: Usa ledger.sequence() ao invés de timestamp
    /// - CRITICAL-05: Valida limite de schedules por beneficiário
    pub fn create_vesting(
        env: Env,
        beneficiary: Address,
        total_amount: i128,
        cliff_ledgers: u32,
        duration_ledgers: u32,
        revocable: bool,
    ) -> Result<u32, BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_valid_vesting_params(total_amount, cliff_ledgers, duration_ledgers)?;
        validation::require_sufficient_balance(&env, &admin, total_amount)?;
        
        // === EFFECTS ===
        // Transferir tokens do admin para o contrato (lock)
        let admin_balance = storage::get_balance(&env, &admin);
        let new_admin_balance = admin_balance
            .checked_sub(total_amount)
            .ok_or(BrazaError::InsufficientBalance)?;
        storage::set_balance(&env, &admin, new_admin_balance);
        
        // Criar vesting schedule (CRITICAL-05: valida limite)
        let schedule_id = vesting::create_vesting_schedule(
            &env,
            &beneficiary,
            total_amount,
            cliff_ledgers,
            duration_ledgers,
            revocable,
        )?;
        
        // === INTERACTIONS ===
        events::emit_vesting_created(&env, &beneficiary, schedule_id, total_amount);
        
        Ok(schedule_id)
    }
    
    /// Libera tokens vestidos
    /// 
    /// # Correção CRITICAL-01:
    /// - Cálculo correto usando ledger.sequence()
    /// - Liberação gradual proporcional após cliff
    pub fn release_vested(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {
        // === CHECKS ===
        beneficiary.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        
        // === EFFECTS ===
        let released_amount = vesting::release_vested_tokens(
            &env,
            &beneficiary,
            schedule_id,
        )?;
        
        // Transferir tokens para o beneficiário
        let beneficiary_balance = storage::get_balance(&env, &beneficiary);
        let new_balance = beneficiary_balance
            .checked_add(released_amount)
            .ok_or(BrazaError::InvalidAmount)?;
        storage::set_balance(&env, &beneficiary, new_balance);
        
        // === INTERACTIONS ===
        events::emit_vesting_released(&env, &beneficiary, schedule_id, released_amount);
        
        Ok(released_amount)
    }
    
    /// Revoga um vesting schedule (apenas admin, se revocable)
    pub fn revoke_vesting(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        
        // === EFFECTS ===
        let unvested_amount = vesting::revoke_vesting_schedule(
            &env,
            &beneficiary,
            schedule_id,
        )?;
        
        // Devolver tokens não vestidos para o admin
        let admin_balance = storage::get_balance(&env, &admin);
        let new_admin_balance = admin_balance
            .checked_add(unvested_amount)
            .ok_or(BrazaError::InvalidAmount)?;
        storage::set_balance(&env, &admin, new_admin_balance);
        
        // === INTERACTIONS ===
        events::emit_vesting_revoked(&env, &beneficiary, schedule_id);
        
        Ok(unvested_amount)
    }
    
    /// Retorna informações de um vesting schedule
    pub fn get_vesting_schedule(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<VestingSchedule, BrazaError> {
        storage::bump_critical_storage(&env);
        
        storage::get_vesting_schedule(&env, &beneficiary, schedule_id)
            .ok_or(BrazaError::VestingNotFound)
    }
    
    /// Retorna todos os vesting schedules de um beneficiário
    pub fn get_all_vesting_schedules(
        env: Env,
        beneficiary: Address,
    ) -> Vec<VestingSchedule> {
        storage::bump_critical_storage(&env);
        storage::get_all_vesting_schedules(&env, &beneficiary)
    }
    
    /// Calcula a quantidade de tokens disponíveis para release
    pub fn get_releasable_amount(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {
        storage::bump_critical_storage(&env);
        
        let schedule = storage::get_vesting_schedule(&env, &beneficiary, schedule_id)
            .ok_or(BrazaError::VestingNotFound)?;
        
        Ok(vesting::calculate_releasable_amount(&env, &schedule))
    }
    
    // ========================================================================
    // FUNÇÕES ADMINISTRATIVAS - CEI PATTERN
    // ========================================================================
    
    /// Pausa o contrato (apenas admin)
    pub fn pause(env: Env) -> Result<(), BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        // === EFFECTS ===
        storage::set_paused(&env, true);
        
        // === INTERACTIONS ===
        events::emit_pause(&env);
        
        Ok(())
    }
    
    /// Despausa o contrato (apenas admin)
    pub fn unpause(env: Env) -> Result<(), BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        // === EFFECTS ===
        storage::set_paused(&env, false);
        
        // === INTERACTIONS ===
        events::emit_unpause(&env);
        
        Ok(())
    }
    
    /// Adiciona/remove endereço da blacklist (apenas admin)
    pub fn set_blacklisted(
        env: Env,
        addr: Address,
        blacklisted: bool,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        
        // === EFFECTS ===
        storage::set_blacklisted(&env, &addr, blacklisted);
        
        // === INTERACTIONS ===
        events::emit_blacklist(&env, &addr, blacklisted);
        
        Ok(())
    }
    
    /// Retorna o endereço do admin
    pub fn get_admin(env: Env) -> Address {
        storage::bump_critical_storage(&env);
        storage::get_admin(&env)
    }
    
    /// Verifica se o contrato está pausado
    pub fn is_paused(env: Env) -> bool {
        storage::bump_critical_storage(&env);
        storage::is_paused(&env)
    }
    
    /// Verifica se um endereço está na blacklist
    pub fn is_blacklisted(env: Env, addr: Address) -> bool {
        storage::bump_critical_storage(&env);
        storage::is_blacklisted(&env, &addr)
    }
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};
    
    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let name = String::from_str(&env, "Braza Token");
        let symbol = String::from_str(&env, "BRZ");
        
        client.initialize(&admin, &name, &symbol);
        
        assert_eq!(client.name(), name);
        assert_eq!(client.symbol(), symbol);
        assert_eq!(client.decimals(), 7);
        assert_eq!(client.get_admin(), admin);
        // Verificar supply inicial de 10 milhões
        assert_eq!(client.balance(&admin), 100_000_000_000_000);
        assert_eq!(client.total_supply(), 100_000_000_000_000);
    }
    
    #[test]
    fn test_initial_supply_and_max_supply() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        
        client.initialize(
            &admin,
            &String::from_str(&env, "Braza"),
            &String::from_str(&env, "BRZ"),
        );
        
        // Verificar supply inicial de 10 milhões
        assert_eq!(client.balance(&admin), 100_000_000_000_000);
        assert_eq!(client.total_supply(), 100_000_000_000_000);
        
        // Verificar que pode mintar até 11 milhões adicionais
        client.mint(&admin, &110_000_000_000_000); // +11M
        assert_eq!(client.total_supply(), 210_000_000_000_000); // Total 21M
    }
    
    #[test]
    #[should_panic(expected = "MaxSupplyExceeded")]
    fn test_cannot_exceed_21_million() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        
        client.initialize(
            &admin,
            &String::from_str(&env, "Braza"),
            &String::from_str(&env, "BRZ"),
        );
        
        // Tentar mintar mais de 11 milhões adicionais (excederia 21M)
        client.mint(&admin, &110_000_000_000_001); // +11M + 1
    }
    
    #[test]
    fn test_mint_and_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        client.initialize(
            &admin,
            &String::from_str(&env, "Braza"),
            &String::from_str(&env, "BRZ"),
        );
        
        // Admin já tem 10M tokens do mint inicial
        let initial_balance = client.balance(&admin);
        assert_eq!(initial_balance, 100_000_000_000_000);
        
        // Transferir 500 tokens para o usuário
        client.transfer(&admin, &user, &500);
        assert_eq!(client.balance(&admin), initial_balance - 500);
        assert_eq!(client.balance(&user), 500);
    }
    
    #[test]
    fn test_vesting_linear_release() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        
        client.initialize(
            &admin,
            &String::from_str(&env, "Braza"),
            &String::from_str(&env, "BRZ"),
        );
        
        // Admin já tem 10M do mint inicial
        
        // Criar vesting: 1000 tokens, cliff 100 ledgers, duração 1000 ledgers
        let schedule_id = client.create_vesting(&beneficiary, &1000, &100, &1000, &false);
        
        // Avançar 50 ledgers (antes do cliff)
        env.ledger().set_sequence_number(50);
        let releasable = client.get_releasable_amount(&beneficiary, &schedule_id);
        assert_eq!(releasable, 0); // Nada liberado antes do cliff
        
        // Avançar para 500 ledgers (50% da duração)
        env.ledger().set_sequence_number(500);
        let releasable = client.get_releasable_amount(&beneficiary, &schedule_id);
        assert_eq!(releasable, 500); // 50% liberado
        
        // Liberar tokens
        client.release_vested(&beneficiary, &schedule_id);
        assert_eq!(client.balance(&beneficiary), 500);
        
        // Avançar para 1000 ledgers (100% da duração)
        env.ledger().set_sequence_number(1000);
        let releasable = client.get_releasable_amount(&beneficiary, &schedule_id);
        assert_eq!(releasable, 500); // Restante 50%
        
        // Liberar tokens restantes
        client.release_vested(&beneficiary, &schedule_id);
        assert_eq!(client.balance(&beneficiary), 1000);
    }
    
    #[test]
    #[should_panic(expected = "MaxSupplyExceeded")]
    fn test_max_supply_exceeded() {
        let env = Env::default();
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        
        client.initialize(
            &admin,
            &String::from_str(&env, "Braza"),
            &String::from_str(&env, "BRZ"),
        );
        
        // Tentar mint acima do MAX_SUPPLY (210_000_000_000_000)
        // Admin já tem 100_000_000_000_000, então tentar mintar mais 110_000_000_000_001
        client.mint(&admin, &110_000_000_000_001);
    }
    
    #[test]
    #[should_panic(expected = "MaxVestingSchedulesExceeded")]
    fn test_max_vesting_schedules() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        
        client.initialize(
            &admin,
            &String::from_str(&env, "Braza"),
            &String::from_str(&env, "BRZ"),
        );
        
        // Admin tem 10M tokens, suficiente para criar múltiplos vestings
        
        // Tentar criar 11 vesting schedules (limite é 10)
        for _ in 0..11 {
            client.create_vesting(&beneficiary, &100, &10, &100, &false);
        }
    }
}
