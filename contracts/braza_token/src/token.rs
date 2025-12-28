#![allow(unused_imports)]
use crate::compliance;
use crate::compliance_cache;
use crate::events;
use crate::storage;
use crate::types::{BrazaError, TokenMetadata, VestingSchedule};
use crate::validation;
use crate::vesting;
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, String as SorobanString,
    Vec,
};

// ============================================================================
// CONTRATO PRINCIPAL - BRAZA TOKEN
// ============================================================================

#[contract]
pub struct BrazaToken;
#[allow(dead_code)]
struct ReentrancyGuard<'a> {
    env: &'a Env,
}
#[allow(dead_code)]
impl<'a> ReentrancyGuard<'a> {
    fn new(env: &'a Env) -> Result<Self, BrazaError> {
        if storage::is_reentrancy_locked(env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(env, true);
        Ok(Self { env })
    }
}

impl<'a> Drop for ReentrancyGuard<'a> {
    fn drop(&mut self) {
        storage::set_reentrancy_guard(self.env, false);
    }
}

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
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(BrazaError::AlreadyInitialized);
        }

        storage::set_admin(&env, &admin);
        storage::set_paused(&env, false);

        storage::set_balance(&env, &admin, storage::INITIAL_SUPPLY);
        storage::set_total_supply(&env, storage::INITIAL_SUPPLY);

        let metadata = TokenMetadata {
            name,
            symbol,
            decimals: 7,
        };
        storage::set_metadata(&env, &metadata);

        // ============================================================================
        // ADICIONAR: Setar país, KYC e risk score DIRETAMENTE NO STORAGE
        // (Não usar compliance:: functions que exigem require_auth)
        // ============================================================================
        let br_code = String::from_str(&env, "BR");
        let key_country = (symbol_short!("ctry"), &admin);
        env.storage().persistent().set(&key_country, &br_code);

        let key_kyc = (symbol_short!("kyc"), &admin);
        env.storage().persistent().set(&key_kyc, &3u32);

        let key_risk = (symbol_short!("risk"), &admin);
        env.storage().persistent().set(&key_risk, &0u32);

        events::emit_mint(&env, &admin, storage::INITIAL_SUPPLY);

        Ok(())
    }

    // ============================================================================
    // UPGRADE
    // ============================================================================

    pub fn update_code(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), BrazaError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::bump_critical_storage(&env);
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    // ============================================================================
    // SEP‑41 — LEITURA
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
    // TRANSFERÊNCIAS
    // ============================================================================

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let result = (|| {
            from.require_auth();

            storage::bump_critical_storage(&env);

            // ✅ VALIDAÇÃO SEP-41: Se from == to
            if from == to {
                validation::require_not_paused(&env)?;
                validation::require_positive_amount(amount)?;
                validation::require_sufficient_balance(&env, &from, amount)?;
                events::emit_transfer(&env, &from, &to, amount);
                return Ok(());
            }

            // ✅ VALIDAÇÕES BÁSICAS
            validation::require_not_paused(&env)?;
            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;

            // ✅ VALIDAÇÕES COM CACHE CORRETO
            compliance_cache::validate_with_cache(&env, &from, 2, 50)?;
            compliance_cache::validate_with_cache(&env, &to, 2, 50)?;

            // ✅ Validação de daily limit
            validation::require_daily_volume_limit(&env, &from, amount)?;

            // ✅ BUMP #2: Antes de MODIFICAR balances
            storage::bump_critical_storage(&env);

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

            // ✅ BUMP #3: Antes de MODIFICAR daily volume
            storage::bump_critical_storage(&env);

            compliance::check_and_update_daily_volume(&env, &from, amount)?;

            // ✅ Invalidar cache após modificação
            compliance_cache::invalidate_cache(&env, &from);

            events::emit_transfer(&env, &from, &to, amount);

            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let result = (|| {
            spender.require_auth();
            storage::bump_critical_storage(&env);

            validation::require_not_paused(&env)?;
            validation::require_not_blacklisted(&env, &spender)?;
            validation::require_not_blacklisted(&env, &from)?;
            validation::require_not_blacklisted(&env, &to)?;

            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;

            validation::require_country_allowed(&env, &spender)?;
            validation::require_country_allowed(&env, &from)?;
            validation::require_country_allowed(&env, &to)?;

            validation::require_kyc_level(&env, &spender, 2)?;
            validation::require_kyc_level(&env, &from, 2)?;
            validation::require_kyc_level(&env, &to, 2)?;

            validation::require_acceptable_risk(&env, &spender, 50)?;
            validation::require_acceptable_risk(&env, &from, 50)?;
            validation::require_acceptable_risk(&env, &to, 50)?;

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
    // ALLOWANCE
    // ============================================================================

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        from.require_auth();
        storage::bump_critical_storage(&env);

        validation::require_not_paused(&env)?;
        validation::require_not_blacklisted(&env, &from)?;
        validation::require_not_blacklisted(&env, &spender)?;

        if amount < 0 {
            return Err(BrazaError::InvalidAmount);
        }

        let current_allowance = storage::get_allowance(&env, &from, &spender);

        if current_allowance != 0 && amount != 0 {
            env.events().publish(
                (symbol_short!("all_race"), &from, &spender),
                (current_allowance, amount),
            );
        }

        storage::set_allowance(&env, &from, &spender, amount);
        storage::bump_allowance(&env, &from, &spender);
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

        Ok(())
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        storage::bump_critical_storage(&env);
        storage::get_allowance(&env, &from, &spender)
    }

    // ============================================================================
    // MINT (Admin)
    // ============================================================================

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), BrazaError> {
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
            validation::require_max_supply_not_exceeded(&env, amount)?;

            validation::require_not_blacklisted(&env, &to)?;
            validation::require_country_allowed(&env, &to)?;
            validation::require_kyc_level(&env, &to, 2)?;
            validation::require_acceptable_risk(&env, &to, 50)?;

            let bal = storage::get_balance(&env, &to);
            let new_bal = bal.checked_add(amount).ok_or(BrazaError::OverflowError)?;

            let supply = storage::get_total_supply(&env);
            let new_supply = supply
                .checked_add(amount)
                .ok_or(BrazaError::MaxSupplyExceeded)?;

            storage::set_balance(&env, &to, new_bal);
            storage::set_total_supply(&env, new_supply);

            events::emit_mint(&env, &to, amount);
            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

    // ============================================================================
    // BURN (User)
    // ============================================================================

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let result = (|| {
            from.require_auth();

            storage::bump_critical_storage(&env);
            validation::require_not_paused(&env)?;
            validation::require_positive_amount(amount)?;
            validation::require_sufficient_balance(&env, &from, amount)?;

            validation::require_not_blacklisted(&env, &from)?;
            storage::validate_burn_not_locked(&env, amount)?;

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

            events::emit_burn(&env, &from, amount);
            Ok(())
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

    // ============================================================================
    // VESTING
    // ============================================================================

    pub fn create_vesting(
        env: Env,
        beneficiary: Address,
        total_amount: i128,
        cliff_ledgers: u32,
        duration_ledgers: u32,
        revocable: bool,
    ) -> Result<u32, BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);

        let result = (|| {
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::bump_critical_storage(&env);

            validation::require_not_paused(&env)?;
            validation::require_valid_vesting_params(
                total_amount,
                cliff_ledgers,
                duration_ledgers,
            )?;
            validation::require_sufficient_balance(&env, &admin, total_amount)?;

            let admin_balance = storage::get_balance(&env, &admin);
            let new_admin_balance = admin_balance
                .checked_sub(total_amount)
                .ok_or(BrazaError::InsufficientBalance)?;

            storage::set_balance(&env, &admin, new_admin_balance);
            storage::increment_locked_balance(&env, total_amount)?;

            let schedule_id = vesting::create_vesting_schedule(
                &env,
                &beneficiary,
                total_amount,
                cliff_ledgers,
                duration_ledgers,
                revocable,
            )?;

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

            let releasable = vesting::release_vested_tokens(&env, &beneficiary, schedule_id)?;

            let bal = storage::get_balance(&env, &beneficiary);
            let new_bal = bal
                .checked_add(releasable)
                .ok_or(BrazaError::InvalidAmount)?;

            storage::set_balance(&env, &beneficiary, new_bal);
            storage::decrement_locked_balance(&env, releasable)?;

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

            let unvested = vesting::revoke_vesting_schedule(&env, &beneficiary, schedule_id)?;

            let adm_bal = storage::get_balance(&env, &admin);
            let new_bal = adm_bal
                .checked_add(unvested)
                .ok_or(BrazaError::InvalidAmount)?;

            storage::set_balance(&env, &admin, new_bal);
            storage::decrement_locked_balance(&env, unvested)?;

            events::emit_vesting_revoked(&env, &beneficiary, schedule_id);
            Ok(unvested)
        })();

        storage::set_reentrancy_guard(&env, false);
        result
    }

    // ============================================================================
    // GETTERS & ADMIN
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

    pub fn get_all_vesting_schedules(env: Env, beneficiary: Address) -> Vec<VestingSchedule> {
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

    pub fn pause(env: Env) -> Result<(), BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        // ANTES: let res = (|| { ... })();
        // DEPOIS:
        let res = {
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::set_paused(&env, true);
            events::emit_pause(&env);
            Ok(())
        };
        storage::set_reentrancy_guard(&env, false);
        res
    }

    pub fn unpause(env: Env) -> Result<(), BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        // DEPOIS:
        let res = {
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::set_paused(&env, false);
            events::emit_unpause(&env);
            Ok(())
        };
        storage::set_reentrancy_guard(&env, false);
        res
    }

    pub fn set_blacklisted(env: Env, addr: Address, blacklisted: bool) -> Result<(), BrazaError> {
        if storage::is_reentrancy_locked(&env) {
            return Err(BrazaError::Unauthorized);
        }
        storage::set_reentrancy_guard(&env, true);
        // DEPOIS:
        let res = {
            let admin = storage::get_admin(&env);
            admin.require_auth();
            storage::set_blacklisted(&env, &addr, blacklisted);
            events::emit_blacklist(&env, &addr, blacklisted);
            Ok(())
        };
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

    // ============================================================================
    // GESTÃO DE ADMINISTRAÇÃO & GOD MODE
    // ============================================================================

    pub fn transfer_ownership(env: Env, new_admin: Address) -> Result<(), BrazaError> {
        let current_admin = storage::get_admin(&env);
        current_admin.require_auth();
        storage::bump_critical_storage(&env);

        env.events()
            .publish((symbol_short!("adm_chg"), current_admin), new_admin.clone());

        storage::set_admin(&env, &new_admin);
        Ok(())
    }

    pub fn recover_tokens(
        env: Env,
        token_address: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        use crate::admin;
        admin::recover_tokens(&env, token_address, to, amount)
    }

    pub fn force_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), BrazaError> {
        use crate::admin;
        admin::force_transfer(&env, &from, &to, amount)
    }

    pub fn force_burn(env: Env, from: Address, amount: i128) -> Result<(), BrazaError> {
        use crate::admin;
        admin::force_burn(&env, &from, amount)
    }

    // ============================================================================
    // GESTÃO DE COMPLIANCE (Expondo para o Client)
    // ============================================================================

    pub fn set_kyc_level(
        env: Env,
        admin: Address,
        user: Address,
        level: u32,
    ) -> Result<(), BrazaError> {
        // ✅ ADICIONAR ESTA LINHA
        validation::validate_kyc_level_value(level)?;

        use crate::compliance;
        compliance::set_kyc_level(&env, &admin, &user, level)
    }

    pub fn set_country_code(
        env: Env,
        admin: Address,
        user: Address,
        code: String,
    ) -> Result<(), BrazaError> {
        use crate::compliance;
        compliance::set_country_code(&env, &admin, &user, code)
    }

    pub fn add_blocked_country(env: Env, admin: Address, code: String) -> Result<(), BrazaError> {
        use crate::compliance;
        compliance::add_blocked_country(&env, &admin, code)
    }

    pub fn set_risk_score(
        env: Env,
        admin: Address,
        user: Address,
        score: u32,
    ) -> Result<(), BrazaError> {
        use crate::compliance;
        compliance::set_risk_score(&env, &admin, &user, score)
    }

    pub fn set_daily_limit(
        env: Env,
        admin: Address,
        user: Address,
        limit: i128,
    ) -> Result<(), BrazaError> {
        use crate::compliance;
        compliance::set_daily_limit(&env, &admin, &user, limit)
    }

    // ✅ Função helper para bumpar storage (apenas para testes)
    // ✅ Função de contrato para bumpar storage (apenas para testes)
}

#[contractimpl]
impl BrazaToken {
    pub fn bump_storage_for_user(env: Env, user: Address) -> Result<(), BrazaError> {
        // ✅ BUMP #1: Geral
        storage::bump_critical_storage(&env);

        // ✅ BUMP #2: Antes de acessar balance
        storage::bump_critical_storage(&env);
        let balance_key = (symbol_short!("balance"), &user);
        if let Some(balance) = env.storage().persistent().get::<_, i128>(&balance_key) {
            storage::bump_critical_storage(&env);
            env.storage().persistent().set(&balance_key, &balance);
        }

        // ✅ BUMP #3: Antes de acessar vol_day
        storage::bump_critical_storage(&env);
        let vol_day_key = (symbol_short!("vol_day"), &user);
        if let Some(vol_day) = env.storage().persistent().get::<_, u32>(&vol_day_key) {
            storage::bump_critical_storage(&env);
            env.storage().persistent().set(&vol_day_key, &vol_day);
        }

        // ✅ BUMP #4: Antes de acessar vol_amt
        storage::bump_critical_storage(&env);
        let vol_amt_key = (symbol_short!("vol_amt"), &user);
        if let Some(vol_amt) = env.storage().persistent().get::<_, i128>(&vol_amt_key) {
            storage::bump_critical_storage(&env);
            env.storage().persistent().set(&vol_amt_key, &vol_amt);
        }

        // ✅ BUMP #5: Antes de acessar daily_lim
        storage::bump_critical_storage(&env);
        let daily_lim_key = (symbol_short!("daily_lim"), &user);
        if let Some(daily_lim) = env.storage().persistent().get::<_, i128>(&daily_lim_key) {
            storage::bump_critical_storage(&env);
            env.storage().persistent().set(&daily_lim_key, &daily_lim);
        }

        Ok(())
    }
}
