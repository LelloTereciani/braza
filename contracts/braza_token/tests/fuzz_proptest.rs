#![cfg(test)]
#![cfg(not(tarpaulin_include))]
mod setup;
use proptest::prelude::*;
use setup::TestEnv;

// Definimos as ações possíveis que o Fuzzer pode escolher
#[derive(Debug, Clone)]
enum Action {
    Mint { amount: i128 },
    Transfer { amount: i128 },
    Burn { amount: i128 },
    Approve { amount: i128 },
}

// Estratégia para gerar uma sequência de 1 a 20 ações aleatórias
fn action_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(
        // CORREÇÃO AQUI: Usar prop_oneof! diretamente
        prop_oneof![
            // 40% de chance de ser Mint
            (1..1_000_000i128).prop_map(|a| Action::Mint { amount: a }),
            // 30% de chance de ser Transfer
            (1..1_000_000i128).prop_map(|a| Action::Transfer { amount: a }),
            // 20% de chance de ser Burn
            (1..1_000_000i128).prop_map(|a| Action::Burn { amount: a }),
            // 10% de chance de ser Approve
            (1..1_000_000i128).prop_map(|a| Action::Approve { amount: a }),
        ],
        1..20, // Tamanho da sequência (1 a 20 operações por teste)
    )
}

proptest! {
    // Configuração: Roda 50 sequências diferentes (aumente para 100 ou 1000 se quiser mais rigor)
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn fuzz_stateful_sequence(actions in action_strategy()) {
        let t = TestEnv::new();
        let user_a = t.create_compliant_user();
        let user_b = t.create_compliant_user();

        // Saldo inicial generoso para aguentar o tranco
        t.client.mint(&user_a, &10_000_000);
        t.client.mint(&user_b, &10_000_000);

        for action in actions {
            match action {
                Action::Mint { amount } => {
                    // Admin minta para User A (pode falhar se estourar supply, ignoramos o erro pois é esperado)
                    let _ = t.client.try_mint(&t.admin, &amount);
                    let _ = t.client.try_mint(&user_a, &amount);
                },
                Action::Transfer { amount } => {
                    // A transfere para B
                    let _ = t.client.try_transfer(&user_a, &user_b, &amount);
                },
                Action::Burn { amount } => {
                    // A queima tokens
                    let _ = t.client.try_burn(&user_a, &amount);
                },
                Action::Approve { amount } => {
                    // A aprova B para gastar
                    let _ = t.client.try_approve(&user_a, &user_b, &amount);
                }
            }
        }

        // === INVARIANTE FINAL (A PROVA REAL) ===
        // Após toda a bagunça, a matemática do universo deve se sustentar.

        let supply = t.client.total_supply();
        let bal_a = t.client.balance(&user_a);
        let bal_b = t.client.balance(&user_b);
        let bal_admin = t.client.balance(&t.admin);

        // A soma de todos os saldos DEVE ser igual ao Total Supply
        // (Considerando que só existem esses 3 atores com saldo no teste)
        assert_eq!(supply, bal_a + bal_b + bal_admin, "Quebra de Invariante: Supply != Soma dos Saldos");
    }
}
