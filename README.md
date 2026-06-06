# ACE Insurance — Decentralized Hack Insurance on Solana

Permissionless insurance protocol for on-chain hack losses. Pay USDC premiums, get covered for wallet compromises, bridge exploits, smart contract vulnerabilities, and more. Claims are validated by a staked validator network selected via Switchboard ECVRF. Payouts are governed entirely by smart contract logic — no human intermediaries, no KYC.

**Program ID (devnet):** `4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU`

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          ACE Insurance Protocol                         │
│                                                                         │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────────────┐  │
│  │  Pool Management│   │Claims Management  │   │Validator Management │  │
│  │                 │   │                  │   │                     │  │
│  │ initialize_pool │   │ submit_claim     │   │ stake_as_validator  │  │
│  │ join_pool       │   │ emergency_payout │   │ validate_claim      │  │
│  │ pay_premium     │   │ fast_track gov   │   │ promote_to_auditor  │  │
│  └────────┬────────┘   └────────┬─────────┘   └──────────┬──────────┘  │
│           │                     │                         │             │
│  ─────────┴─────────────────────┴─────────────────────────┴──────────── │
│                         Shared PDA State                                │
│   InsurancePool │ UserCoverage │ ClaimRequest │ ValidatorStake          │
│  ──────────────────────────────────────────────────────────────────────  │
│           │                     │                         │             │
│  ┌────────▼────────┐   ┌────────▼─────────┐   ┌──────────▼──────────┐  │
│  │  VRF Integration│   │  Distribution    │   │    Risk Score       │  │
│  │                 │   │  Queue           │   │    (OCCR)           │  │
│  │ Phase 1:        │   │                  │   │                     │  │
│  │  request ───►   │   │ add_to_queue     │   │ calculate_discount  │  │
│  │  Switchboard    │   │ distribute_claims│   │ assess_severity     │  │
│  │                 │   │ payout_claim     │   │                     │  │
│  │ Phase 2:        │   └──────────────────┘   └─────────────────────┘  │
│  │  ◄─── oracle    │                                                    │
│  │  fulfills       │   ┌──────────────────┐                             │
│  └─────────────────┘   │  Yield Generation│                             │
│                        │ deposit/withdraw  │                             │
│                        └──────────────────┘                             │
└─────────────────────────────────────────────────────────────────────────┘
```

### Claim Lifecycle

```
submit_claim
     │
     ▼
 [Pending] ──► request_validator_selection ──► Switchboard VRF oracle
                                                       │
                                                       │  ECVRF proof computed off-chain
                                                       │  (unknowable to any on-chain actor
                                                       │   before the oracle publishes it)
                                                       ▼
                                             fulfill_vrf_randomness  (oracle CPI)
                                                       │
                                                       ▼
                                              [UnderValidation]
                                              validators assigned via
                                              Fisher-Yates shuffle
                                              seeded by VRF result
                                                       │
                              ┌────────────────────────┤
                              ▼                        ▼
                         [Approved]               [Rejected]
                              │
                    ┌─────────┴──────────┐
                    ▼                    ▼
              [Distributed]          [Queued]
              payout_claim()      (oversubscribed:
                                   VRF lottery picks
                                   which claims win)
```

### Fast-Track Governance (Emergency Payouts)

For widely-verified protocol exploits (bridge hacks, major rug pulls already documented on Rekt):

```
Pool Authority → propose_fast_track     (24-hour proposal window)
Auditor × 5   → approve_fast_track     (Auditor-tier validators only, 3x vote weight)
Pool Authority → activate_fast_track
Claimant      → request_emergency_payout  → 20% immediate USDC transfer
                                            remaining 80% → normal validation queue
