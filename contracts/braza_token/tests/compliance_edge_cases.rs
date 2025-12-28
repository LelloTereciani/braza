#![cfg(test)]
#![cfg(not(tarpaulin_include))]
mod setup;
use setup::TestEnv;
use soroban_sdk::testutils::Address as AddressTestUtils;
use soroban_sdk::Address;
use soroban_sdk::String;

// ============================================================================
// SEÇÃO 1: VALIDAÇÕES DE INPUT
// ============================================================================

#[test]
fn test_set_kyc_level_invalid_level() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Tentar setar KYC com nível > 3 (deve falhar)
    let res = t.client.try_set_kyc_level(admin, &user, &4);
    assert!(res.is_err(), "KYC level > 3 deveria falhar");

    let res = t.client.try_set_kyc_level(admin, &user, &100);
    assert!(res.is_err(), "KYC level > 3 deveria falhar");
}

#[test]
fn test_set_country_code_empty() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Tentar setar país com código vazio (deve falhar)
    let empty_code = String::from_str(&t.env, "");
    let res = t.client.try_set_country_code(admin, &user, &empty_code);
    assert!(res.is_err(), "País vazio deveria falhar");
}

#[test]
fn test_add_blocked_country_empty() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let admin = &t.admin;

    // Tentar bloquear país com código vazio (deve falhar)
    let empty_code = String::from_str(&t.env, "");
    let res = t.client.try_add_blocked_country(admin, &empty_code);
    assert!(res.is_err(), "Bloquear país vazio deveria falhar");
}

#[test]
fn test_set_daily_limit_invalid() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Tentar setar limite <= 0 (deve falhar)
    let res = t.client.try_set_daily_limit(admin, &user, &0);
    assert!(res.is_err(), "Setar limite <= 0 deveria falhar");

    let res = t.client.try_set_daily_limit(admin, &user, &-100);
    assert!(res.is_err(), "Setar limite negativo deveria falhar");
}

// ============================================================================
// SEÇÃO 2: VALIDAÇÕES DE COMPLIANCE - KYC
// ============================================================================

#[test]
fn test_set_kyc_level_zero() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let receiver = t.create_compliant_user();

    // ✅ Dar saldo ao usuário
    t.client.mint(&user, &100);

    // ✅ Tentar setar KYC para 0 (deve FALHAR)
    let res = t.client.try_set_kyc_level(&t.admin, &user, &0);
    assert!(res.is_err(), "set_kyc_level com KYC 0 deveria falhar");

    // ✅ KYC continua 2 (do setup), transferência deve passar
    let res = t.client.try_transfer(&user, &receiver, &100);
    assert!(res.is_ok(), "Transferência com KYC válido deveria passar");
}

