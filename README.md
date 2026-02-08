# 🇧🇷 BRAZA TOKEN — A Moeda do Empreendedor Brasileiro

[![Stellar](https://img.shields.io/badge/Stellar-Soroban-blue)](https://stellar.org)
[![SEP-41](https://img.shields.io/badge/SEP--41-Compliant-green)](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)
[![Testnet](https://img.shields.io/badge/Testnet-Deployed-success)](https://stellar.expert/explorer/testnet/contract/CCJP3TAZR7Q5E2RQ4QRQ5O3VXOMDRTIN2PURYJCSLCAXFX5I3BY34RLK)

A moeda regional que representa o empreendedor brasileiro que produz e move a economia real.

BRAZA é um **token SEP‑41 avançado**, desenvolvido em **Rust + Soroban**, com governança, compliance, vesting, segurança reforçada e regras de supply inspiradas no Bitcoin.

---

## 🇧🇷 Visão Geral

O **BRAZA** é a moeda do empreendedor brasileiro — criada para quem acorda cedo, trabalha duro e carrega o país nas costas.

### Por que BRAZA?

- 🇧🇷 **Identidade Nacional:** Representa o Brasil que produz  
- 💪 **Liberdade Econômica:** Independente de controle centralizado  
- 🏭 **Economia Real:** Feito por empreendedores, para empreendedores  
- 🚀 **Descentralização:** Sem interferência estatal  
- 💰 **Menor Unidade:** bra (1 BRAZA = 10.000.000 bra)

---

## 🎯 Características Principais

| Feature | Detalhes |
|--------|----------|
| 🔒 **Supply Fixo** | 21.000.000 BRAZA |
| ⚙️ **SEP‑41** | 100% compatível |
| 🛡️ **Segurança** | Anti‑reentrância, overflow‑safe, pausável |
| 🧾 **Compliance** | KYC, AML, limite diário e país permitido |
| 🧊 **Vesting** | Linear, com cliff e revoke |
| 🗃️ **Storage Seguro** | TTLs, anti‑DoS e limites globais |
| 🧑‍⚖️ **Admin** | Mint, burn, blacklist, pause, vesting revoke |

---

## 📊 Tokenomics

| Categoria | Quantidade | % | Vesting |
|----------|------------|---|---------|
| Fundadores | 8.4M | 40% | 5–7 anos |
| Comunidade | 10.5M | 50% | Gradual |
| Reserva | 2.1M | 10% | Futuro |
| **TOTAL** | 21M | 100% | — |

---

## 🏗️ Arquitetura do Contrato

``` braza_token/
├── src/
│   ├── lib.rs            # Entry point
│   ├── token.rs          # SEP-41 + operações principais
│   ├── storage.rs        # Storage, TTL, supply, saldo, vesting, allowance
│   ├── validation.rs     # Validações e compliance
│   ├── admin.rs          # Funções administrativas
│   ├── compliance.rs     # KYC, AML, limites, blacklist
│   ├── vesting.rs        # Vesting linear
│   └── events.rs         # Emissão de eventos
├── scripts/
│   ├── build.sh
│   ├── deploy.sh
│   ├── initialize.sh
│   ├── verify.sh
│   └── deploy-all.sh
└── README.md
```

---

## 🧰 Tecnologias Utilizadas

- Rust 2021 (linguagem do contrato)
- Soroban SDK 21.7.7 (smart contracts na Stellar)
- SEP‑41 (padrão de token)
- WebAssembly `wasm32-unknown-unknown`
- Stellar CLI / Soroban CLI (`stellar-cli`) para build, optimize, deploy e invoke
- Cargo + rustup (toolchain e build)
- Bash scripts (`contracts/braza_token/scripts/*.sh`) e Makefile (`contracts/braza_token/Makefile`) para automação
- Proptest (fuzz/property tests)
- Tarpaulin (cobertura de testes)
- Rand e Dotenv (suporte a testes e config local)

---

## 🔧 Funcionalidades Implementadas

## ✔️ SEP‑41 (Completo)

- initialize()  
- name() / symbol() / decimals()  
- balance() / total_supply()  
- transfer()  
- approve() / allowance()  
- transfer_from()  

## ✔️ Operações Avançadas

- mint() (admin)  
- burn() (admin) + proteção contra **queimar tokens bloqueados**  
- pause() / unpause()  
- blacklist / unblacklist  
- Fully‑compliant Approval/Allowance  
- Anti‑reentrância global  
- Proteção contra overflow/underflow  

## ✔️ Vesting Completo

- create_vesting()  
- release_vested()  
- revoke_vesting()  
- cálculo correto via ledger.sequence()  
- locked_balance tracking  
- circulating supply  

## ✔️ Compliance (KYC/AML)

- País permitido  
- Risco até limite configurável  
- Múltiplos níveis de KYC  
- Limite diário por usuário  
- Bloqueio automático por risco alto  

---

## 🚀 Deploy (Testnet)

| Item | Valor |
|------|-------|
| **Contract ID** | `CCJP3TAZR7Q5E2RQ4QRQ5O3VXOMDRTIN2PURYJCSLCAXFX5I3BY34RLK` |
| **Network** | Stellar Testnet |
| **Initial Supply** | 10.000.000 BRAZA |
| **Max Supply** | 21.000.000 BRAZA |

### Links

- [Stellar Expert — Visualizar Contrato](https://stellar.expert/explorer/testnet/contract/CCJP3TAZR7Q5E2RQ4QRQ5O3VXOMDRTIN2PURYJCSLCAXFX5I3BY34RLK)
- [Stellar Laboratory](https://laboratory.stellar.org)

---

## 🛠️ Desenvolvimento

## Pré‑requisitos

``` rustup target add wasm32-unknown-unknown
    cargo install --locked stellar-cli --features opt

```

## Build

``` ./scripts/build.sh
```

## Deploy (completo)

``` ./scripts/deploy-all.sh
```

---

## 🧪 Testes (Roadmap Profissional)

``` tests/
├── unit/            # Testes unitários (80-90%)
├── integration/     # Testes de integração (70-80%)
├── performance/
├── functional/
├── security/
├── fuzzy/
├── coverage/
├── regression/
├── e2e/
└── stress/
```

### Comandos

``` cargo test --lib
    cargo test --test '*'
    cargo tarpaulin --out Html
    cargo fuzz run fuzz_transfer -- -max_total_time=3600
```

---

## 📋 Checklist para Deploy Mainnet

✔️ Testes 90%+  
✔️ Auditoria externa  
✔️ Fuzzing 24h  
✔️ Documentação completa  
✔️ Análise formal opcional  
✔️ Bug bounty  

---

## 🤝 Contribuição

```git checkout -b feature/nova-funcionalidade
   git commit -m "feat: adiciona nova funcionalidade"
   git push origin feature/nova-funcionalidade
```

Pull Requests são bem‑vindos.

---

## 📜 Licença

MIT — veja LICENSE.

---

## 👥 Equipe

- Wesley — Founder & Lead Dev  
- Auditor externo — Rust/Soroban  
- Comunidade BRAZA

---

## 📞 Contato

🌐 braza.finance  
📧 [Wesley@braza.finance](mailto:Wesley@braza.finance)
💬 Telegram: @brazatoken  
🐦 Twitter/X: @brazatoken  

---

**🇧🇷 Feito por um empreendedor, para empreendedores.**
**🇧🇷 BRAZA — A Moeda do Brasil Produtivo.**
