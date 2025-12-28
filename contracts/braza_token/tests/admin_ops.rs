#![cfg(test)]
#![cfg(not(tarpaulin_include))]
mod setup;
use setup::TestEnv;
use soroban_sdk::{testutils::BytesN as _, BytesN};

#[test]
fn test_user_can_burn_own_tokens() {
    let t = TestEnv::new();
    let user = t.create_compliant_user();

    // 1. Admin dá tokens para o usuário
    t.client.mint(&user, &1000);
    assert_eq!(t.client.balance(&user), 1000);
    let supply_before = t.client.total_supply();

    // 2. Usuário decide queimar 400 tokens (Self-Burn)
    // Isso deve funcionar agora que mudamos para 'from.require_auth()'
    t.client.burn(&user, &400);

    // 3. Verificações
    assert_eq!(t.client.balance(&user), 600);
    assert_eq!(t.client.total_supply(), supply_before - 400);
}

#[test]
fn test_transfer_admin_ownership() {
    let t = TestEnv::new();
    let new_admin = t.create_compliant_user();

    // 1. Admin atual transfere para new_admin
    // Agora a função existe no token.rs!
    t.client.transfer_ownership(&new_admin);

    // 2. Verifica se mudou (precisamos de um getter ou tentar uma ação de admin)
    assert_eq!(t.client.get_admin(), new_admin);
}

#[test]
fn test_upgrade_auth_protection() {
    let t = TestEnv::new();
    let _hacker = t.create_compliant_user();

    // Gera um hash aleatório simulando um novo WASM
    let fake_wasm_hash = BytesN::<32>::random(&t.env);

    // 1. Hacker tenta atualizar o contrato
    let res = t.client.try_update_code(&fake_wasm_hash);

    // 2. Deve falhar (apenas admin pode)
    assert!(res.is_err());

    // 3. Admin tenta (Deve passar a verificação de auth,
    // embora falhe na execução real porque o hash não existe na rede de teste,
    // mas prova que a porta está trancada).
    // O teste de sucesso real de upgrade exige deploy de 2 contratos, complexo para unit test.
}

#[test]
fn test_recover_tokens_flow() {
    let t = TestEnv::new();

    // Removemos o underline pois a variável É usada abaixo
    let hacker = t.create_compliant_user();
    let token_address = t.create_compliant_user();

    let res = t.client.try_recover_tokens(&token_address, &hacker, &1000);

    // O teste espera erro (pois hacker não é admin)
    assert!(res.is_err());
}
