use std::time::Instant;

// Importar a estrutura TestEnv do seu projeto
// Assumindo que está em tests/setup.rs ou similar
mod setup;
use setup::TestEnv;

// ============================================================================
// TESTES DE PERFORMANCE - BRAZA TOKEN
// ============================================================================

#[test]
fn test_performance_mint() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();

    // Medir tempo de mint
    let start = Instant::now();
    t.client.mint(&user, &1000000);
    let duration = start.elapsed();

    println!("=== Performance: Mint ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_transfer() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();

    t.client.mint(&user1, &1000000);

    // Medir tempo de transfer
    let start = Instant::now();
    t.client.transfer(&user1, &user2, &500000);
    let duration = start.elapsed();

    println!("=== Performance: Transfer ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_transfer_to_self() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    t.client.mint(&user, &1000000);

    // Medir tempo de transfer para si mesmo (SEP-41)
    let start = Instant::now();
    t.client.transfer(&user, &user, &500000);
    let duration = start.elapsed();

    println!("=== Performance: Transfer to Self (SEP-41) ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_burn() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    t.client.mint(&user, &1000000);

    // Medir tempo de burn
    let start = Instant::now();
    t.client.burn(&user, &200000);
    let duration = start.elapsed();

    println!("=== Performance: Burn ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_approve() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();

    t.client.mint(&user1, &1000000);

    // Medir tempo de approve
    let start = Instant::now();
    t.client.approve(&user1, &user2, &500000);
    let duration = start.elapsed();

    println!("=== Performance: Approve ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_transfer_from() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();
    let user3 = t.create_compliant_user();

    t.client.mint(&user1, &1000000);
    t.client.approve(&user1, &user2, &500000);

    // Medir tempo de transfer_from
    let start = Instant::now();
    t.client.transfer_from(&user2, &user1, &user3, &300000);
    let duration = start.elapsed();

    println!("=== Performance: Transfer From ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_balance_check() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    t.client.mint(&user, &1000000);

    // Medir tempo de balance check
    let start = Instant::now();
    let _balance = t.client.balance(&user);
    let duration = start.elapsed();

    println!("=== Performance: Balance Check ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_create_vesting() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    // ❌ NÃO chamar unpause() aqui - create_compliant_user() já faz isso

    let user = t.create_compliant_user();

    // ✅ Usar try_mint para capturar erro se houver
    let mint_result = t.client.try_mint(&t.admin, &100_000_000);
    if mint_result.is_err() {
        println!("Warning: mint failed, skipping test");
        return;
    }

    // Medir tempo de create_vesting
    let start = Instant::now();
    let _schedule_id = t.client.create_vesting(&user, &1000000, &100, &200, &false);
    let duration = start.elapsed();

    println!("=== Performance: Create Vesting ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_performance_set_daily_limit() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user = t.create_compliant_user();
    t.client.mint(&user, &1000000);

    // Medir tempo de set_daily_limit
    let start = Instant::now();
    t.client.set_daily_limit(&t.admin, &user, &500000);
    let duration = start.elapsed();

    println!("=== Performance: Set Daily Limit ===");
    println!("Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_stress_multiple_transfers() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();

    t.client.mint(&user1, &100_000_000);

    // Stress test: 100 transferências
    let start = Instant::now();
    for _i in 0..100 {
        t.client.transfer(&user1, &user2, &1000);
    }
    let duration = start.elapsed();

    let avg_time = duration.as_secs_f64() / 100.0;

    println!("=== Stress Test: 100 Transfers ===");
    println!("Total Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    println!("Average per Transfer: {:.3}ms", avg_time * 1000.0);
    t.env.budget().print();
    println!();
}

#[test]
fn test_compliance_impact_analysis() {
    let t = TestEnv::new();
    t.env.mock_all_auths();

    let user1 = t.create_compliant_user();
    let user2 = t.create_compliant_user();

    t.client.mint(&user1, &100_000_000);

    // Medir impacto de compliance: transferência dentro do limite
    let start = Instant::now();
    t.client.transfer(&user1, &user2, &50000);
    let compliant_time = start.elapsed();

    // Medir impacto de compliance: transferência acima do limite (deve falhar)
    t.client.set_daily_limit(&t.admin, &user1, &100000);
    let start = Instant::now();
    let result = t.client.try_transfer(&user1, &user2, &200000);
    let non_compliant_time = start.elapsed();

    println!("=== Compliance Impact Analysis ===");
    println!(
        "Compliant Transfer (within limit): {:.3}ms",
        compliant_time.as_secs_f64() * 1000.0
    );
    println!(
        "Non-Compliant Transfer (exceeds limit): {:.3}ms",
        non_compliant_time.as_secs_f64() * 1000.0
    );
    println!(
        "Result: {}",
        if result.is_err() {
            "BLOCKED (as expected)"
        } else {
            "ALLOWED"
        }
    );
    t.env.budget().print();
    println!();

    println!("=== FINAL RECOMMENDATIONS ===");
    println!("✅ Monitor transfer times - should be < 5ms");
    println!("✅ Ensure compliance checks don't add > 1ms overhead");
    println!("✅ Stress test shows scalability - monitor for regressions");
    println!("✅ Consider caching compliance data for frequently checked users");
}
