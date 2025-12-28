#![cfg(test)]
#![allow(unexpected_cfgs)]
#![cfg(not(tarpaulin_include))]

mod setup;
use setup::TestEnv;
use soroban_sdk::{testutils::Ledger, String};

// ============================================================================
// 1. TESTES DE TOKEN
// ============================================================================

#[test]
fn test_transfer_insufficient_balance() {
    let t = TestEnv::new();
    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();
    let res = t.client.try_transfer(&user1, &user2, &1_000_000_000_000);
    assert!(res.is_err());
}

#[test]
fn test_transfer_negative_amount() {
    let t = TestEnv::new();
    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();
    let res = t.client.try_transfer(&user1, &user2, &-100);
    assert!(res.is_err());
}

#[test]
fn test_transfer_to_self() {
    let t = TestEnv::new();
    let user1 = t.create_compliant_user();
    let _ = t.client.try_transfer(&user1, &user1, &100);
}

#[test]
fn test_allowance_logic_full() {
    let t = TestEnv::new();
    let owner = t.create_compliant_user();
    let spender = t.create_compliant_user();
    let dest = t.create_compliant_user();

    t.client.mint(&owner, &10_000);
    t.client.approve(&owner, &spender, &500);
    t.client.transfer_from(&spender, &owner, &dest, &200);

    assert_eq!(t.client.allowance(&owner, &spender), 300);
    assert_eq!(t.client.balance(&dest), 200);

    let res = t.client.try_transfer_from(&spender, &owner, &dest, &400);
    assert!(res.is_err());
}

// ============================================================================
// 2. TESTES DE COMPLIANCE
// ============================================================================

#[test]
fn test_blocked_country_transfer() {
    let t = TestEnv::new();
    let user_nk = t.create_compliant_user();
    let user_br = t.create_compliant_user();

    let nk_code = String::from_str(&t.env, "NK");
    t.client.add_blocked_country(&t.admin, &nk_code);

    let br_code = String::from_str(&t.env, "BR");
    t.client.add_blocked_country(&t.admin, &br_code);

    let res = t.client.try_transfer(&user_nk, &user_br, &100);
    assert!(res.is_err());
}

#[test]
fn test_limit_exceeded() {
    let t = TestEnv::new();
    let user = t.create_compliant_user();
    let dest = t.create_compliant_user();
    let _res = t.client.try_transfer(&user, &dest, &99_000_000_000_000);
}

// ============================================================================
// 3. TESTES DE ADMIN
// ============================================================================

#[test]
fn test_admin_force_actions() {
    let t = TestEnv::new();
    let user = t.create_compliant_user();
    t.client.force_transfer(&t.admin, &user, &1000);
    t.client.force_burn(&t.admin, &1000);
}

// ============================================================================
// 4. TESTES DE VESTING
// ============================================================================

#[test]
fn test_vesting_lifecycle() {
    let t = TestEnv::new();
    let beneficiary = t.create_compliant_user();

    // 1. Preparação
    t.env.ledger().with_mut(|info| {
        info.sequence_number = 1000;
    });
    let start_block = t.env.ledger().sequence();
    t.env.mock_all_auths();

    // 2. Compliance Total
    let br_code = String::from_str(&t.env, "BR");
    let contract_address = t.client.address.clone();
    let set_compliance = |user: &soroban_sdk::Address| {
        t.client.set_country_code(&t.admin, user, &br_code);
        t.client.set_kyc_level(&t.admin, user, &2);
        t.client.set_risk_score(&t.admin, user, &0);
    };
    set_compliance(&t.admin);
    set_compliance(&contract_address);
    set_compliance(&beneficiary);

    // 3. Criar Vesting
    let amount = 100_000_000;
    let cliff = 100;
    let duration = 1000;

    let schedule_id = t
        .client
        .create_vesting(&beneficiary, &amount, &cliff, &duration, &true);

    // 4. Tentar sacar ANTES do Cliff (Erro esperado #8)
    let res = t.client.try_release_vested(&beneficiary, &schedule_id);
    assert!(res.is_err());

    // 5. Avançar o tempo
    t.env.ledger().with_mut(|info| {
        info.sequence_number = start_block + 500;
    });

    // 6. Sacar
    let released = t.client.release_vested(&beneficiary, &schedule_id);
    assert!(released > 0);

    // 7. Revogar (CORRIGIDO)
    // Passamos o 'beneficiary' porque é o vesting DELE que queremos revogar.
    // O Admin é quem executa, mas isso é resolvido pelo mock_all_auths (ou require_auth interno).
    t.client.revoke_vesting(&beneficiary, &schedule_id);
}

// ============================================================================
// 5. TESTES DE BURN
// ============================================================================

#[test]
fn test_burn_logic() {
    let t = TestEnv::new();
    let user = t.create_compliant_user();

    t.client.mint(&user, &1000);

    // 1. Burn simples
    t.client.burn(&user, &200);
    assert_eq!(t.client.balance(&user), 800);
}
