# ACE Insurance — On-Chain Hack Insurance Protocol

> **Decentralized crypto hack insurance on Solana.** Users pay premiums, stake validators, submit claims with on-chain evidence, and receive payouts — entirely governed by smart contract logic with no centralized adjudicator.

---

##  Status

| Component | Status |
|---|---|
| Smart Contract (Anchor/Rust) |  Complete |
| Test Suite (TypeScript) |  Complete |
| Devnet Deployment |  In Progress |
| Frontend (Next.js) |  In Progress |

> Program ID (Devnet): `coming soon`

---

## What Is ACE Insurance?

ACE is a **permissionless insurance protocol** for on-chain hack losses — wallet hacks, phishing attacks, bridge exploits, flash loan attacks, smart contract exploits.

Users join a pool, pay premiums (in USDC), and gain coverage up to a defined cap. When a hack occurs, they submit a claim with on-chain evidence. A decentralized validator set — selected via pseudo-VRF — investigates and votes. Approved claims are paid from the pool vault. Everything is verifiable on-chain.

### Why It Exists

Centralized crypto insurance (Nexus Mutual requires KYC, Etherisc is Ethereum-only) leaves a gap: **a permissionless, Solana-native insurance primitive**. ACE fills that gap.

---

## Protocol Architecture

```
                          ┌─────────────────────────────────────┐
                          │           ACE Insurance Pool         │
                          │                                       │
                          │  premium_amount  coverage_amount      │
                          │  min_validators  claim_period         │
                          │  fast_track_active  governance_auth   │
                          └────────────┬────────────┬────────────┘
                                       │            │
                    ┌──────────────────┘            └──────────────────┐
                    ▼                                                   ▼
         ┌──────────────────┐                             ┌────────────────────┐
         │   Pool Vault     │                             │  Validator System   │
         │  (USDC PDA)      │◄──── premiums flow in       │                    │
         │                  │                             │  ValidatorStake    │
         │                  │───── payouts flow out ────► │  ValidatorStakePool│
         └──────────────────┘                             │  VRF State         │
                                                          └────────────────────┘
                    │
                    ▼
         ┌──────────────────┐
         │ DistributionQueue│
         │                  │
         │ Normal: pay all  │
         │ Oversubscribed:  │
         │   VRF lottery    │
         └──────────────────┘
```

---

## Claim Lifecycle

```
  User joins pool           User pays premium         Incident occurs
  [join_pool]               [pay_premium]             [submit_claim]
       │                          │                         │
       ▼                          ▼                         ▼
 UserCoverage PDA          coverage_active = true    ClaimRequest PDA
 occr_discount set         premiums_paid++           status: Pending
 coverage_amount set                                 evidence_hash stored
                                                     incident_timestamp set
                                                            │
                                                            ▼
                                                  [request_validator_selection]
                                                  VRF selects min_validators+2
                                                  validators_assigned populated
                                                  status: UnderValidation
                                                            │
                                          ┌─────────────────┴──────────────────┐
                                          ▼                                     ▼
                                   Validators vote                       [fast_track path]
                                   [validate_claim]                      5 auditor approvals
                                   approve/reject +                      → pool.fast_track_active
                                   technical_analysis                    → 20% emergency payout
                                          │
                              ┌───────────┴───────────┐
                              ▼                       ▼
                         majority YES           majority NO
                         status: Approved       status: Rejected
                              │                 fraud detected?
                              ▼                       ▼
                    [add_to_distribution_queue]  slash_validator_full()
                    [distribute_claims]          reputation = 0
                    [payout_claim]               stake = 0
                    USDC → claimant
```

---

## OCCR Discount System

**OCCR** = On-Chain Coverage Risk Rating. Users who follow security best practices pay lower premiums.

| Security Posture | Discount |
|---|---|
| Neither multisig nor hardware wallet | 0% |
| Multisig only | 10% |
| Hardware wallet only | 10% |
| Both multisig + hardware wallet | **20%** |

The discount is recalculated on-chain anytime via `calculate_occr_discount`. This creates a **direct economic incentive** for users to adopt best practices.

---

## Validator Tier System

```
                    ┌─────────────────────────────────────────────┐
                    │              Validator Tiers                 │
                    │                                              │
                    │   STANDARD                  AUDITOR          │
                    │   ─────────                 ───────          │
                    │   Min stake: 0.1 SOL        Promoted by auth  │
                    │   vote_weight: 1            vote_weight: 3    │
                    │   Initial rep: 5000         Min rep: [thresh] │
                    │                                              │
                    │   Promotion path:                            │
                    │   reputation ≥ AUDITOR_THRESHOLD             │
                    │   → promote_to_auditor()                     │
                    └─────────────────────────────────────────────┘

  Reputation Mechanics:
  ├── Voted with majority  →  +100 reputation, successful_validations++
  └── Voted against majority →  -200 reputation, -6% stake slashed

  Fraud Detection:
  └── Claim rejected + "fraud"/"self-hack" in reason → slash_validator_full()
      reputation = 0, stake = 0
```

---

## Fast-Track Emergency System

For verified large-scale protocol exploits (e.g. a major bridge hack), ACE has a governance fast-track:

```
  authority proposes          auditors vote           5 approvals reached
  [propose_fast_track]   →   [approve_fast_track]  →  [activate_fast_track]
  approval_count = 1         Auditor tier only       pool.fast_track_active = true
  expires_at = +24h          +1 per vote             24h window enforced
                                                              │
                                                             ▼
                                                   [request_emergency_payout]
                                                   20% of claim amount
                                                   paid immediately
                                                   remaining goes to
                                                   normal validation queue
```

---

## Distribution Queue

When multiple claims are approved simultaneously:

