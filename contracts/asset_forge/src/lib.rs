#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 1,
    AttestationExpired = 2,
    ReserveDeficit = 3,
    TransferRestricted = 4,
}

#[contracttype]
#[derive(Clone)]
pub struct ReserveAttestation {
    pub auditor_pubkey: BytesN<32>,
    pub timestamp: u64,
    pub verified_grams_reserve: u128,
    pub merkle_root: BytesN<32>,
}

#[contracttype]
pub enum DataKey {
    Admin,
    TotalSupply,
    ReserveProof,
}

#[contract]
pub struct AssetForgeRWA;

#[contractimpl]
impl AssetForgeRWA {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0u128);
    }

    pub fn submit_reserve_attestation(
        env: Env,
        attestation: ReserveAttestation,
        max_staleness_seconds: u64,
    ) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let current_time = env.ledger().timestamp();
        if current_time > attestation.timestamp + max_staleness_seconds {
            return Err(Error::AttestationExpired);
        }

        let total_supply: u128 = env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0);
        if attestation.verified_grams_reserve < total_supply {
            return Err(Error::ReserveDeficit);
        }

        env.storage().instance().set(&DataKey::ReserveProof, &attestation);
        env.events().publish(
            (symbol_short!("attest"), symbol_short!("success")),
            attestation.verified_grams_reserve,
        );

        Ok(())
    }
}
