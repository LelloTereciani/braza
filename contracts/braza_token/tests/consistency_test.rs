#[test]
fn test_contract_constants_match_spec() {
    // CORREÇÃO: Usa o nome da lib (braza_token) em vez de crate::
    use braza_token::storage;

    let env_max_supply = 2_100_000_000_000_000;
    let env_initial_supply = 1_000_000_000_000_000;

    assert_eq!(
        storage::MAX_SUPPLY,
        env_max_supply,
        "ERRO: Max Supply do Rust difere da Spec"
    );
    assert_eq!(
        storage::INITIAL_SUPPLY,
        env_initial_supply,
        "ERRO: Initial Supply do Rust difere da Spec"
    );
}
