# 🔐 Identity Vault — Self-Sovereign Identity on Stellar

> **Own your data. Share only what you choose.**

Identity Vault is a [Soroban](https://soroban.stellar.org) smart contract on the Stellar blockchain that puts users in complete control of their personal identity data. Instead of handing your information to centralised services and hoping they keep it safe, you store it in your own on-chain vault and grant (or revoke) access to specific fields, for specific parties, at any time.

---

## 📖 Project Description

Traditional identity systems are broken. Your name, email, date of birth, passport number, and other sensitive fields are scattered across dozens of apps — each one a potential breach. Identity Vault flips this model: **you are the custodian**.

Built on Stellar's Soroban smart-contract platform, the vault is a tamper-proof container for your identity fields. No third party can read a private field unless you explicitly grant them access, and you can revoke that access at any moment — even if the verifier still holds the grant on their side.

---

## ⚙️ What It Does

| Action | Who can do it | Description |
|---|---|---|
| `initialize` | Deployer (once) | Creates the vault and registers the owner address |
| `set_field` | Owner | Adds or updates an identity field (key / value / public flag) |
| `remove_field` | Owner | Permanently deletes a field from the vault |
| `grant_access` | Owner | Allows a verifier address to read a named list of fields, with an optional expiry timestamp |
| `revoke_access` | Owner | Instantly removes all access for a verifier |
| `get_field` | Caller (auth required) | Returns a field value — enforcing visibility rules at the contract level |
| `list_fields` | Owner | Returns all stored field keys |
| `list_grants` | Owner | Returns all active grants |
| `get_owner` | Anyone | Returns the vault owner address |

### Visibility Rules (enforced on-chain)

```
Owner        → can read any field, always
Public field → readable by any authenticated caller
Verifier     → can read only the fields named in their grant, only before expiry
Everyone else → receives None (access denied, no error)
```

No off-chain middleware. No API to compromise. The rules live in the contract.

---

## ✨ Features

### 🗃️ Structured Identity Storage
Store any key-value identity field — `name`, `email`, `date_of_birth`, `passport_number`, `tax_id`, or anything custom. Each field carries a public/private flag so you decide its default visibility.

### 🔍 Selective Disclosure
Grant a verifier (a dApp, an employer, a government portal) access to *only the fields they need*. A KYC provider can read your `dob` and `passport_number` without ever seeing your `email` or home `address`.

### ⏰ Time-Bound Grants
Every grant accepts an optional Unix-timestamp expiry. Set `expires_at = 0` for a permanent grant, or supply a future timestamp to auto-expire access — no cron job required.

### 🚫 Instant Revocation
Call `revoke_access` to immediately cut off a verifier. Because the rule is enforced at read-time on-chain, there is no cached copy they can continue to use.

### 🔒 Auth-Gated Reads
`get_field` requires `require_auth()` on the caller. Every read is an explicit, signed action — no anonymous scraping.

### 🧪 Full Test Coverage
The contract ships with unit tests covering:
- Public field read by owner and stranger
- Private field hidden from unauthorised callers
- Selective grant (email yes, SSN no)
- Access revocation

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM target
rustup target add wasm32-unknown-unknown

# Install the Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Build

```bash
git clone https://github.com/your-org/identity-vault
cd identity-vault

stellar contract build
```

The compiled `.wasm` is output to `target/wasm32-unknown-unknown/release/identity_vault.wasm`.

### Test

```bash
cargo test --features testutils
```

### Deploy to Testnet

```bash
# Fund a test account
stellar keys generate alice --network testnet
stellar keys fund alice --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/identity_vault.wasm \
  --source alice \
  --network testnet
```

### Initialise & Use

```bash
CONTRACT_ID=<your_deployed_contract_id>
ALICE=$(stellar keys address alice)

# Initialise
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- initialize --owner $ALICE

# Set a private field
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- set_field --key email --value alice@example.com --is_public false

# Grant a verifier access to email only (expires 2026-12-31)
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- grant_access \
     --verifier GVERIFIER... \
     --allowed_fields '["email"]' \
     --expires_at 1767225600

# Revoke at any time
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet \
  -- revoke_access --verifier GVERIFIER...
```

---

## 🗂️ Project Structure

```
identity-vault/
├── Cargo.toml                          # Workspace manifest
└── contracts/
    └── identity_vault/
        ├── Cargo.toml                  # Contract dependencies
        └── src/
            └── lib.rs                  # Contract logic + tests
```

---

## 🛣️ Roadmap

- [ ] Multi-field batch set / remove
- [ ] Verifier-specific read log (audit trail)
- [ ] Encrypted field values (client-side encryption + on-chain ciphertext)
- [ ] Credential attestations (third-party issuers sign fields)
- [ ] Frontend dApp (React + Freighter wallet)

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.