```

---

## Why Switchboard VRF, Not keccak Hash

The prior implementation hashed on-chain data (`slot + timestamp + pubkeys`) to select validators. This is manipulable: a validator can observe the inputs and time their transaction to land in a slot producing a hash that selects friendly validators for their own claim.

Switchboard's **ECVRF (RFC 9381)** closes this attack:

1. The oracle computes randomness off-chain using a secret key — the output is unknowable to anyone before the oracle publishes the proof.
2. The proof is verified on-chain inside the Switchboard program before our callback fires.
3. The oracle is economically slashable on its oracle queue, aligning incentives with honest behavior.

**Two-phase async flow:**

| Phase | Instruction | Caller | What happens |
|-------|------------|--------|-------------|
| 1 | `request_validator_selection` | Claimant | CPI to Switchboard; claim added to `pending_claims` |
| 2 | `fulfill_vrf_randomness` | Switchboard oracle (CPI) | VRF result read from account; validators assigned |

---

## OCCR — On-Chain Coverage Risk Rating

Users who follow verifiable security practices get a premium discount:

| Security Posture | Discount |
|------------------|----------|
| No multisig, no hardware wallet | 0% |
| Multisig only | 10% |
| Hardware wallet only | 10% |
| Both | **20% (max)** |

Discount is recalculated on-chain via `calculate_occr_discount`, creating direct economic incentive for users to adopt best practices.

---

## Validator Tier System

```
STANDARD tier                         AUDITOR tier
─────────────────                     ──────────────────────────────────
Min stake: 0.1 SOL                    Promoted by pool authority
vote_weight: 1                        vote_weight: 3
Initial reputation: 5000              Min reputation: 8000 (threshold)

Reputation mechanics:
  Voted with majority    →  +100 rep, successful_validations++
  Voted against majority →  -200 rep, -6% stake slashed
  "fraud"/"self-hack" in reason (rejected claim)  →  full stake slash, rep = 0
```

---

## Getting Started

### Prerequisites

- Rust 1.79+
- Solana CLI 2.x — `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`
- Anchor CLI 0.31.1 — `cargo install --git https://github.com/coral-xyz/anchor anchor-cli --tag v0.31.1`
- Node.js 18+, Yarn

### Install

```bash
git clone https://github.com/AkashRanjan18/Ace_Insurance
cd Ace_Insurance
yarn install
```

### Build

```bash
anchor build
```

Compiled binary → `target/deploy/ace_insurance.so`  
IDL → `target/idl/ace_insurance.json`

### Test (localnet)

```bash
# Terminal 1
solana-test-validator

# Terminal 2
anchor test --skip-local-validator
```

### Deploy to Devnet

```bash
# Fund your wallet
solana airdrop 4 --url devnet

# Deploy
anchor deploy --provider.cluster devnet

# Verify it's live
solana program show 4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU --url devnet
```

### Set Up Switchboard VRF (Devnet)

```bash
# Install Switchboard CLI
npm install -g @switchboard-xyz/cli

# Create VRF account linked to this program's fulfill_vrf_randomness callback
sb vrf create \
  --cluster devnet \
  --keypair ~/.config/solana/id.json \
  --callback-program-id 4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU \
  --callback-ix-name "fulfill_vrf_randomness"

# Fund VRF escrow with wrapped SOL (pays oracle reward)
sb vrf fund --cluster devnet --vrf <VRF_ACCOUNT_PUBKEY>

# Pass the VRF account pubkey to initialize_vrf_state — you're live
```

---

## Frontend

The `frontend/` directory contains a Next.js 14 app with Tailwind CSS and Solana Wallet Adapter.

