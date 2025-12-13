use soroban_sdk::{contracttype, contracterror, String};

// ============================================================================
// ERROS DO CONTRATO
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BrazaError {
    /// Contrato já foi inicializado
    AlreadyInitialized = 1,
    
    /// Operação não autorizada (falta autenticação ou permissão)
    Unauthorized = 2,
    
    /// Saldo insuficiente para realizar operação
    InsufficientBalance = 3,
    
    /// Valor inválido (negativo, zero quando não permitido, overflow)
    InvalidAmount = 4,
    
    /// Contrato está pausado
    Paused = 5,
    
    /// Endereço está na blacklist
    Blacklisted = 6,
    
    /// Supply máximo de 21 milhões seria excedido
    MaxSupplyExceeded = 7,
    
    /// Vesting schedule não encontrado
    VestingNotFound = 8,
    
    /// Tokens do vesting já foram liberados
    VestingAlreadyReleased = 9,
    
    /// Período de cliff ainda não foi atingido
    CliffNotReached = 10,
    
    /// Vesting schedule não é revogável
    NotRevocable = 11,
    
    /// Limite de 10 vesting schedules por beneficiário excedido
    MaxVestingSchedulesExceeded = 12,
    
    /// Parâmetros de vesting inválidos (cliff > duration, valores negativos)
    InvalidVestingParams = 13,
    
    /// Não há tokens disponíveis para release no momento
    NoTokensToRelease = 14,
    
    /// Timelock de mint/burn ainda não expirou
    TimelockNotExpired = 15,
    
    /// Limite global de 10.000 vesting schedules excedido
    GlobalVestingLimitExceeded = 16,
    
    /// Cooldown de 2h entre criações de vesting ainda ativo
    VestingCooldownActive = 17,
    
    /// Valor de vesting abaixo do mínimo de 1 BRZ
    VestingAmountTooLow = 18,

    
    /// ✅ NOVO: Allowance insuficiente para transfer_from
    /// 
    /// # Quando ocorre:
    /// - Spender tenta gastar mais tokens do que foi aprovado
    /// - Allowance não foi definido (é 0)
    /// - Allowance foi parcialmente consumido e restante é insuficiente
    /// 
    /// # Solução:
    /// - Owner deve chamar approve() para aumentar allowance
    /// - Ou usar increase_allowance() para incrementar
    InsufficientAllowance = 19,
}

// ============================================================================
// METADADOS DO TOKEN
// ============================================================================

/// Metadados do token BRZ
/// 
/// # Conformidade SEP:
/// - SEP-0041: Stellar Asset Contract padrão
/// 
/// # Campos:
/// - `name`: Nome completo do token (ex: "Braza Token")
/// - `symbol`: Símbolo do token (ex: "BRZ")
/// - `decimals`: Número de casas decimais (fixo em 7)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

// ============================================================================
// VESTING SCHEDULE
// ============================================================================

/// Estrutura de um vesting schedule
/// 
/// # Campos:
/// - `beneficiary`: Endereço que receberá os tokens
/// - `total_amount`: Quantidade total de tokens no vesting
/// - `released_amount`: Quantidade já liberada
/// - `start_ledger`: Ledger de início do vesting
/// - `cliff_ledgers`: Período de cliff (em ledgers)
/// - `duration_ledgers`: Duração total do vesting (em ledgers)
/// - `revocable`: Se o vesting pode ser revogado pelo admin
/// - `revoked`: Se o vesting foi revogado
/// 
/// # Cálculo de Liberação:
/// - Antes do cliff: 0 tokens disponíveis
/// - Após o cliff: Liberação linear proporcional ao tempo
/// - Após duration: 100% dos tokens disponíveis
/// 
/// # Exemplo:
/// ```
/// total_amount = 1000 BRZ
/// cliff_ledgers = 100
/// duration_ledgers = 1000
/// 
/// Ledger 50:  0 BRZ disponíveis (antes do cliff)
/// Ledger 100: 100 BRZ disponíveis (cliff atingido)
/// Ledger 500: 500 BRZ disponíveis (50% do tempo)
/// Ledger 1000: 1000 BRZ disponíveis (100% completo)
/// ```
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub beneficiary: soroban_sdk::Address,
    pub total_amount: i128,
    pub released_amount: i128,
    pub start_ledger: u32,
    pub cliff_ledgers: u32,
    pub duration_ledgers: u32,
    pub revocable: bool,
    pub revoked: bool,
}

// ============================================================================
// ✅ TESTES UNITÁRIOS - TIPOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_ordering() {
        // Verificar que erros estão em ordem crescente
        assert!(BrazaError::AlreadyInitialized < BrazaError::Unauthorized);
        assert!(BrazaError::Unauthorized < BrazaError::InsufficientBalance);
        assert!(BrazaError::VestingAmountTooLow < BrazaError::InsufficientAllowance);
    }
    
    #[test]
    fn test_error_values() {
        // Verificar valores específicos dos erros
        assert_eq!(BrazaError::AlreadyInitialized as u32, 1);
        assert_eq!(BrazaError::InsufficientBalance as u32, 3);
        assert_eq!(BrazaError::InsufficientAllowance as u32, 19);
    }
    
    #[test]
    fn test_error_equality() {
        let error1 = BrazaError::InsufficientAllowance;
        let error2 = BrazaError::InsufficientAllowance;
        let error3 = BrazaError::InsufficientBalance;
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
    
    #[test]
    fn test_error_clone() {
        let error = BrazaError::InsufficientAllowance;
        let cloned = error.clone();
        
        assert_eq!(error, cloned);
    }
    
    #[test]
    fn test_vesting_schedule_clone() {
        use soroban_sdk::{Env, Address};
        
        let env = Env::default();
        let beneficiary = Address::generate(&env);
        
        let schedule = VestingSchedule {
            beneficiary: beneficiary.clone(),
            total_amount: 1000,
            released_amount: 0,
            start_ledger: 0,
            cliff_ledgers: 100,
            duration_ledgers: 1000,
            revocable: true,
            revoked: false,
        };
        
        let cloned = schedule.clone();
        assert_eq!(schedule, cloned);
    }
}