```
  available_funds >= total_requested        available_funds < total_requested
  ─────────────────────────────────        ──────────────────────────────────
  Normal distribution:                     Oversubscribed:
  All approved claims paid in full         VRF randomness selects claims
  selected_claims = pending_claims         Fair lottery — not first-come-first-served
```

---

## Account Structure

| Account | Seeds | Purpose |
|---|---|---|
| `InsurancePool` | `["pool", authority]` | Pool config, stats, governance |
| `UserCoverage` | `["coverage", user, pool]` | Per-user coverage state, OCCR |
| `ClaimRequest` | `["claim", claimant, pool, timestamp]` | Claim state + validations |
| `ValidatorStake` | `["validator", validator, pool]` | Per-validator stake + reputation |
| `ValidatorStakePool` | `["validator_stake", pool]` | Registry of all validators |
| `VrfState` | `["vrf_state", pool]` | Randomness state for selection |
| `DistributionQueue` | `["distribution", pool]` | Claim payout queue |
| `FastTrackProposal` | `["fast_track", pool, protocol]` | Governance proposal |
| Pool Vault | `["vault", pool]` | USDC token vault (PDA-owned) |

---

## Instruction Reference

### Pool Management
| Instruction | Description |
|---|---|
| `initialize_pool` | Create a new insurance pool with premium, coverage, validator config |
| `join_pool` | Join pool, pay initial premium, set security posture |
| `pay_premium` | Pay monthly premium with OCCR discount applied |

### Claims
| Instruction | Description |
|---|---|
| `submit_claim` | Submit hack claim with evidence hash, incident timestamp, wallet loss proof |
| `request_emergency_payout` | Claim 20% immediately when fast-track is active |
| `assess_hack_severity` | Authority/validator sets `SeverityTier` on claim |

### Validators
| Instruction | Description |
|---|---|
| `initialize_validator_stake` | Set up validator registry for pool |
| `stake_as_validator` | Stake ≥ 0.1 SOL, enter validator set |
| `validate_claim` | Vote approve/reject with technical analysis (≤500 chars) |
| `promote_to_auditor` | Promote validator to Auditor tier (3x vote weight) |

### Distribution
| Instruction | Description |
|---|---|
| `initialize_distribution_queue` | Set up distribution queue for pool |
| `add_to_distribution_queue` | Add approved claim to queue |
| `distribute_claims` | Run normal or VRF-lottery distribution |
| `payout_claim` | Execute individual USDC transfer to claimant |

### Governance
| Instruction | Description |
|---|---|
| `propose_fast_track` | Propose emergency fast-track for known protocol exploit |
| `approve_fast_track` | Auditor-tier validator approves proposal |
| `activate_fast_track` | Activate after 5 auditor approvals (24h window) |

### Risk & VRF
| Instruction | Description |
|---|---|
| `calculate_occr_discount` | Recalculate security discount for user |
| `initialize_vrf_state` | Set up pseudo-VRF state for pool |
| `request_validator_selection` | Randomly assign validators to claim |

---

## Security Properties

- **Checked arithmetic** on every numeric operation — no integer overflow possible
- **PDA-owned vault** — pool funds held by program, not EOA
- **Fraud detection** — validators using "fraud"/"self-hack" keywords trigger full stake slash
- **Anti-backdating** — `joined_at < incident_timestamp` enforced on-chain
- **Claim period enforcement** — incidents outside `claim_period` window rejected
- **Evidence requirements** — non-empty IPFS hash + valid protocol address required
- **Wallet loss proof** — `balance_before > balance_after` enforced on-chain
- **Validator assignment via VRF** — Keccak256(claim_id + pool + timestamp + slot) prevents manipulation
- **Dynamic account realloc** — `ClaimRequest` grows with each validation, no fixed-size ceiling

---

## Known Limitations (In Progress)

- Vault bump seed in emergency payout uses pool bump — fix pending
- `activate_fast_track` missing governance authority check — fix pending  
- Fraud detection is string-based — moving to dedicated `is_fraud_flagged: bool` field
- VRF uses pseudo-randomness — Switchboard VRF integration planned (`switchboard_vrf` field reserved)
- `yield_generation` is a placeholder stub — DeFi yield routing not yet implemented

---

## Getting Started

### Prerequisites
- Rust + Anchor CLI (`anchor --version`)
- Solana CLI (`solana --version`)
- Node.js + Yarn

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

### Test
```bash
anchor test
```

### Deploy to Devnet
```bash
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

---

## Hack Types Supported

| Type | Description |
|---|---|
| `WalletHack` | Private key compromise, unauthorized access |
| `PhishingAttack` | Social engineering, fake sites |
| `BridgeExploit` | Cross-chain bridge vulnerability |
| `FlashLoanAttack` | Flash loan price manipulation |
| `SmartContractExploit` | Protocol-level vulnerability |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Smart Contract | Rust, Anchor Framework |
| Token Standard | SPL Token (USDC) |
| Randomness | Keccak256 pseudo-VRF (Switchboard planned) |
| Test Suite | TypeScript, Mocha/Chai, `@coral-xyz/anchor` |
| Network | Solana (Devnet → Mainnet) |

---

## Roadmap

- [ ] Fix vault bump seed bug
- [ ] Add authority check to `activate_fast_track`
- [ ] Replace string fraud detection with boolean flag
- [ ] Integrate Switchboard VRF for true randomness
- [ ] Implement yield generation (Kamino/MarginFi integration)
- [ ] Frontend (Next.js + Wallet Adapter)
- [ ] Mainnet deployment
- [ ] Security audit

---

## License

MIT

---

*Built on Solana. No KYC. No intermediaries. Your coverage, your keys.*