```bash
cd frontend
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

**Features:**
- Connect Phantom / Solflare (any Wallet Adapter wallet)
- View live pool stats, join pool, pay premiums
- Submit hack claims with evidence hash, incident timestamp, wallet loss proof
- Track claim status through the full lifecycle
- Validator dashboard — stake SOL, validate claims, track reputation

---

## Program Modules

| Module | File | Responsibility |
|--------|------|----------------|
| Pool Management | `pool_management.rs` | Init pools, join, pay premiums, OCCR discount |
| Claims Management | `claims_management.rs` | Submit claims, fast-track governance, emergency payouts |
| Validators | `validators_management.rs` | Stake, validate, promote, reputation/slash mechanics |
| VRF Integration | `vrf_integration.rs` | **Switchboard ECVRF**-backed random validator selection (2-phase) |
| Distribution | `distribution.rs` | Claim queue, oversubscription lottery, USDC payouts |
| Risk Score | `risk_score.rs` | OCCR discount calculation, severity assessment |
| Yield Generation | `yield_generation.rs` | Deposit/withdraw idle pool funds to external yield vault |

---

## Account Reference

| Account | Seeds | Purpose |
|---------|-------|---------|
| `InsurancePool` | `["pool", authority]` | Pool config, stats, governance flags |
| `UserCoverage` | `["coverage", user, pool]` | Per-user coverage state, OCCR score |
| `ClaimRequest` | `["claim", claimant, pool, timestamp]` | Full claim state + validation votes |
| `ValidatorStake` | `["validator", validator, pool]` | Per-validator stake + reputation |
| `ValidatorStakePool` | `["validator_stake", pool]` | Registry of all validators for a pool |
| `VrfState` | `["vrf_state", pool]` | Switchboard VRF account link + pending queue |
| `DistributionQueue` | `["distribution", pool]` | Approved claims awaiting payout |
| `FastTrackProposal` | `["fast_track", pool, protocol]` | Active governance proposal |
| Pool Vault | `["vault", pool]` | PDA-owned USDC token account |

---

## Design Decisions

**Why Solana?**
Claims validation is latency-sensitive. Solana's 400ms block times let validators submit assessments in seconds. Sub-cent fees mean small-premium users aren't priced out of on-chain interactions.

**Why USDC for premiums?**
Denominating in stablecoins removes actuarial complexity — claim amounts and pool reserves are always dollar-comparable regardless of SOL price.

**Why stake-based validators instead of token governance?**
Token governance favors whales. Requiring validators to put SOL at risk (slashable by reputation decay and outright fraud detection) aligns economic incentives with honest assessment. Auditor-tier validators (verified security researchers) carry 3× vote weight to weight expertise over capital.

**Why two-phase VRF instead of commit-reveal?**
Commit-reveal requires multi-round coordination. If any party ghosts, the protocol stalls. Switchboard VRF is single-sided: the claimant requests, the oracle delivers asynchronously. Liveness depends only on the oracle network.

**Fast-track tradeoff**
The full validation cycle takes 24–72 hours. For publicly documented exploits, that delay harms victims needlessly. The 5-auditor threshold and 24-hour expiry are guardrails against governance capture.

---

## Security Properties

| Property | Mechanism |
|----------|-----------|
| No integer overflow | `checked_add` / `checked_div` on every numeric op |
| Tamper-proof fund custody | PDA-owned vault (pool has no keypair) |
| Unbiased validator selection | Switchboard ECVRF — oracle-sourced, verifiable, unmanipulable |
| Anti-backdating | `joined_at < incident_timestamp` enforced on-chain |
| Evidence requirements | Non-empty IPFS hash + valid protocol address required |
| Wallet loss proof | `balance_before > balance_after` enforced |
| Fraud detection | "fraud"/"self-hack" in rejected claim reason → full stake slash |
| Fast-track guardrails | Auditor majority (5 votes) + 24-hour proposal expiry |
| Emergency payout cap | 20% max, idempotent per claim |

---

## Hack Types Covered

`SmartContractExploit` · `WalletHack` · `PhishingAttack` · `SocialEngineering` · `FlashLoanAttack` · `OracleManipulation` · `ReentrancyAttack` · `AccessControlBypass` · `BridgeExploit` · `RugPull` · `Other`

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Smart contract | Rust, Anchor 0.31.1 |
| Token standard | SPL Token (USDC) |
| Randomness | **Switchboard V2 ECVRF** (RFC 9381) |
| Test suite | TypeScript, Mocha/Chai, `@coral-xyz/anchor` |
| Frontend | Next.js 14, Tailwind CSS, Solana Wallet Adapter |
| Network | Solana Devnet → Mainnet |

---

## Roadmap

- [ ] Mainnet deployment after security audit
- [ ] Switchboard On-Demand migration (single-tx randomness, no async)
- [ ] Yield vault implementation (Kamino / MarginFi integration)
- [ ] Insurance NFT as transferable proof of coverage
- [ ] Cross-pool reinsurance mechanism
- [ ] Governance token for protocol parameter updates

---

## License

MIT

---

*No KYC. No intermediaries. Your coverage, your keys.*
