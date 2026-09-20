# BRAZA Token

**Classification:** Independent Project · Testnet · Experimental

BRAZA is an experimental SEP-41-oriented token contract study for Stellar/Soroban, implemented in Rust. The repository explores token operations, administrative controls, vesting, compliance-related configuration, storage limits, and property-oriented testing.

## Scope and limitations

- Testnet and local-development experimentation only.
- This repository does not claim a mainnet launch, commercial users, regulatory approval, an external audit, or investment suitability.
- Administrative controls such as minting, burning, pausing, blacklisting, and vesting are included for study and require independent security and economic review.
- The contract address in the README is a Testnet reference and may become stale.

## Development

Requirements: Rust, Cargo, the Stellar CLI, and the WASM target required by the current Soroban toolchain.

```bash
rustup target add wasm32-unknown-unknown
cargo test
```

Deployment helpers are under `contracts/braza_token/scripts/`. Inspect configuration and use only disposable Testnet credentials. Never commit populated `.env` files or private keys.

## Technology

Rust, Soroban, SEP-41 concepts, WebAssembly, Stellar CLI, Cargo, and property-based testing.

## License

MIT. See [LICENSE](LICENSE).

## Author

Lello Tereciani
