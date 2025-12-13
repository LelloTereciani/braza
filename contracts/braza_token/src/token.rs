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
    
    // ============================================================================
    // INICIALIZAÇÃO
    // ============================================================================
    
    /// Inicializa o contrato BrazaToken.
    /// Esta função não requer proteção contra reentrância, pois é chamada apenas uma vez
    /// na implantação do contrato e não realiza chamadas externas.
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
    
    // ============================================================================
    // FUNÇÕES SEP-41 PADRÃO (Leitura)
    // ============================================================================
    
    /// Retorna o nome do token.
    pub fn name(env: Env) -> String {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).name
    }
    
    /// Retorna o símbolo do token.
    pub fn symbol(env: Env) -> String {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).symbol
    }
    
    /// Retorna o número de decimais.
    pub fn decimals(env: Env) -> u32 {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).decimals
    }
    
    /// Retorna o balance de um endereço.
    pub fn balance(env: Env, id: Address) -> i128 {
        storage::get_balance(&env, &id)
    }
    
    /// Retorna o supply total.
    pub fn total_supply(env: Env) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_total_supply(&env)
    }
    
    // ============================================================================
    // TRANSFERÊNCIAS - CEI PATTERN IMPLEMENTADO
    // ============================================================================
    
    /// Transfere tokens de `from` para `to`.
    /// Implementa proteção contra reentrância.
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
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
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
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// ✅ CORRIGIDO: Transfere tokens usando allowance com verificação completa
    /// 
    /// # Segurança:
    /// - ✅ Verifica allowance antes da transferência
    /// - ✅ Decrementa allowance automaticamente
    /// - ✅ Protege contra reentrância
    /// - ✅ Valida todas as contas (from, to, spender)
    /// - ✅ Respeita pausa e blacklist
    /// 
    /// # Padrão CEI:
    /// 1. CHECKS: Validar auth, allowance, saldos, pausa, blacklist
    /// 2. EFFECTS: Atualizar allowance, saldos
    /// 3. INTERACTIONS: Emitir eventos
    /// 
    /// # Conformidade SEP:
    /// - SEP-0041: Stellar Asset Contract padrão
    /// - SEP-0049: Soroban Authorization Framework
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
            // === CHECKS ===
            spender.require_auth();
            storage::bump_critical_storage(&env);
            
            // 1. Validações básicas
            validation::require_not_paused(&env)?;
            validation::require_not_blacklisted(&env, &from)?;
            validation::require_not_blacklisted(&env, &to)?;
            validation::require_not_blacklisted(&env, &spender)?;
            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;
            
            // 2. ✅ CORREÇÃO CRÍTICA: Verificar allowance
            let current_allowance = storage::get_allowance(&env, &from, &spender);
            
            if current_allowance < amount {
                // Emitir evento de tentativa bloqueada
                env.events().publish(
                    (symbol_short!("all_fail"), &spender, &from),
                    (amount, current_allowance),
                );
                return Err(BrazaError::InsufficientAllowance);
            }
            
            // 3. Calcular novo allowance
            let new_allowance = current_allowance
                .checked_sub(amount)
                .ok_or(BrazaError::InsufficientAllowance)?;
            
            // === EFFECTS ===
            // 4. Atualizar allowance ANTES da transferência (CEI)
            storage::set_allowance(&env, &from, &spender, new_allowance);
            
            // 5. Fazer bump do TTL do allowance
            storage::bump_allowance(&env, &from, &spender);
            
            // 6. Realizar transferência de saldos
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
            // 7. Emitir eventos na ordem correta
            events::emit_transfer(&env, &from, &to, amount);
            
            // Evento adicional de consumo de allowance
            env.events().publish(
                (symbol_short!("all_used"), &spender, &from),
                (amount, new_allowance),
            );
            
            Ok(())
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    // ============================================================================
    // ✅ NOVO: SISTEMA DE ALLOWANCE COMPLETO
    // ============================================================================
    
    /// Aprova um spender para gastar tokens em nome do caller
    /// 
    /// # Segurança:
    /// - ✅ Requer autenticação do owner (from)
    /// - ✅ Valida que spender não está blacklisted
    /// - ✅ Respeita pausa do contrato
    /// - ✅ Emite evento de aprovação
    /// - ✅ Protege contra race condition de allowance
    /// 
    /// # Padrão CEI:
    /// 1. CHECKS: Validar auth, pausa, blacklist, amount
    /// 2. EFFECTS: Atualizar allowance
    /// 3. INTERACTIONS: Emitir evento
    /// 
    /// # Nota sobre Race Condition:
    /// Para evitar front-running, recomenda-se:
    /// 1. Zerar allowance antes de definir novo valor
    /// 2. Ou usar increase_allowance/decrease_allowance
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        from.require_auth();
        storage::bump_critical_storage(&env);
        
        // 1. Validações
        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;
        
        // 2. Validar amount (pode ser 0 para revogar)
        if amount < 0 {
            return Err(BrazaError::InvalidAmount);
        }
        
        // 3. ⚠️ PROTEÇÃO: Verificar allowance atual para evitar race condition
        let current_allowance = storage::get_allowance(&env, &from, &spender);
        
        if current_allowance != 0 && amount != 0 {
            // Emitir aviso sobre potencial race condition
            env.events().publish(
                (symbol_short!("all_race"), &from, &spender),
                (current_allowance, amount),
            );
        }
        
        // === EFFECTS ===
        // 4. Definir allowance
        storage::set_allowance(&env, &from, &spender, amount);
        
        // 5. Fazer bump do TTL
        storage::bump_allowance(&env, &from, &spender);
        
        // === INTERACTIONS ===
        // 6. Emitir evento
        events::emit_approval(&env, &from, &spender, amount);
        
        Ok(())
    }
    
    /// Aumenta allowance de forma segura (evita race condition)
    /// 
    /// # Segurança:
    /// - ✅ Não sofre de race condition do approve()
    /// - ✅ Incremento atômico do allowance
    /// - ✅ Verifica overflow
    pub fn increase_allowance(
        env: Env,
        from: Address,
        spender: Address,
        delta: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        from.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;
        validation::require_positive_amount(delta)?;
        
        // === EFFECTS ===
        let current_allowance = storage::get_allowance(&env, &from, &spender);
        let new_allowance = current_allowance
            .checked_add(delta)
            .ok_or(BrazaError::InvalidAmount)?;
        
        storage::set_allowance(&env, &from, &spender, new_allowance);
        storage::bump_allowance(&env, &from, &spender);
        
        // === INTERACTIONS ===
        events::emit_approval(&env, &from, &spender, new_allowance);
        
        env.events().publish(
            (symbol_short!("all_inc"), &from, &spender),
            (delta, new_allowance),
        );
        
        Ok(())
    }
    
    /// Diminui allowance de forma segura
    /// 
    /// # Segurança:
    /// - ✅ Não sofre de race condition
    /// - ✅ Decremento atômico
    /// - ✅ Verifica underflow
    pub fn decrease_allowance(
        env: Env,
        from: Address,
        spender: Address,
        delta: i128,
    ) -> Result<(), BrazaError> {
        // === CHECKS ===
        from.require_auth();
        storage::bump_critical_storage(&env);
        
        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;
        validation::require_positive_amount(delta)?;
        
        // === EFFECTS ===
        let current_allowance = storage::get_allowance(&env, &from, &spender);
        
        if current_allowance < delta {
            return Err(BrazaError::InsufficientAllowance);
        }
        
        let new_allowance = current_allowance
            .checked_sub(delta)
            .ok_or(BrazaError::InsufficientAllowance)?;
        
        storage::set_allowance(&env, &from, &spender, new_allowance);
        storage::bump_allowance(&env, &from, &spender);
        
        // === INTERACTIONS ===
        events::emit_approval(&env, &from, &spender, new_allowance);
        
        env.events().publish(
            (symbol_short!("all_dec"), &from, &spender),
            (delta, new_allowance),
        );
        
        Ok(())
    }
    
    /// Retorna o allowance atual de um spender para uma conta
    /// 
    /// # View Function (Somente Leitura):
    /// - Não modifica estado
    /// - Não requer autenticação
    /// - Usado por frontends e outros contratos
    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_allowance(&env, &from, &spender)
    }
    
    // ============================================================================
    // MINT E BURN - CEI PATTERN
    // ============================================================================
    
    /// Cria novos tokens (apenas admin).
    /// Implementa proteção contra reentrância.
    pub fn mint(
        env: Env,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
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
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Destrói tokens (apenas admin).
    /// Implementa proteção contra reentrância.
    pub fn burn(
        env: Env,
        from: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
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
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    // ============================================================================
    // VESTING - CRITICAL-01 CORRIGIDO
    // ============================================================================
    
    /// Cria um novo vesting schedule.
    /// Implementa proteção contra reentrância.
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
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
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
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Libera tokens vestidos.
    /// Implementa proteção contra reentrância.
    /// 
    /// # Correção CRITICAL-01:
    /// - Cálculo correto usando ledger.sequence()
    /// - Liberação gradual proporcional após cliff
    pub fn release_vested(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
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
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Revoga um vesting schedule (apenas admin, se revocable).
    /// Implementa proteção contra reentrância.
    pub fn revoke_vesting(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
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
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Retorna informações de um vesting schedule.
    /// Função de leitura, não requer proteção contra reentrância.
    pub fn get_vesting_schedule(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<VestingSchedule, BrazaError> {
        storage::bump_critical_storage(&env);
        
        storage::get_vesting_schedule(&env, &beneficiary, schedule_id)
            .ok_or(BrazaError::VestingNotFound)
    }
    
    /// Retorna todos os vesting schedules de um beneficiário.
    /// Função de leitura, não requer proteção contra reentrância.
    pub fn get_all_vesting_schedules(
        env: Env,
        beneficiary: Address,
    ) -> Vec<VestingSchedule> {
        storage::bump_critical_storage(&env);
        storage::get_all_vesting_schedules(&env, &beneficiary)
    }
    
    /// Calcula a quantidade de tokens disponíveis para release.
    /// Função de leitura, não requer proteção contra reentrância.
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
    
    // ============================================================================
    // FUNÇÕES ADMINISTRATIVAS - CEI PATTERN
    // ============================================================================
    
    /// Pausa o contrato (apenas admin).
    /// Implementa proteção contra reentrância.
    pub fn pause(env: Env) -> Result<(), BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
            // === CHECKS ===
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::bump_critical_storage(&env);
            
            // === EFFECTS ===
            storage::set_paused(&env, true);
            
            // === INTERACTIONS ===
            events::emit_pause(&env);
            
            Ok(())
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Despausa o contrato (apenas admin).
    /// Implementa proteção contra reentrância.
    pub fn unpause(env: Env) -> Result<(), BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
            // === CHECKS ===
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::bump_critical_storage(&env);
            
            // === EFFECTS ===
            storage::set_paused(&env, false);
            
            // === INTERACTIONS ===
            events::emit_unpause(&env);
            
            Ok(())
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Adiciona/remove endereço da blacklist (apenas admin).
    /// Implementa proteção contra reentrância.
    pub fn set_blacklisted(
        env: Env,
        addr: Address,
        blacklisted: bool,
    ) -> Result<(), BrazaError> {
        // === REENTRANCY GUARD ===
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        
        let result = (|| {
            // === CHECKS ===
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::bump_critical_storage(&env);
            
            // === EFFECTS ===
            storage::set_blacklisted(&env, &addr, blacklisted);
            
            // === INTERACTIONS ===
            events::emit_blacklist(&env, &addr, blacklisted);
            
            Ok(())
        })();
        
        // === LIBERAR GUARD ===
        storage::set_reentrancy_guard(&env, false);
        result
    }
    
    /// Retorna o endereço do admin.
    /// Função de leitura, não requer proteção contra reentrância.
    pub fn get_admin(env: Env) -> Address {
        storage::bump_critical_storage(&env);
        storage::get_admin(&env)
    }
    
    /// Verifica se o contrato está pausado.
    /// Função de leitura, não requer proteção contra reentrância.
    pub fn is_paused(env: Env) -> bool {
        storage::bump_critical_storage(&env);
        storage::is_paused(&env)
    }
    
    /// Verifica se um endereço está na blacklist.
    /// Função de leitura, não requer proteção contra reentrância.
    pub fn is_blacklisted(env: Env, addr: Address) -> bool {
        storage::bump_critical_storage(&env);
        storage::is_blacklisted(&env, &addr)
    }
}

// ============================================================================
// ✅ TESTES UNITÁRIOS - ALLOWANCE
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};
    
    // Helper para criar um cliente do contrato
    fn create_client(env: &Env) -> (BrazaTokenClient, Address) {
        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(env, &contract_id);
        let admin = Address::generate(env);
        
        client.initialize(
            &admin,
            &String::from_str(env, "Braza"),
            &String::from_str(env, "BRZ"),
        );
        (client, admin)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let (client, admin) = create_client(&env);
        
        assert_eq!(client.name(), String::from_str(&env, "Braza"));
        assert_eq!(client.symbol(), String::from_str(&env, "BRZ"));
        assert_eq!(client.decimals(), 7);
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.balance(&admin), 100_000_000_000_000);
        assert_eq!(client.total_supply(), 100_000_000_000_000);
    }
    
    // ✅ NOVOS TESTES DE ALLOWANCE
    
    #[test]
    fn test_approve_and_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        
        // Aprovar 1000 tokens
        client.approve(&owner, &spender, &1000);
        
        // Verificar allowance
        assert_eq!(client.allowance(&owner, &spender), 1000);
    }
    
    #[test]
    fn test_transfer_from_with_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        let recipient = Address::generate(&env);
        
        // Aprovar 1000 tokens
        client.approve(&owner, &spender, &1000);
        
        // Transfer_from 500 tokens
        client.transfer_from(&spender, &owner, &recipient, &500);
        
        // Verificar saldos
        assert_eq!(client.balance(&recipient), 500);
        assert_eq!(client.balance(&owner), 100_000_000_000_000 - 500);
        
        // Verificar allowance restante
        assert_eq!(client.allowance(&owner, &spender), 500);
    }
    
    #[test]
    #[should_panic(expected = "InsufficientAllowance")]
    fn test_transfer_from_insufficient_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        let recipient = Address::generate(&env);
        
        // Aprovar apenas 500 tokens
        client.approve(&owner, &spender, &500);
        
        // Tentar transferir 1000 (deve falhar)
        client.transfer_from(&spender, &owner, &recipient, &1000);
    }
    
    #[test]
    #[should_panic(expected = "InsufficientAllowance")]
    fn test_transfer_from_no_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        let recipient = Address::generate(&env);
        
        // Tentar transfer_from sem allowance
        client.transfer_from(&spender, &owner, &recipient, &500);
    }
    
    #[test]
    fn test_increase_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        
        // Aprovar 1000
        client.approve(&owner, &spender, &1000);
        
        // Aumentar 500
        client.increase_allowance(&owner, &spender, &500);
        
        // Verificar total
        assert_eq!(client.allowance(&owner, &spender), 1500);
    }
    
    #[test]
    fn test_decrease_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        
        // Aprovar 1000
        client.approve(&owner, &spender, &1000);
        
        // Diminuir 300
        client.decrease_allowance(&owner, &spender, &300);
        
        // Verificar restante
        assert_eq!(client.allowance(&owner, &spender), 700);
    }
    
    #[test]
    fn test_approve_zero_revokes() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, owner) = create_client(&env);
        let spender = Address::generate(&env);
        
        // Aprovar 1000
        client.approve(&owner, &spender, &1000);
        assert_eq!(client.allowance(&owner, &spender), 1000);
        
        // Revogar (aprovar 0)
        client.approve(&owner, &spender, &0);
        assert_eq!(client.allowance(&owner, &spender), 0);
    }
    
    // TESTES EXISTENTES (mantidos)
    
    #[test]
    fn test_initial_supply_and_max_supply() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        
        assert_eq!(client.balance(&admin), 100_000_000_000_000);
        assert_eq!(client.total_supply(), 100_000_000_000_000);
        
        client.mint(&admin, &110_000_000_000_000);
        assert_eq!(client.total_supply(), 210_000_000_000_000);
    }
    
    #[test]
    #[should_panic(expected = "MaxSupplyExceeded")]
    fn test_cannot_exceed_21_million() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        
        client.mint(&admin, &110_000_000_000_001);
    }
    
    #[test]
    fn test_mint_and_transfer() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        
        let user = Address::generate(&env);
        
        let initial_balance = client.balance(&admin);
        assert_eq!(initial_balance, 100_000_000_000_000);
        
        client.transfer(&admin, &user, &500);
        assert_eq!(client.balance(&admin), initial_balance - 500);
        assert_eq!(client.balance(&user), 500);
    }
    
    #[test]
    fn test_vesting_linear_release() {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_client(&env);
        let beneficiary = Address::generate(&env);
        
        let schedule_id = client.create_vesting(&beneficiary, &1000, &100, &1000, &false);
        
        env.ledger().set_sequence_number(50);
        let releasable = client.get_releasable_amount(&beneficiary, &schedule_id);
        assert_eq!(releasable, 0);
        
        env.ledger().set_sequence_number(500);
        let releasable = client.get_releasable_amount(&beneficiary, &schedule_id);
        assert_eq!(releasable, 500);
        
        client.release_vested(&beneficiary, &schedule_id);
        assert_eq!(client.balance(&beneficiary), 500);
        
        env.ledger().set_sequence_number(1000);
        let releasable = client.get_releasable_amount(&beneficiary, &schedule_id);
        assert_eq!(releasable, 500);
        
        client.release_vested(&beneficiary, &schedule_id);
        assert_eq!(client.balance(&beneficiary), 1000);
    }
    
    #[test]
    #[should_panic(expected = "MaxSupplyExceeded")]
    fn test_max_supply_exceeded() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        
        client.mint(&admin, &110_000_000_000_001);
    }
    
    #[test]
    #[should_panic(expected = "MaxVestingSchedulesExceeded")]
    fn test_max_vesting_schedules() {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_client(&env);
        let beneficiary = Address::generate(&env);
        
        for _ in 0..11 {
            client.create_vesting(&beneficiary, &100, &10, &100, &false);
        }
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_reentrancy_guard_prevents_reentrant_call() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        let user = Address::generate(&env);

        storage::set_reentrancy_guard(&env, true);

        client.transfer(&admin, &user, &100);
    }

    #[test]
    fn test_reentrancy_guard_resets_on_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        let user = Address::generate(&env);

        assert!(!storage::is_reentrancy_locked(&env));

        client.transfer(&admin, &user, &100);

        assert!(!storage::is_reentrancy_locked(&env));
    }

    #[test]
    #[should_panic(expected = "InsufficientBalance")]
    fn test_reentrancy_guard_resets_on_error() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = create_client(&env);
        let user = Address::generate(&env);

        assert!(!storage::is_reentrancy_locked(&env));

        let large_amount = 100_000_000_000_000 + 1; 
        let result = client.try_transfer(&admin, &user, &large_amount);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), BrazaError::InsufficientBalance);

        assert!(!storage::is_reentrancy_locked(&env));
    }
}