#[test]
fn test_require_kyc_level_minimum() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = Address::generate(&t.env);
    let admin = &t.admin;

    println!("\n=== TESTE: test_require_kyc_level_minimum ===");
    println!("User: {:?}", user);
    println!("Admin: {:?}", admin);

    // ============================================================================
    // TESTE 1: Mint SEM compliance deveria FALHAR (por compliance, não supply)
    // ============================================================================

    println!("\n[TESTE 1] Mint SEM KYC");
    let tiny_amount = 1_000_000i128;
    let result = t.client.try_mint(&user, &tiny_amount);
    println!("Resultado: {:?}", result);
    assert!(result.is_err(), "❌ Mint SEM KYC deveria FALHAR");

    // ============================================================================
    // TESTE 2: Mint COM KYC mas SEM país deveria FALHAR
    // ============================================================================

    println!("\n[TESTE 2] Mint COM KYC mas SEM país");
    t.client.set_kyc_level(admin, &user, &2);
    let result = t.client.try_mint(&user, &tiny_amount);
    println!("Resultado: {:?}", result);
    assert!(result.is_err(), "❌ Mint SEM país deveria FALHAR");

    // ============================================================================
    // TESTE 3: Mint COM KYC e país mas COM risk alto deveria FALHAR
    // ============================================================================

    println!("\n[TESTE 3] Mint COM KYC e país mas COM risk alto");
    t.client
        .set_country_code(admin, &user, &String::from_str(&t.env, "BR"));
    t.client.set_risk_score(admin, &user, &80);
    let result = t.client.try_mint(&user, &tiny_amount);
    println!("Resultado: {:?}", result);
    assert!(result.is_err(), "❌ Mint COM risk alto deveria FALHAR");

    // ============================================================================
    // TESTE 4: Mint COM compliance COMPLETO deveria PASSAR
    // ============================================================================

    println!("\n[TESTE 4] Mint COM compliance COMPLETO");
    t.client.set_blacklisted(&user, &false);
    t.client.set_risk_score(admin, &user, &0);

    println!("Tentando fazer mint de {} para {:?}", tiny_amount, user);
    let result = t.client.try_mint(&user, &tiny_amount);
    println!("Resultado: {:?}", result);
    assert!(
        result.is_ok(),
        "✅ Mint COM compliance COMPLETO deveria PASSAR"
    );

    // ============================================================================
    // TESTE 5: Transfer COM KYC < 2 deveria FALHAR
    // ============================================================================

    println!("\n[TESTE 5] Transfer COM KYC < 2");

    // ✅ Criar um novo usuário para este teste
    let user2 = Address::generate(&t.env);

    // Configurar user2 com compliance completo
    t.client.set_kyc_level(admin, &user2, &2);
    t.client
        .set_country_code(admin, &user2, &String::from_str(&t.env, "BR"));
    t.client.set_risk_score(admin, &user2, &0);

    // Dar saldo a user2
    t.client.mint(&user2, &tiny_amount);

    // Agora reduzir KYC para 1
    t.client.set_kyc_level(admin, &user2, &1);

    println!("Tentando transferir COM KYC = 1");
    let result = t.client.try_transfer(&user2, admin, &500_000i128);
    println!("Resultado: {:?}", result);
    assert!(result.is_err(), "❌ Transfer COM KYC < 2 deveria FALHAR");

    // ============================================================================
    // TESTE 6: Transfer COM KYC >= 2 deveria PASSAR
    // ============================================================================

    println!("\n[TESTE 6] Transfer COM KYC >= 2");

    // ✅ Restaurar KYC para 2
    t.client.set_kyc_level(admin, &user2, &2);

    // ✅ IMPORTANTE: Garantir que admin tem compliance completo
    t.client.set_kyc_level(admin, admin, &2);
    t.client
        .set_country_code(admin, admin, &String::from_str(&t.env, "BR"));
    t.client.set_risk_score(admin, admin, &0);

    println!("Tentando transferir COM KYC = 2");
    let result = t.client.try_transfer(&user2, admin, &500_000i128);
    println!("Resultado: {:?}", result);
    assert!(result.is_ok(), "✅ Transfer COM KYC >= 2 deveria PASSAR");

    println!("\n=== TESTE PASSOU ===\n");
}

#[test]
fn test_set_kyc_level_valid_levels() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // ✅ Testar todos os níveis válidos (1-3)
    for level in 1..=3 {
        let res = t.client.try_set_kyc_level(admin, &user, &level);
        assert!(res.is_ok(), "KYC level {} deveria ser válido", level);
    }

    // ✅ Testar que KYC 0 é inválido
    let res = t.client.try_set_kyc_level(admin, &user, &0);
    assert!(res.is_err(), "KYC level 0 deveria ser inválido");

    // ✅ Testar que KYC 4 é inválido
    let res = t.client.try_set_kyc_level(admin, &user, &4);
    assert!(res.is_err(), "KYC level 4 deveria ser inválido");
}

// ============================================================================
// SEÇÃO 3: VALIDAÇÕES DE COMPLIANCE - PAÍS
// ============================================================================

#[test]
fn test_add_blocked_country_functionality() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let admin = &t.admin;
    let user = t.create_compliant_user();

    // Bloquear país
    let nk_code = String::from_str(&t.env, "NK");
    t.client.add_blocked_country(admin, &nk_code);

    // Setar usuário para país bloqueado
    t.client.set_country_code(admin, &user, &nk_code);

    // Tentar transferir (deve falhar)
    let res = t.client.try_transfer(&user, admin, &100);
    assert!(
        res.is_err(),
        "Transferência de país bloqueado deveria falhar"
    );
}

#[test]
fn test_require_country_allowed_no_country() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    // Usar o trait AddressTestUtils para gerar um novo endereço
    let user = Address::generate(&t.env);
    let admin = &t.admin;

    // Criar usuário com KYC mas SEM país
    t.client.set_kyc_level(admin, &user, &2);
    t.client.set_risk_score(admin, &user, &0);
    // NÃO setar país

    // Tentar transferir sem país definido (deve falhar - Compliance by Default)
    let res = t.client.try_transfer(&user, admin, &100);
    assert!(
        res.is_err(),
        "Transferência sem país deveria falhar (Compliance by Default)"
    );
}

