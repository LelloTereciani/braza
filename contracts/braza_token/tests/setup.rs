#![cfg(test)]
#![cfg(not(tarpaulin_include))]
use soroban_sdk::{Address, Env, String};
// IMPORTANTE: Trazemos o Trait para o escopo para habilitar Address::generate()
use soroban_sdk::testutils::Address as _;

use braza_token::token::{BrazaToken, BrazaTokenClient};

pub struct TestEnv<'a> {
    pub env: Env,
    pub client: BrazaTokenClient<'a>,
    pub admin: Address,
}

impl<'a> TestEnv<'a> {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        // MOCKING: Comentado para evitar o erro de Ledger que travava tudo.
        // env.ledger().set_sequence(100_000);
        // env.ledger().set_timestamp(1690000000);

        // Agora Address::generate vai funcionar porque importamos o Trait acima
        let admin = Address::generate(&env);

        let contract_id = env.register_contract(None, BrazaToken);
        let client = BrazaTokenClient::new(&env, &contract_id);

        client.initialize(
            &admin,
            &String::from_str(&env, "Braza Token"),
            &String::from_str(&env, "BRZ"),
        );

        Self { env, client, admin }
    }

    pub fn create_compliant_user(&self) -> Address {
        // Address::generate funciona aqui também
        let user = Address::generate(&self.env);

        self.client
            .set_country_code(&self.admin, &user, &String::from_str(&self.env, "BR"));
        self.client.set_kyc_level(&self.admin, &user, &2);
        self.client.set_risk_score(&self.admin, &user, &0);

        user
    }
    #[allow(dead_code)]
    pub fn jump_time(&self, _ledgers: u32) {
        // Função vazia para manter compatibilidade
    }
}
