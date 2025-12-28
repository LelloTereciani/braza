#![cfg(test)]
#![cfg(not(tarpaulin_include))]
mod setup;
use setup::TestEnv;

#[test]
fn test_offchain_risk_integration_flow() {
    let t = TestEnv::new();
    let user = t.create_compliant_user();

    // 1. Simulação: Seu Backend consulta a API da Chainalysis/Elliptic (Off-chain)
    // Suponha que a API retornou risco 85 (Alto)
    let api_risk_score = 85;
    let risk_threshold_critical = 80; // Valor definido no seu .env (RISK_CRITICAL_MIN)

    // 2. Ação do Backend: O script detecta risco > threshold e chama o contrato
    if api_risk_score >= risk_threshold_critical {
        // O backend chama set_risk_score.
        // Nota: A lógica interna do contrato (compliance.rs) já faz o auto-blacklist se score >= 80
        t.client.set_risk_score(&t.admin, &user, &api_risk_score);
    }

    // 3. Verificação On-Chain: O contrato deve ter bloqueado o usuário automaticamente?
    assert_eq!(t.client.is_blacklisted(&user), true);

    // 4. Prova Real: Tenta mover fundos (Deve falhar)
    let res = t.client.try_transfer(&user, &t.admin, &100);
    assert!(res.is_err());
}
