#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

/// Status codes returned by `verify_seal`.
const STATUS_NONE: u32 = 0;
const STATUS_VALID: u32 = 1;
const STATUS_EXPIRED: u32 = 2;
const STATUS_REVOKED: u32 = 3;

/// On-chain record describing a fair-trade certification seal.
#[derive(Clone)]
#[contracttype]
pub struct Seal {
    pub auditor: Address,
    pub standard: Symbol,
    pub issued_at: u64,
    pub expires_at: u64,
    pub status: u32,
    pub reason: Symbol,
}

/// Storage keys; each producer cooperative is identified by a `Symbol`.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// The currently active seal record for a producer.
    Seal(Symbol),
    /// Cumulative number of seals (issued + renewed) per producer.
    Count(Symbol),
}

#[contract]
pub struct FairTradeSeal;

#[contractimpl]
impl FairTradeSeal {
    /// Issue a brand-new fair-trade seal for `producer_id`.
    ///
    /// * `auditor` must authorize the call (`require_auth`).
    /// * `standard` is a short symbol identifying the certification scheme
    ///   (e.g. `FAIRTRADE`, `RAINFORST`, `ORGANIC`).
    /// * `expires_at` is a Unix timestamp (seconds). Must be strictly greater
    ///   than the current ledger timestamp.
    ///
    /// Panics if a non-revoked, non-expired seal already exists for that
    /// producer — use `renew_seal` instead. Returns the new cumulative seal
    /// count for the producer.
    pub fn issue_seal(
        env: Env,
        auditor: Address,
        producer_id: Symbol,
        standard: Symbol,
        expires_at: u64,
    ) -> u32 {
        auditor.require_auth();

        let now = env.ledger().timestamp();
        if expires_at <= now {
            panic!("expires_at must be in the future");
        }

        let seal_key = DataKey::Seal(producer_id.clone());
        if let Some(existing) = env.storage().persistent().get::<DataKey, Seal>(&seal_key) {
            if existing.status == STATUS_VALID && existing.expires_at > now {
                panic!("active seal already exists; call renew_seal instead");
            }
        }

        let seal = Seal {
            auditor: auditor.clone(),
            standard,
            issued_at: now,
            expires_at,
            status: STATUS_VALID,
            reason: symbol_short!("NONE"),
        };
        env.storage().persistent().set(&seal_key, &seal);

        let count_key = DataKey::Count(producer_id);
        let new_count: u32 = env
            .storage()
            .persistent()
            .get::<DataKey, u32>(&count_key)
            .unwrap_or(0)
            + 1;
        env.storage().persistent().set(&count_key, &new_count);

        new_count
    }

    /// Revoke a producer's seal (e.g. for non-compliance, fraud).
    ///
    /// Only the original certifying `auditor` may revoke. The `reason` is a
    /// short symbol such as `FRAUD`, `NONCOMPL`, `WITHDRAWN`. Panics if no
    /// seal exists or if the caller is not the original auditor.
    pub fn revoke_seal(env: Env, auditor: Address, producer_id: Symbol, reason: Symbol) {
        auditor.require_auth();

        let seal_key = DataKey::Seal(producer_id);
        let mut seal: Seal = env
            .storage()
            .persistent()
            .get(&seal_key)
            .unwrap_or_else(|| panic!("no seal found for this producer"));

        if seal.auditor != auditor {
            panic!("only the issuing auditor can revoke this seal");
        }
        if seal.status == STATUS_REVOKED {
            panic!("seal already revoked");
        }

        seal.status = STATUS_REVOKED;
        seal.reason = reason;
        env.storage().persistent().set(&seal_key, &seal);
    }

    /// Verify the current status of a producer's seal.
    ///
    /// Returns:
    /// * `0` — no seal has ever been issued
    /// * `1` — seal is valid and not yet expired
    /// * `2` — seal exists but has expired (needs `renew_seal`)
    /// * `3` — seal was explicitly revoked by the auditor
    pub fn verify_seal(env: Env, producer_id: Symbol) -> u32 {
        let seal_key = DataKey::Seal(producer_id);
        let seal: Seal = match env.storage().persistent().get(&seal_key) {
            Some(s) => s,
            None => return STATUS_NONE,
        };

        if seal.status == STATUS_REVOKED {
            return STATUS_REVOKED;
        }
        if seal.expires_at <= env.ledger().timestamp() {
            return STATUS_EXPIRED;
        }
        STATUS_VALID
    }

    /// Renew an existing seal with a later expiry timestamp.
    ///
    /// Only the original certifying `auditor` may renew. A revoked seal
    /// cannot be renewed — a fresh `issue_seal` is required after the
    /// producer is re-audited. `new_expiry` must be strictly greater than
    /// the current ledger timestamp. Increments the cumulative seal count.
    pub fn renew_seal(env: Env, auditor: Address, producer_id: Symbol, new_expiry: u64) {
        auditor.require_auth();

        let now = env.ledger().timestamp();
        if new_expiry <= now {
            panic!("new_expiry must be in the future");
        }

        let seal_key = DataKey::Seal(producer_id.clone());
        let mut seal: Seal = env
            .storage()
            .persistent()
            .get(&seal_key)
            .unwrap_or_else(|| panic!("no seal found for this producer"));

        if seal.auditor != auditor {
            panic!("only the issuing auditor can renew this seal");
        }
        if seal.status == STATUS_REVOKED {
            panic!("revoked seals cannot be renewed; issue a new seal");
        }

        seal.status = STATUS_VALID;
        seal.issued_at = now;
        seal.expires_at = new_expiry;
        seal.reason = symbol_short!("NONE");
        env.storage().persistent().set(&seal_key, &seal);

        let count_key = DataKey::Count(producer_id);
        let new_count: u32 = env
            .storage()
            .persistent()
            .get::<DataKey, u32>(&count_key)
            .unwrap_or(0)
            + 1;
        env.storage().persistent().set(&count_key, &new_count);
    }

    /// Return the cumulative number of seals (issued + renewals) recorded
    /// for `producer_id`. Useful as a public reputation signal — a long
    /// renewal history indicates sustained compliance.
    pub fn list_seals(env: Env, producer_id: Symbol) -> u32 {
        env.storage()
            .persistent()
            .get::<DataKey, u32>(&DataKey::Count(producer_id))
            .unwrap_or(0)
    }

    /// Return the `Address` of the auditor who certified `producer_id`.
    /// Panics if the producer has never been issued a seal.
    pub fn get_auditor(env: Env, producer_id: Symbol) -> Address {
        let seal: Seal = env
            .storage()
            .persistent()
            .get(&DataKey::Seal(producer_id))
            .unwrap_or_else(|| panic!("no seal found for this producer"));
        seal.auditor
    }

    /// Return the certification standard symbol (e.g. `FAIRTRADE`) recorded
    /// on the producer's current seal. Panics if no seal exists.
    pub fn get_standard(env: Env, producer_id: Symbol) -> Symbol {
        let seal: Seal = env
            .storage()
            .persistent()
            .get(&DataKey::Seal(producer_id))
            .unwrap_or_else(|| panic!("no seal found for this producer"));
        seal.standard
    }
}
