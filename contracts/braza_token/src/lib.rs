#![no_std]
#![allow(dead_code)]
use soroban_sdk::contract;

mod storage;
mod types;
mod validation;
mod vesting;
mod events;
mod token;
mod admin;
mod compliance;

// ============================================================================
// CONTRATO PRINCIPAL
// ============================================================================

#[contract]
pub struct BrazaTokenContract;

pub use token::BrazaToken;
pub use types::*;


