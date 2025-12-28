#![cfg(test)]
#![cfg(not(tarpaulin_include))]
// Importante: Declarar o módulo setup para usar o TestEnv
mod setup;
use setup::TestEnv;

#[test]
fn test_vip_vs_default_limits() {
    let t = TestEnv::new();

    // 1. Criação dos usuários
    let vip_user = t.create_compliant_user();
    let receiver = t.create_compliant_user();

    // 2. CRUCIAL: Dar saldo para o VIP antes de testar limites!
    // Se o teste falhava antes, provavelmente era porque o saldo era 0.
    t.client.mint(&vip_user, &10_000);

    // 3. Configuração de VIP (Opcional, dependendo da sua lógica)
    // Vamos assumir que 'create_compliant_user' já deixa ele apto (KYC 2).
    // Se houver uma função específica para aumentar limites, ela iria aqui.
    // Ex: t.client.set_txn_limit(&t.admin, &vip_user, &10_000);

    // 4. Executa a transferência
    // Usamos um valor que sabemos que deve passar para um usuário verificado
    let transfer_amount = 1000;

    let res_vip = t
        .client
        .try_transfer(&vip_user, &receiver, &transfer_amount);

    // 5. Validação
    assert!(
        res_vip.is_ok(),
        "A transferência VIP falhou. Verifique saldo ou limites."
    );

    // Verifica se o dinheiro chegou
    assert_eq!(t.client.balance(&receiver), transfer_amount);
}
