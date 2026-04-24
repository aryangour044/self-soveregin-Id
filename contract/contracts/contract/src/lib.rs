#![no_std]

use soroban_sdk::{
    Address, Env,     contract, contractimpl, contracttype, symbol_short,
Map, String, Symbol, Vec,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const OWNER: Symbol = symbol_short!("OWNER");
const FIELDS: Symbol = symbol_short!("FIELDS");
const GRANTS: Symbol  =symbol_short!("GRANTS");

// ─── Data Types ──────────────────────────────────────────────────────────────

/// A single identity field (e.g. "name", "email", "dob")
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct IdentityField {
    pub key: String,
    pub value: String,
    pub is_public: bool,
}

/// A selective-disclosure grant given to a verifier address
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Grant {
    pub verifier: Address,
    pub allowed_fields: Vec<String>,
    pub expires_at: u64, // Unix timestamp; 0 = no expiry
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct IdentityVaultContract;

#[contractimpl]
impl IdentityVaultContract {
    // ── Initialisation ───────────────────────────────────────────────────────

    /// Initialise the vault. Must be called once by the identity owner.
    pub fn initialize(env: Env, owner: Address) {
        if env.storage().instance().has(&OWNER) {
            panic!("already initialised");
        }
        env.storage().instance().set(&OWNER, &owner);
        env.storage()
            .instance()
            .set(&FIELDS, &Map::<String, IdentityField>::new(&env));
        env.storage()
            .instance()
            .set(&GRANTS, &Map::<Address, Grant>::new(&env));
    }

    // ── Identity Management ──────────────────────────────────────────────────

    /// Add or update an identity field. Only the owner may call this.
    pub fn set_field(
        env: Env,
        key: String,
        value: String,
        is_public: bool,
    ) {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        owner.require_auth();

        let mut fields: Map<String, IdentityField> =
            env.storage().instance().get(&FIELDS).unwrap();

        fields.set(
            key.clone(),
            IdentityField { key, value, is_public },
        );

        env.storage().instance().set(&FIELDS, &fields);
    }

    /// Remove an identity field. Only the owner may call this.
    pub fn remove_field(env: Env, key: String) {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        owner.require_auth();

        let mut fields: Map<String, IdentityField> =
            env.storage().instance().get(&FIELDS).unwrap();

        fields.remove(key);
        env.storage().instance().set(&FIELDS, &fields);
    }

    // ── Selective Disclosure ─────────────────────────────────────────────────

    /// Grant a verifier access to a specific list of fields.
    pub fn grant_access(
        env: Env,
        verifier: Address,
        allowed_fields: Vec<String>,
        expires_at: u64,
    ) {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        owner.require_auth();

        let mut grants: Map<Address, Grant> =
            env.storage().instance().get(&GRANTS).unwrap();

        grants.set(
            verifier.clone(),
            Grant { verifier, allowed_fields, expires_at },
        );

        env.storage().instance().set(&GRANTS, &grants);
    }

    /// Revoke a verifier's access entirely. Only the owner may call this.
    pub fn revoke_access(env: Env, verifier: Address) {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        owner.require_auth();

        let mut grants: Map<Address, Grant> =
            env.storage().instance().get(&GRANTS).unwrap();

        grants.remove(verifier);
        env.storage().instance().set(&GRANTS, &grants);
    }

    // ── Read Access ──────────────────────────────────────────────────────────

    /// Read a field value. Rules:
    ///   - Owner can always read any field.
    ///   - Public fields are readable by anyone.
    ///   - A verifier with a valid (non-expired) grant may read allowed fields.
    pub fn get_field(env: Env, caller: Address, key: String) -> Option<String> {
        caller.require_auth();

        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        let fields: Map<String, IdentityField> =
            env.storage().instance().get(&FIELDS).unwrap();

        let field = match fields.get(key.clone()) {
            Some(f) => f,
            None => return None,
        };

        // Owner sees everything
        if caller == owner {
            return Some(field.value);
        }

        // Public fields visible to all
        if field.is_public {
            return Some(field.value);
        }

        // Check grant
        let grants: Map<Address, Grant> =
            env.storage().instance().get(&GRANTS).unwrap();

        if let Some(grant) = grants.get(caller) {
            let now = env.ledger().timestamp();
            let valid = grant.expires_at == 0 || grant.expires_at > now;
            if valid {
                for allowed in grant.allowed_fields.iter() {
                    if allowed == key {
                        return Some(field.value);
                    }
                }
            }
        }

        None // Access denied
    }

    /// Return all field keys (owner only).
    pub fn list_fields(env: Env, caller: Address) -> Vec<String> {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        if caller != owner {
            panic!("not authorised");
        }
        caller.require_auth();

        let fields: Map<String, IdentityField> =
            env.storage().instance().get(&FIELDS).unwrap();

        fields.keys()
    }

    /// Return all active grants (owner only).
    pub fn list_grants(env: Env, caller: Address) -> Vec<Grant> {
        let owner: Address = env.storage().instance().get(&OWNER).unwrap();
        if caller != owner {
            panic!("not authorised");
        }
        caller.require_auth();

        let grants: Map<Address, Grant> =
            env.storage().instance().get(&GRANTS).unwrap();

        grants.values()
    }

    /// Return the vault owner address.
    pub fn get_owner(env: Env) -> Address {
        env.storage().instance().get(&OWNER).unwrap()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec, Env};

    fn setup() -> (Env, Address, IdentityVaultContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, IdentityVaultContract);
        let client = IdentityVaultContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        client.initialize(&owner);
        (env, owner, client)
    }

    #[test]
    fn test_set_and_get_public_field() {
        let (env, owner, client) = setup();
        let key = String::from_str(&env, "name");
        let value = String::from_str(&env, "Alice");

        client.set_field(&key, &value, &true);

        // Owner can read
        let result = client.get_field(&owner, &key);
        assert_eq!(result, Some(value.clone()));

        // Stranger can also read (it's public)
        let stranger = Address::generate(&env);
        let result2 = client.get_field(&stranger, &key);
        assert_eq!(result2, Some(value));
    }

    #[test]
    fn test_private_field_hidden_from_stranger() {
        let (env, _owner, client) = setup();
        let key = String::from_str(&env, "ssn");
        let value = String::from_str(&env, "123-45-6789");

        client.set_field(&key, &value, &false);

        let stranger = Address::generate(&env);
        let result = client.get_field(&stranger, &key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_selective_grant() {
        let (env, _owner, client) = setup();
        let key_email = String::from_str(&env, "email");
        let key_ssn = String::from_str(&env, "ssn");

        client.set_field(&key_email, &String::from_str(&env, "alice@example.com"), &false);
        client.set_field(&key_ssn, &String::from_str(&env, "123-45-6789"), &false);

        let verifier = Address::generate(&env);
        client.grant_access(
            &verifier,
            &vec![&env, key_email.clone()],
            &0u64, // no expiry
        );

        // Verifier can see email
        let email = client.get_field(&verifier, &key_email);
        assert!(email.is_some());

        // But not SSN
        let ssn = client.get_field(&verifier, &key_ssn);
        assert_eq!(ssn, None);
    }

    #[test]
    fn test_revoke_access() {
        let (env, _owner, client) = setup();
        let key = String::from_str(&env, "email");
        client.set_field(&key, &String::from_str(&env, "alice@example.com"), &false);

        let verifier = Address::generate(&env);
        client.grant_access(&verifier, &vec![&env, key.clone()], &0u64);

        // Has access before revoke
        assert!(client.get_field(&verifier, &key).is_some());

        client.revoke_access(&verifier);

        // No access after revoke
        assert_eq!(client.get_field(&verifier, &key), None);
    }
}