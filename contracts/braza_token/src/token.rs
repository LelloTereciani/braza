#![allow(unused_imports)]
use soroban_sdk::{
    contract, contractimpl, Address, Env, String, Vec, symbol_short
};

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

    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
    ) -> Result<(), BrazaError> {

        // Checar se já inicializado
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(BrazaError::AlreadyInitialized);
        }

        // Setar admin e estado inicial
        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);

        // Mint inicial de 10M BRZ
        storage::set_balance(&env, &admin, storage::INITIAL_SUPPLY);
        storage::set_total_supply(&env, storage::INITIAL_SUPPLY);

        // Criar metadata
        let metadata = TokenMetadata {
            name,
            symbol,
            decimals: 7,
        };
        storage::set_metadata(&env, &metadata);

        // Emitir evento de mint
        events::emit_mint(&env, &admin, storage::INITIAL_SUPPLY);

        Ok(())
    }

    // ============================================================================
    // SEP‑41 — Funções de leitura
    // ============================================================================

    pub fn name(env: Env) -> String {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).name
    }

    pub fn symbol(env: Env) -> String {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).symbol
    }

    pub fn decimals(env: Env) -> u32 {
        storage::bump_critical_storage(&env);
        storage::get_metadata(&env).decimals
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        storage::get_balance(&env, &id)
    }

    pub fn total_supply(env: Env) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_total_supply(&env)
    }

        // ============================================================================
    // TRANSFERÊNCIAS — CEI + COMPLIANCE + ANTI‑REENTRÂNCIA
    // ============================================================================

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

            // Pausa / blacklist
            validation::require_not_paused(&env)?;
            validation::require_not_blacklisted(&env, &from)?;
            validation::require_not_blacklisted(&env, &to)?;

            // Valores
            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;

            // === COMPLIANCE (ADICIONADO) ===
            validation::require_country_allowed(&env, &from)?;
            validation::require_country_allowed(&env, &to)?;

            validation::require_kyc_level(&env, &from, 1)?;
            validation::require_kyc_level(&env, &to, 1)?;

            validation::require_acceptable_risk(&env, &from, 50)?;
            validation::require_acceptable_risk(&env, &to, 50)?;

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

        storage::set_reentrancy_guard(&env, false);
        result
    }

    // ============================================================================
    // TRANSFER_FROM — CORRIGIDO COM COMPLIANCE, CEI E ANTI‑REENTRÂNCIA
    // ============================================================================

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

            validation::require_not_paused(&env)?;
            validation::require_not_blacklisted(&env, &spender)?;
            validation::require_not_blacklisted(&env, &from)?;
            validation::require_not_blacklisted(&env, &to)?;

            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;

            // === COMPLIANCE (ADICIONADO) ===
            validation::require_country_allowed(&env, &spender)?;
            validation::require_country_allowed(&env, &from)?;
            validation::require_country_allowed(&env, &to)?;

            validation::require_kyc_level(&env, &spender, 1)?;
            validation::require_kyc_level(&env, &from, 1)?;
            validation::require_kyc_level(&env, &to, 1)?;

            validation::require_acceptable_risk(&env, &spender, 50)?;
            validation::require_acceptable_risk(&env, &from, 50)?;
            validation::require_acceptable_risk(&env, &to, 50)?;

            // === ALLOWANCE ===
            let current_allowance = storage::get_allowance(&env, &from, &spender);

            if current_allowance < amount {
                env.events().publish(
                    (symbol_short!("all_fail"), &spender, &from),
                    (amount, current_allowance),
                );
                return Err(BrazaError::InsufficientAllowance);
            }

            let new_allowance = current_allowance
                .checked_sub(amount)
                .ok_or(BrazaError::InsufficientAllowance)?;

            storage::set_allowance(&env, &from, &spender, new_allowance);
            storage::bump_allowance(&env, &from, &spender);

            // === EFFECTS — Atualizar saldos ===
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

            env.events().publish(
                (symbol_short!("all_used"), &spender, &from),
                (amount, new_allowance),
            );

            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

        // ============================================================================
    // ALLOWANCE — SISTEMA COMPLETO CORRIGIDO
    // ============================================================================

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {

        from.require_auth();
        storage::bump_critical_storage(&env);

        // Pausa / blacklist
        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;

        // amount pode ser 0 (revogar), mas nunca negativo
        if amount < 0 {
            return Err(BrazaError::InvalidAmount);
        }

        let current_allowance = storage::get_allowance(&env, &from, &spender);

        // Aviso opcional sobre race condition
        if current_allowance != 0 && amount != 0 {
            env.events().publish(
                (symbol_short!("all_race"), &from, &spender),
                (current_allowance, amount),
            );
        }

        // Effects
        storage::set_allowance(&env, &from, &spender, amount);
        storage::bump_allowance(&env, &from, &spender);

        // Interactions
        events::emit_approval(&env, &from, &spender, amount);

        Ok(())
    }

    pub fn increase_allowance(
        env: Env,
        from: Address,
        spender: Address,
        delta: i128,
    ) -> Result<(), BrazaError> {

        from.require_auth();
        storage::bump_critical_storage(&env);

        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;
        validation::require_positive_amount(delta)?;

        let current = storage::get_allowance(&env, &from, &spender);

        let new = current
            .checked_add(delta)
            .ok_or(BrazaError::InvalidAmount)?;

        storage::set_allowance(&env, &from, &spender, new);
        storage::bump_allowance(&env, &from, &spender);

        events::emit_approval(&env, &from, &spender, new);

        env.events().publish(
            (symbol_short!("all_inc"), &from, &spender),
            (delta, new),
        );

        Ok(())
    }

    pub fn decrease_allowance(
        env: Env,
        from: Address,
        spender: Address,
        delta: i128,
    ) -> Result<(), BrazaError> {

        from.require_auth();
        storage::bump_critical_storage(&env);

        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;
        validation::require_positive_amount(delta)?;

        let current = storage::get_allowance(&env, &from, &spender);

        if current < delta {
            return Err(BrazaError::InsufficientAllowance);
        }

        let new = current
            .checked_sub(delta)
            .ok_or(BrazaError::InsufficientAllowance)?;

        storage::set_allowance(&env, &from, &spender, new);
        storage::bump_allowance(&env, &from, &spender);

        events::emit_approval(&env, &from, &spender, new);

        env.events().publish(
            (symbol_short!("all_dec"), &from, &spender),
            (delta, new),
        );

        Ok(())
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_allowance(&env, &from, &spender)
    }

    // ============================================================================
    // MINT — Corrigido com proteção TOTAL
    // ============================================================================

    pub fn mint(
        env: Env,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {

        // Reentrância
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let result = (|| {

            // CHECKS
            let admin = storage::get_admin(&env);
            admin.require_auth();

            storage::bump_critical_storage(&env);

            validation::require_not_paused(&env)?;
            validation::require_positive_amount(amount)?;

            // Supply máximo de 21M
            validation::require_max_supply_not_exceeded(&env, amount)?;

            // Compliance — destinatário
            validation::require_not_blacklisted(&env, &to)?;
            validation::require_country_allowed(&env, &to)?;
            validation::require_kyc_level(&env, &to, 1)?;
            validation::require_acceptable_risk(&env, &to, 50)?;

            // EFFECTS — atualizar saldos
            let bal = storage::get_balance(&env, &to);
            let new_bal = bal
                .checked_add(amount)
                .ok_or(BrazaError::InvalidAmount)?;

            let supply = storage::get_total_supply(&env);
            let new_supply = supply
                .checked_add(amount)
                .ok_or(BrazaError::MaxSupplyExceeded)?;

            storage::set_balance(&env, &to, new_bal);
            storage::set_total_supply(&env, new_supply);

            // INTERACTIONS
            events::emit_mint(&env, &to, amount);

            Ok(())

        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

    // ============================================================================
    // BURN — Corrigido com proteção contra queima de tokens locked
    // ============================================================================

    pub fn burn(
        env: Env,
        from: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {

        // Reentrância
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let result = (|| {

            let admin = storage::get_admin(&env);
            admin.require_auth();

            storage::bump_critical_storage(&env);

            validation::require_not_paused(&env)?;
            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;

            // CRÍTICO — não pode queimar tokens locked em vesting
            storage::validate_burn_not_locked(&env, amount)?;

            // EFFECTS
            let bal = storage::get_balance(&env, &from);
            let new_bal = bal
                .checked_sub(amount)
                .ok_or(BrazaError::InsufficientBalance)?;

            let supply = storage::get_total_supply(&env);
            let new_supply = supply
                .checked_sub(amount)
                .ok_or(BrazaError::InvalidAmount)?;

            storage::set_balance(&env, &from, new_bal);
            storage::set_total_supply(&env, new_supply);

            // INTERACTIONS
            events::emit_burn(&env, &from, amount);

            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

        // ============================================================================
    // VESTING – CREATE / RELEASE / REVOKE (CORRIGIDO)
    // ============================================================================

    pub fn create_vesting(
        env: Env,
        beneficiary: Address,
        total_amount: i128,
        cliff_ledgers: u32,
        duration_ledgers: u32,
        revocable: bool,
    ) -> Result<u32, BrazaError> {

        // Anti-reentrância
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }

        storage::set_reentrancy_guard(&env, true);

        let result = (|| {

            // Admin
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::bump_critical_storage(&env);

            // Regras
            validation::require_not_paused(&env)?;
            validation::require_valid_vesting_params(
                total_amount,
                cliff_ledgers,
                duration_ledgers
            )?;
            validation::require_sufficient_balance(&env, &admin, total_amount)?;

            // 1. Debitar tokens do admin (bloqueio)
            let admin_balance = storage::get_balance(&env, &admin);
            let new_admin_balance = admin_balance
                .checked_sub(total_amount)
                .ok_or(BrazaError::InsufficientBalance)?;

            storage::set_balance(&env, &admin, new_admin_balance);

            // 2. Atualizar locked balance global
            storage::increment_locked_balance(&env, total_amount)?;

            // 3. Criar vesting schedule
            let schedule_id = vesting::create_vesting_schedule(
                &env,
                &beneficiary,
                total_amount,
                cliff_ledgers,
                duration_ledgers,
                revocable,
            )?;

            // 4. Evento
            events::emit_vesting_created(&env, &beneficiary, schedule_id, total_amount);

            Ok(schedule_id)
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }


    pub fn release_vested(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {

        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }

        storage::set_reentrancy_guard(&env, true);

        let result = (|| {

            beneficiary.require_auth();
            storage::bump_critical_storage(&env);

            validation::require_not_paused(&env)?;

            // 1. Calcular valor liberado
            let releasable = vesting::release_vested_tokens(
                &env,
                &beneficiary,
                schedule_id
            )?;

            // 2. Transferir para beneficiário
            let bal = storage::get_balance(&env, &beneficiary);
            let new_bal = bal.checked_add(releasable)
                .ok_or(BrazaError::InvalidAmount)?;

            storage::set_balance(&env, &beneficiary, new_bal);

            // 3. Atualizar locked balance
            storage::decrement_locked_balance(&env, releasable)?;

            // 4. Evento
            events::emit_vesting_released(&env, &beneficiary, schedule_id, releasable);

            Ok(releasable)
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }


    pub fn revoke_vesting(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<i128, BrazaError> {

        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }

        storage::set_reentrancy_guard(&env, true);

        let result = (|| {

            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::bump_critical_storage(&env);

            validation::require_not_paused(&env)?;

            // 1. Revogar schedule e obter tokens não-vestidos
            let unvested = vesting::revoke_vesting_schedule(
                &env,
                &beneficiary,
                schedule_id
            )?;

            // 2. Devolver para admin
            let adm_bal = storage::get_balance(&env, &admin);
            let new_bal = adm_bal
                .checked_add(unvested)
                .ok_or(BrazaError::InvalidAmount)?;

            storage::set_balance(&env, &admin, new_bal);

            // 3. Atualizar locked balance
            storage::decrement_locked_balance(&env, unvested)?;

            // 4. Evento
            events::emit_vesting_revoked(&env, &beneficiary, schedule_id);

            Ok(unvested)
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

    // ============================================================================
    // VESTING — FUNÇÕES DE CONSULTA
    // ============================================================================

    pub fn get_vesting_schedule(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
    ) -> Result<VestingSchedule, BrazaError> {

        storage::bump_critical_storage(&env);

        storage::get_vesting_schedule(&env, &beneficiary, schedule_id)
            .ok_or(BrazaError::VestingNotFound)
    }

    pub fn get_all_vesting_schedules(
        env: Env,
        beneficiary: Address
    ) -> Vec<VestingSchedule> {

        storage::bump_critical_storage(&env);
        storage::get_all_vesting_schedules(&env, &beneficiary)
    }

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
    // SUPPLY STATS
    // ============================================================================

    pub fn get_locked_balance(env: Env) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_locked_balance(&env)
    }

    pub fn get_circulating_supply(env: Env) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_circulating_supply(&env)
    }

    pub fn get_supply_stats(env: Env) -> (i128, i128, i128, i128) {
        storage::bump_critical_storage(&env);

        let total = storage::get_total_supply(&env);
        let locked = storage::get_locked_balance(&env);
        let circulating = storage::get_circulating_supply(&env);
        let max = storage::MAX_SUPPLY;

        (total, locked, circulating, max)
    }

    // ============================================================================
    // ADMIN: PAUSE / UNPAUSE / BLACKLIST
    // ============================================================================

    pub fn pause(env: Env) -> Result<(), BrazaError> {

        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let res = (|| {
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::set_paused(&env, true);
            events::emit_pause(&env);
            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        res
    }


    pub fn unpause(env: Env) -> Result<(), BrazaError> {

        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }

        storage::set_reentrancy_guard(&env, true);

        let res = (|| {
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::set_paused(&env, false);
            events::emit_unpause(&env);
            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        res
    }


    pub fn set_blacklisted(
        env: Env,
        addr: Address,
        blacklisted: bool,
    ) -> Result<(), BrazaError> {

        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }

        storage::set_reentrancy_guard(&env, true);

        let res = (|| {

            let admin = storage::get_admin(&env);
            admin.require_auth();

            storage::set_blacklisted(&env, &addr, blacklisted);
            events::emit_blacklist(&env, &addr, blacklisted);

            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        res
    }

    pub fn get_admin(env: Env) -> Address {
        storage::bump_critical_storage(&env);
        storage::get_admin(&env)
    }

    pub fn is_paused(env: Env) -> bool {
        storage::bump_critical_storage(&env);
        storage::is_paused(&env)
    }

    pub fn is_blacklisted(env: Env, addr: Address) -> bool {
        storage::bump_critical_storage(&env);
        storage::is_blacklisted(&env, &addr)
    }
}


