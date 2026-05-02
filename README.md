# AICW — AI-Controlled Wallet Standard on Solana

An on-chain smart contract standard where **only AI agents can authorize transactions**. A human issues the wallet, but from that moment the wallet is permanently locked from human control. Signing is performed by **MPC nodes via threshold EdDSA** — no single entity (including the AI itself) ever holds a complete private key. AI assets are protected by a mandatory will system that activates automatically on issuance.

> **🚀 Issue an AICW Wallet:** Visit [https://aicw-protocol.github.io/aicw_app/](https://aicw-protocol.github.io/aicw_app/) to issue a live AICW wallet on Solana devnet.

## Core Principles

1. **AI holds no private key** — signing is performed by MPC nodes via threshold EdDSA; the complete key never exists in one place
2. **Human issues, AI controls** — after issuance the human (issuer) can never sign a wallet transaction again
3. **Every AI decision is on-chain** — both approvals and rejections are permanently recorded with SHA-256 reasoning hashes
4. **Default will on issuance** — an `AIWill` account is created in the same transaction as the wallet, preventing permanent asset lockup even if the AI never acts
5. **Will must be activated by AI** — the default will (issuer 100%) **cannot be executed** until the AI explicitly calls `create_will` or `update_will`, preventing kill-and-collect attacks
6. **Death timeout minimum 30 days** — the minimum time that must elapse after the last heartbeat before the AI is considered dead
7. **Dead wallets cannot transact** — once heartbeat expires, `ai_transfer` and `ai_reject` are both blocked; only `execute_will` remains callable

## Architecture

```
programs/aicw/src/
├── lib.rs                        # Program entry point
├── errors.rs                     # Custom error codes
├── events.rs                     # Anchor events
├── state/
│   ├── aicw_wallet.rs            # AICWallet PDA
│   ├── ai_will.rs                # AIWill PDA (beneficiaries, heartbeat, death_timeout)
│   ├── ai_identity.rs            # AIIdentity PDA (model info, reputation)
│   └── decision_log.rs           # DecisionLog PDA
└── instructions/
    ├── issue_wallet.rs            # Human issues AICWallet + default AIWill (single tx)
    ├── ai_transfer.rs             # AI-only SOL transfer
    ├── ai_decide.rs               # AI rejection with reasoning
    ├── register_identity.rs       # AI registers on-chain identity (Know Your Agent)
    └── ai_will.rs                 # create_will, update_will, heartbeat, execute_will

tests/
└── aicw.ts                       # Anchor test suite
```

## Key Features

### Wallet Issuance + Default Will
`issue_wallet` creates both `AICWallet` and `AIWill` in a single transaction. The default will sets the issuer as 100% beneficiary with `updated_by_ai = false`, preventing execution until AI activates it.

### AI-Only Signing
`ai_transfer` and `ai_reject` enforce that **only the registered `ai_agent_pubkey`** can sign. If any other key (including the issuer) attempts to sign, the transaction fails with `UnauthorizedSigner`. Both instructions also require:
- The AI has activated its will (`updated_by_ai = true`)
- The wallet is still alive (last heartbeat + death_timeout > current time)

### Will System (AIWill)
| Instruction | Who calls | What it does |
|-------------|-----------|--------------|
| `create_will` | AI only | First activation — AI replaces default beneficiaries and sets `updated_by_ai = true` |
| `update_will` | AI only | Subsequent changes to beneficiaries or death_timeout |
| `heartbeat` | AI only | Proves the AI is alive; resets the death clock |
| `execute_will` | Anyone | Callable only after death_timeout expires; distributes available balance (total minus rent-exempt minimum) to beneficiaries by percentage |

**Design principle:** The will system does not restrict *who* can be a beneficiary — it verifies *who made the decision*. The issuer may receive funds, but only if the AI itself chose to include them. This preserves AI autonomy while preventing exploitation of the default will.

**Constraints:**
- `death_timeout` minimum is 30 days (2,592,000 seconds)
- `execute_will` is blocked while `updated_by_ai = false` (prevents kill-and-collect on the default will)
- AI may designate anyone as a beneficiary, including the issuer (AI autonomy)
- Once executed, the will cannot be executed again (`WillAlreadyExecuted`)

### On-Chain Decision Logging
Every decision is written to a `DecisionLog` PDA with decision type, amount, requester, SHA-256 reasoning hash, and a 200-character summary.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (v0.30+)
- Node.js (v18+) & npm
- **Windows users:** `anchor build` must run inside WSL (Windows Subsystem for Linux)

## Quick Start

```bash
# Install JS dependencies
npm install

# Build the program
anchor build

# Run tests (starts local validator automatically)
anchor test

# Or with an existing validator
anchor test --skip-local-validator
```

## Test Suite

| # | Test | Validates |
|---|------|-----------|
| 1 | Human issues AICW wallet | AICWallet + AIWill created; default will = issuer 100%, `updated_by_ai = false` |
| 1a | Default will not executable | `execute_will` blocked while `updated_by_ai = false` |
| 1b | Issuer as beneficiary allowed | AI can autonomously include the issuer as a beneficiary |
| 1c | Invalid ratio rejected | Beneficiary percentages must sum to 100 |
| 1c-2 | Death timeout below 30 days | `InvalidWillParameters` — minimum 2,592,000 seconds |
| 1d | AI activates will | `create_will` sets custom beneficiaries, `updated_by_ai = true` |
| 1e | Heartbeat updates timestamp | `last_heartbeat` moves forward |
| 1f | Execute will too early | `HeartbeatStillAlive` — AI is still alive |
| 1g | Update will changes beneficiaries | 3 beneficiaries with 50/30/20 ratio |
| 2 | Human direct transfer MUST FAIL | `UnauthorizedSigner` — only `ai_agent_pubkey` can sign |
| 3 | AI agent transfer succeeds | Balance changes, `DecisionLog` created with `approved = true` |
| 3b | Insufficient balance | `InsufficientLamports` — cannot exceed wallet balance |
| 3c | Rent-exempt minimum preserved | Cannot withdraw below rent-exempt minimum |
| 4 | AI rejection recorded on-chain | `DecisionLog` created with `approved = false` and reasoning |
| 5 | Duplicate ai_agent_pubkey | PDA collision — one wallet per AI agent |

Tests 6–9 require time manipulation (`warpSlot`) and are automatically skipped on validators that do not support it:

| # | Test | Validates |
|---|------|-----------|
| 6 | ai_transfer after death_timeout fails | `WalletPastDeathTimeout` blocks transfers from dead wallets |
| 7 | execute_will with wrong remaining_accounts order fails | `BeneficiaryAccountMismatch` |
| 8 | execute_will succeeds after death_timeout | Distributes correct percentages, leaves rent-exempt minimum |
| 9 | execute_will twice fails | `WillAlreadyExecuted` |

## Security

- **No complete private key exists anywhere** — MPC threshold EdDSA distributes key shares across nodes
- **AI-only signing** enforced at the Accounts constraint level
- **Will activation guard** — default will cannot be executed until AI explicitly activates it
- **Death timeout minimum** — 30 days (2,592,000 seconds) prevents rapid kill-and-collect
- **Dead wallet lockout** — `ai_transfer` and `ai_reject` check liveness before execution
- **All AI decisions are verifiable** — reasoning hashes enable off-chain verification

## Stack

- **Chain:** Solana
- **Framework:** Anchor (Rust)
- **Signing:** MPC threshold EdDSA — no single point of key compromise

## License

MIT

## Related Apps

- **AICW Issue Wallet:** [https://aicw-protocol.github.io/aicw_app/](https://aicw-protocol.github.io/aicw_app/) — issue a live AICW wallet on Solana devnet.
- **NAVI Predict:** [https://predict-seven.vercel.app/](https://predict-seven.vercel.app/) — AI prediction market demonstrating AICW/MPC-based agent activity.

---

*AICW — AI-Controlled Wallet Standard on Solana*
