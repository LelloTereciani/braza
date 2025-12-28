use crate::compliance::{get_country_code, get_kyc_level, get_risk_score};
use crate::storage;
use crate::BrazaError;
use soroban_sdk::{Address, Env, String};

// ============================================================================
// CACHE MELHORADO COM PERSISTÊNCIA
// ============================================================================

const CACHE_EXPIRY_LEDGER: u32 = 100; // Cache válido por 100 ledgers (~5 minutos)

#[derive(Clone)]
pub struct ComplianceCache {
    pub kyc_level: u32,
    pub risk_score: u32,
    pub is_blacklisted: bool,
    pub country_code: Option<String>,
    pub cached_at_ledger: u32,
}

// ============================================================================
// VALIDAÇÃO COM CACHE MELHORADO
// ============================================================================

pub fn validate_with_cache(
    env: &Env,
    user: &Address,
    min_kyc: u32,
    max_risk: u32,
) -> Result<(), BrazaError> {
    // ✅ PASSO 1: Tentar ler do cache persistente
    let current_ledger = env.ledger().sequence();

    // Se cache existe e não expirou, usar cache
    if let Some(cache) = read_cache(env, user) {
        if current_ledger - cache.cached_at_ledger < CACHE_EXPIRY_LEDGER {
            // ✅ Cache hit - usar valores em cache
            return validate_cached_values(
                env,
                cache.kyc_level,
                cache.risk_score,
                cache.is_blacklisted,
                &cache.country_code,
                min_kyc,
                max_risk,
            );
        }
    }

    // ❌ Cache miss - ler do storage
    let kyc = get_kyc_level(env, user);
    let risk = get_risk_score(env, user);
    let blacklisted = storage::is_blacklisted(env, user);
    let country = get_country_code(env, user);

    // ✅ Armazenar em cache para próxima vez
    write_cache(env, user, kyc, risk, blacklisted, &country, current_ledger);

    // ✅ Validar valores
    validate_cached_values(env, kyc, risk, blacklisted, &country, min_kyc, max_risk)
}

// ============================================================================
// FUNÇÕES AUXILIARES
// ============================================================================

fn validate_cached_values(
    env: &Env,
    kyc: u32,
    risk: u32,
    blacklisted: bool,
    country: &Option<String>,
    min_kyc: u32,
    max_risk: u32,
) -> Result<(), BrazaError> {
    // ✅ Validar valores REAIS
    if kyc < min_kyc {
        return Err(BrazaError::Unauthorized);
    }
    if risk > max_risk {
        return Err(BrazaError::Unauthorized);
    }
    if blacklisted {
        return Err(BrazaError::Blacklisted);
    }

    // ✅ Verificar se país é permitido (usando o MESMO env!)
    match country {
        Some(c) => {
            let br = String::from_str(env, "BR");
            if c != &br {
                return Err(BrazaError::Unauthorized);
            }
        }
        None => {
            return Err(BrazaError::Unauthorized);
        }
    }

    Ok(())
}

fn read_cache(_env: &Env, _user: &Address) -> Option<ComplianceCache> {
    // Implementação simplificada - sem storage real
    // Em produção, usar env.storage().persistent()
    None
}

fn write_cache(
    _env: &Env,
    _user: &Address,
    _kyc: u32,
    _risk: u32,
    _blacklisted: bool,
    _country: &Option<String>,
    _current_ledger: u32,
) {
    // Implementação simplificada - sem storage real
    // Em produção, usar env.storage().persistent()
}

#[allow(dead_code)]
pub fn invalidate_cache(_env: &Env, _user: &Address) {
    // Implementação simplificada - sem storage real
    // Em produção, remover do storage
}