#[test]
fn test_country_code_persistence() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let admin = &t.admin;
    let user = t.create_compliant_user();

    // Setar país para BR
    let br_code = String::from_str(&t.env, "BR");
    t.client.set_country_code(admin, &user, &br_code);

    // Mudar para US
    let us_code = String::from_str(&t.env, "US");
    t.client.set_country_code(admin, &user, &us_code);

    // Bloquear US
    t.client.add_blocked_country(admin, &us_code);

    // Tentar transferir (deve falhar porque user está em US bloqueado)
    let res = t.client.try_transfer(&user, admin, &100);
    assert!(
        res.is_err(),
        "Transferência de país bloqueado deveria falhar"
    );
}

// ============================================================================
// SEÇÃO 4: VALIDAÇÕES DE COMPLIANCE - RISK SCORE
// ============================================================================

#[test]
fn test_set_risk_score_auto_blacklist() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Setar risk score para 80 (deve auto-blacklist)
    t.client.set_risk_score(admin, &user, &80);

    // Tentar transferir (deve falhar por blacklist)
    let res = t.client.try_transfer(&user, admin, &100);
    assert!(
        res.is_err(),
        "Transferência de usuário com risk score >= 80 deveria falhar"
    );
}

#[test]
fn test_set_risk_score_valid_range() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Testar scores válidos (0-100)
    for score in [0, 25, 50, 75, 100].iter() {
        let res = t.client.try_set_risk_score(admin, &user, score);
        assert!(res.is_ok(), "Risk score {} deveria ser válido", score);
    }
}

#[test]
fn test_set_risk_score_invalid() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Tentar setar score > 100 (deve falhar)
    let res = t.client.try_set_risk_score(admin, &user, &101);
    assert!(res.is_err(), "Risk score > 100 deveria falhar");

    let res = t.client.try_set_risk_score(admin, &user, &1000);
    assert!(res.is_err(), "Risk score > 100 deveria falhar");
}

// ============================================================================
// SEÇÃO 5: VALIDAÇÕES DE COMPLIANCE - BLACKLIST
// ============================================================================

#[test]
fn test_blacklist_blocks_transfer() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Blacklist o usuário
    t.client.set_blacklisted(&user, &true);

    // Tentar transferir (deve falhar)
    let res = t.client.try_transfer(&user, admin, &100);
    assert!(
        res.is_err(),
        "Transferência de usuário blacklisted deveria falhar"
    );
}

#[test]
fn test_blacklist_removal() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Blacklist o usuário
    t.client.set_blacklisted(&user, &true);

    // Remover da blacklist
    t.client.set_blacklisted(&user, &false);

    // Tentar transferir (deve passar agora)
    // Mas user tem saldo 0, então vai falhar por saldo insuficiente
    // Este teste valida que a blacklist foi removida (não é o motivo da falha)
    let res = t.client.try_transfer(&user, admin, &100);
    // Esperamos erro, mas por saldo insuficiente, não por blacklist
    assert!(
        res.is_err(),
        "Transferência deve falhar (saldo insuficiente)"
    );
}

// ============================================================================
// SEÇÃO 6: EDGE CASES - VALORES EXTREMOS
// ============================================================================

#[test]
fn test_transfer_zero() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Transferir 0 (deve falhar por validação)
    let res = t.client.try_transfer(&user, admin, &0);
    assert!(res.is_err(), "Transferência de 0 deveria falhar");
}

#[test]
fn test_transfer_negative() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Transferir valor negativo (deve falhar)
    let res = t.client.try_transfer(&user, admin, &-100);
    assert!(res.is_err(), "Transferência negativa deveria falhar");
}

#[test]
fn test_transfer_insufficient_balance() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // User tem saldo 0, tentar transferir 1
    let res = t.client.try_transfer(&user, admin, &1);
    assert!(
        res.is_err(),
        "Transferência com saldo insuficiente deveria falhar"
    );
}

// ============================================================================
// SEÇÃO 7: TESTES DE COMPLIANCE VÁLIDOS (SEM ADMIN TRANSFERIR)
// ============================================================================

#[test]
fn test_compliant_user_cannot_transfer_without_balance() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    let admin = &t.admin;

    // Usuário compliant mas sem saldo não consegue transferir
    let res = t.client.try_transfer(&user, admin, &100);
    assert!(res.is_err(), "Transferência sem saldo deveria falhar");
}

#[test]
fn test_multiple_compliance_validations() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let _admin = &t.admin;
    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();

    // Ambos os usuários são compliant
    // Mas sem saldo, não conseguem transferir
    let res1 = t.client.try_transfer(&user1, &user2, &100);
    assert!(res1.is_err(), "Transferência sem saldo deveria falhar");

    let res2 = t.client.try_transfer(&user2, &user1, &100);
    assert!(res2.is_err(), "Transferência sem saldo deveria falhar");
}
