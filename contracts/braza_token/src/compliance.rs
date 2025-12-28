use crate::storage;
use crate::types::BrazaError;
use crate::validation;
use soroban_sdk::{symbol_short, Address, Env, String};

// ============================================================================
// ENUMS E CONSTANTES
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum KYCLevel {
    None = 0,
    Basic = 1,
    Intermediate = 2,
    Advanced = 3,
}

const LEDGERS_PER_DAY: u32 = 17_280;

// ============================================================================
// GESTÃO DE KYC
// ============================================================================

pub fn set_kyc_level(
    env: &Env,
    admin: &Address,
    user: &Address,
    level: u32,
) -> Result<(), BrazaError> {
    // Validação de Admin
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    if level > 3 {
        return Err(BrazaError::InvalidAmount); // Ou InvalidArgs se tiver no enum
    }

    let key = (symbol_short!("kyc"), user);
    env.storage().persistent().set(&key, &level);

    // Timestamp da última atualização de KYC
    let ts_key = (symbol_short!("kyc_ts"), user);
    env.storage()
        .persistent()
        .set(&ts_key, &env.ledger().sequence());

    env.events()
        .publish((symbol_short!("kyc_set"), user), level);

    Ok(())
}

pub fn get_kyc_level(env: &Env, user: &Address) -> u32 {
    storage::bump_critical_storage(env);
    let key = (symbol_short!("kyc"), user);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn require_kyc_level(env: &Env, user: &Address, min: u32) -> Result<(), BrazaError> {
    let lvl = get_kyc_level(env, user);
    if lvl < min {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

// ============================================================================
// INVESTIDORES CREDENCIADOS
// ============================================================================

pub fn set_accredited_investor(
    env: &Env,
    admin: &Address,
    investor: &Address,
    accredited: bool,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    // Se for marcar como credenciado, exige conformidade prévia
    if accredited {
        require_kyc_level(env, investor, 3)?;
        // Aqui chamamos a versão interna para evitar dependência circular com validation
        require_country_allowed(env, investor)?;
    }

    let key = (symbol_short!("accred"), investor);
    env.storage().persistent().set(&key, &accredited);

    let ts_key = (symbol_short!("accr_ts"), investor);
    env.storage()
        .persistent()
        .set(&ts_key, &env.ledger().sequence());

    env.events()
        .publish((symbol_short!("accr_set"), investor), accredited);

    Ok(())
}

pub fn is_accredited_investor(env: &Env, addr: &Address) -> bool {
    storage::bump_critical_storage(env);
    let key = (symbol_short!("accred"), addr);
    env.storage().persistent().get(&key).unwrap_or(false)
}

// ============================================================================
// PAÍS / GEOBLOCK
// ============================================================================

pub fn set_country_code(
    env: &Env,
    admin: &Address,
    user: &Address,
    code: String,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    if code.is_empty() {
        return Err(BrazaError::InvalidAmount);
    }

    let key = (symbol_short!("country"), user);
    env.storage().persistent().set(&key, &code);

    env.events()
        .publish((symbol_short!("ctry_set"), user), code);

    Ok(())
}

pub fn get_country_code(env: &Env, user: &Address) -> Option<String> {
    storage::bump_critical_storage(env);
    let key = (symbol_short!("country"), user);
    env.storage().persistent().get(&key)
}

pub fn add_blocked_country(env: &Env, admin: &Address, code: String) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    if code.is_empty() {
        return Err(BrazaError::InvalidAmount);
    }

    let key = (symbol_short!("blk_ctry"), code.clone());
    env.storage().persistent().set(&key, &true);

    env.events().publish((symbol_short!("ctry_blk"),), code);

    Ok(())
}

pub fn remove_blocked_country(env: &Env, admin: &Address, code: String) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    let key = (symbol_short!("blk_ctry"), code.clone());
    env.storage().persistent().remove(&key);

    env.events().publish((symbol_short!("ctry_unb"),), code);

    Ok(())
}

pub fn is_country_blocked(env: &Env, code: String) -> bool {
    storage::bump_critical_storage(env);
    let key = (symbol_short!("blk_ctry"), code);
    env.storage().persistent().get(&key).unwrap_or(false)
}

/// Verifica se o país do usuário é permitido.
/// CORREÇÃO: Agora falha se o usuário NÃO tiver país definido.
pub fn require_country_allowed(env: &Env, user: &Address) -> Result<(), BrazaError> {
    match get_country_code(env, user) {
        Some(code) => {
            if code.is_empty() {
                return Err(BrazaError::Unauthorized);
            }
            if is_country_blocked(env, code) {
                return Err(BrazaError::Unauthorized);
            }
            Ok(())
        }
        None => {
            // CRÍTICO: Se não tem país definido, não pode operar.
            // Isso garante "Compliance by Default".
            Err(BrazaError::Unauthorized)
        }
    }
}

// ============================================================================
// AML / RISCO
// ============================================================================

pub fn set_risk_score(
    env: &Env,
    admin: &Address,
    user: &Address,
    score: u32,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    if score > 100 {
        return Err(BrazaError::InvalidAmount);
    }

    let key = (symbol_short!("risk"), user);
    env.storage().persistent().set(&key, &score);

    let ts_key = (symbol_short!("risk_ts"), user);
    env.storage()
        .persistent()
        .set(&ts_key, &env.ledger().sequence());

    // Auto-blacklist se risco for muito alto
    if score >= 80 {
        storage::set_blacklisted(env, user, true);
    }

    env.events()
        .publish((symbol_short!("risk_set"), user), score);

    Ok(())
}

pub fn get_risk_score(env: &Env, user: &Address) -> u32 {
    storage::bump_critical_storage(env);
    let key = (symbol_short!("risk"), user);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn require_acceptable_risk(env: &Env, user: &Address, max: u32) -> Result<(), BrazaError> {
    let score = get_risk_score(env, user);
    if score > max {
        return Err(BrazaError::Unauthorized);
    }
    Ok(())
}

// ============================================================================
// LIMITES DIÁRIOS
// ============================================================================

pub fn set_daily_limit(
    env: &Env,
    admin: &Address,
    user: &Address,
    limit: i128,
) -> Result<(), BrazaError> {
    admin.require_auth();
    storage::bump_critical_storage(env);
    validation::require_admin(env, admin)?;

    if limit <= 0 {
        return Err(BrazaError::InvalidAmount);
    }

    let key = (symbol_short!("day_lim"), user);
    env.storage().persistent().set(&key, &limit);

    env.events()
        .publish((symbol_short!("lim_set"), user), limit);

    Ok(())
}

pub fn get_daily_limit(env: &Env, user: &Address) -> i128 {
    storage::bump_critical_storage(env);
    let key = (symbol_short!("day_lim"), user);
    env.storage().persistent().get(&key).unwrap_or(i128::MAX)
}

pub fn check_and_update_daily_volume(
    env: &Env,
    user: &Address,
    amount: i128,
) -> Result<(), BrazaError> {
    storage::bump_critical_storage(env);
    validation::require_positive_amount(amount)?;

    let now = env.ledger().sequence();
    let today = now / LEDGERS_PER_DAY;

    let day_key = (symbol_short!("vol_day"), user);

    // ✅ BUMP ANTES DE ACESSAR day_key
    storage::bump_critical_storage(env);
    let last_day: u32 = env.storage().persistent().get(&day_key).unwrap_or(0);

    let amt_key = (symbol_short!("vol_amt"), user);

    // ✅ BUMP ANTES DE ACESSAR amt_key
    storage::bump_critical_storage(env);
    let mut volume: i128 = if last_day == today {
        env.storage().persistent().get(&amt_key).unwrap_or(0)
    } else {
        0
    };

    volume = volume
        .checked_add(amount)
        .ok_or(BrazaError::InvalidAmount)?;

    // ✅ BUMP ANTES DE CHAMAR get_daily_limit
    storage::bump_critical_storage(env);
    let limit = get_daily_limit(env, user);

    if volume > limit {
        return Err(BrazaError::Unauthorized);
    }

    // ✅ BUMP ANTES DE FAZER SET
    storage::bump_critical_storage(env);
    env.storage().persistent().set(&day_key, &today);
    env.storage().persistent().set(&amt_key, &volume);

    Ok(())
}

pub fn get_daily_volume(env: &Env, user: &Address) -> i128 {
    storage::bump_critical_storage(env);

    let now = env.ledger().sequence();
    let today = now / LEDGERS_PER_DAY;

    let day_key = (symbol_short!("vol_day"), user);
    let last_day: u32 = env.storage().persistent().get(&day_key).unwrap_or(0);

    if last_day == today {
        let amt_key = (symbol_short!("vol_amt"), user);
        env.storage().persistent().get(&amt_key).unwrap_or(0)
    } else {
        0
    }
}

pub fn is_fully_compliant(env: &Env, user: &Address) -> bool {
    storage::bump_critical_storage(env);

    if storage::is_blacklisted(env, user) {
        return false;
    }

    if get_kyc_level(env, user) < 2 {
        return false;
    }

    // Reutiliza a lógica estrita de país
    if require_country_allowed(env, user).is_err() {
        return false;
    }

    if get_risk_score(env, user) >= 50 {
        return false;
    }

    true
}

// ============================================================================
// MÓDULO DE TESTES (APENAS EM MODO TEST)
// ============================================================================

#[cfg(any(test, feature = "testutils"))]
#[cfg(not(tarpaulin_include))]
pub mod test_utils {
    // Re-exportar funções internas APENAS para testes
    pub use super::{
        add_blocked_country, check_and_update_daily_volume, get_country_code, get_daily_limit,
        get_daily_volume, get_kyc_level, get_risk_score, is_accredited_investor,
        is_country_blocked, is_fully_compliant, remove_blocked_country, require_acceptable_risk,
        require_country_allowed, require_kyc_level, set_accredited_investor, set_country_code,
        set_daily_limit, set_kyc_level, set_risk_score,
    };
}
#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_get_daily_limit() {
        let env = Env::default();
        let contract = Address::generate(&env);
        let user = Address::generate(&env);

        // Registrar o contrato no ambiente
        env.register_contract(&contract, crate::BrazaToken);

        env.as_contract(&contract, || {
            // Default é i128::MAX
            let limit = get_daily_limit(&env, &user);
            assert_eq!(limit, i128::MAX);
        });
    }

    #[test]
    fn test_get_daily_volume() {
        let env = Env::default();
        let contract = Address::generate(&env);
        let user = Address::generate(&env);

        // Registrar o contrato no ambiente
        env.register_contract(&contract, crate::BrazaToken);

        env.as_contract(&contract, || {
            // Default é 0
            let volume = get_daily_volume(&env, &user);
            assert_eq!(volume, 0);
        });
    }

    #[test]
    fn test_is_fully_compliant_default() {
        let env = Env::default();
        let contract = Address::generate(&env);
        let user = Address::generate(&env);

        // Registrar o contrato no ambiente
        env.register_contract(&contract, crate::BrazaToken);

        env.as_contract(&contract, || {
            // Novo usuário não é compliant (sem KYC, sem país, etc)
            let compliant = is_fully_compliant(&env, &user);
            assert!(!compliant);
        });
    }
}
