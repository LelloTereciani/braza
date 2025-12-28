#![no_std]

// ============================================================================
// MÓDULOS DO SISTEMA
// ============================================================================

mod admin; // Lógica administrativa (Mint/Burn/Timelock)
pub mod compliance; // Regras de KYC, GeoBlock e Risco
mod compliance_cache; // Cache de compliance com lazy validation
pub mod events; // Emissão de eventos padronizados
pub mod storage; // Persistência de dados
pub mod token; // Contrato Principal (BrazaToken)
pub mod types; // Structs e Erros (BrazaError, VestingSchedule)
pub mod validation; // Validações centrais (CEI)
pub mod vesting; // Lógica de Vesting

// ============================================================================
// RE-EXPORTAÇÕES PÚBLICAS
// ============================================================================

// Exporta os tipos para facilitar o uso em testes e front-end
pub use types::*;

// Exporta o contrato real definido em src/token.rs
pub use token::BrazaToken;

// O SDK gera o Client automaticamente baseado no #[contractimpl] do token.rs.
// Nós o re-exportamos aqui para ficar acessível na raiz do crate.
#[cfg(any(test, feature = "testutils"))]
pub use token::BrazaTokenClient;

// ============================================================================
// COMPATIBILIDADE
// ============================================================================

// Alias para garantir que testes antigos que usam "BrazaTokenContract" continuem funcionando.
// Agora, "BrazaTokenContract" é apenas um apelido para o verdadeiro "BrazaToken".
pub type BrazaTokenContract = BrazaToken;
