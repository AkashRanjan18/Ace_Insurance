"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import {
  Star,
  Shield,
  TrendingUp,
  CheckCircle,
  XCircle,
  AlertTriangle,
} from "lucide-react";
import ValidatorCard from "@/components/ValidatorCard";
import StatsCard from "@/components/StatsCard";

const MOCK_VALIDATORS = [
  {
    address: "7mFsYhCbDnJpR4oTqXzE6aKwG2iNvU9L",
    tier: "Auditor" as const,
    stakeAmount: 5.5,
    reputationScore: 9200,
    validationsCompleted: 48,
    successfulValidations: 46,
  },
  {
    address: "3xKj9pQmNrVv2yAaLBt5uWd8eHfG1cZq",
    tier: "Auditor" as const,
    stakeAmount: 3.2,
    reputationScore: 8500,
    validationsCompleted: 31,
    successfulValidations: 29,
  },
  {
    address: "9pRsLmWdGfHkY7cJbNqVtEzXoAuI3nBx",
    tier: "Standard" as const,
    stakeAmount: 1.0,
    reputationScore: 7100,
    validationsCompleted: 19,
    successfulValidations: 16,
  },
  {
    address: "5nQaKvEcTjMrY2pHsGwXfBdI8uZ4lC6o",
    tier: "Standard" as const,
    stakeAmount: 0.5,
    reputationScore: 6300,
    validationsCompleted: 11,
    successfulValidations: 9,
  },
];

const PENDING_CLAIMS = [
  {
    claimId: "7mFsYhCbDnJpR4oTqXzE6aKwG2iNvU9L",
    hackType: "PhishingAttack",
    amount: 1200,
    daysLeft: 3,
    approvals: 2,
    rejections: 0,
    required: 5,
  },
];

export default function ValidatorsPage() {
  const { connected } = useWallet();
  const [stakeAmount, setStakeAmount] = useState("1.0");
  const [staking, setStaking] = useState(false);
  const [staked, setStaked] = useState(false);
  const [validating, setValidating] = useState<string | null>(null);
  const [votes, setVotes] = useState<Record<string, boolean>>({});

  const handleStake = async () => {
    if (Number(stakeAmount) < 0.1) return;
    setStaking(true);
    await new Promise((r) => setTimeout(r, 2000));
    setStaking(false);
    setStaked(true);
  };

  const handleVote = async (claimId: string, approve: boolean) => {
    setValidating(claimId);
    await new Promise((r) => setTimeout(r, 1800));
    setValidating(null);
    setVotes({ ...votes, [claimId]: approve });
  };

  return (
    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-12">
      <div className="mb-10">
        <h1 className="text-3xl font-bold text-white mb-2">
          Validator Dashboard
        </h1>
        <p className="text-slate-400">
          Stake SOL to become a validator. Investigate claims, vote correctly,
          earn reputation. Auditor-tier validators carry 3× vote weight.
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
        <StatsCard label="Total Validators" value="24" sub="active this period" />
        <StatsCard
          label="Auditors"
          value="6"
          sub="verified security researchers"
          accent
        />
        <StatsCard label="Claims Validated" value="183" sub="all time" />
        <StatsCard label="Avg Accuracy" value="91%" sub="majority vote alignment" />
      </div>

      <div className="grid lg:grid-cols-3 gap-8">
        {/* Stake to validate */}
        <div>
          <div className="rounded-xl border border-ace-border bg-ace-card p-6 mb-6">
            <h2 className="text-white font-semibold mb-1">Become a Validator</h2>
            <p className="text-slate-400 text-sm mb-6">
              Stake SOL as collateral. Earn reputation by voting with the majority.
              Reach 8,000 rep to be promoted to Auditor tier.
            </p>

            {!connected && (
              <div className="mb-4 text-center">
                <WalletMultiButton />
              </div>
            )}

            <div className="mb-4">
              <label className="block text-sm text-slate-400 mb-2">
                Stake Amount (SOL)
              </label>
              <input
                type="number"
                min={0.1}
                step={0.1}
                value={stakeAmount}
                onChange={(e) => setStakeAmount(e.target.value)}
                className="w-full bg-ace-dark border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors"
              />
              <p className="text-xs text-ace-muted mt-1">Minimum: 0.1 SOL</p>
            </div>

            {/* Mechanics */}
            <div className="rounded-lg bg-white/3 border border-ace-border p-4 mb-5 space-y-2 text-xs text-slate-400">
              <p className="text-slate-200 font-semibold text-sm">
                Reputation mechanics
              </p>
              <div className="flex items-center gap-2">
                <TrendingUp className="w-3.5 h-3.5 text-ace-green" />
                Voted with majority → +100 rep
              </div>
              <div className="flex items-center gap-2">
                <AlertTriangle className="w-3.5 h-3.5 text-yellow-400" />
                Voted against majority → −200 rep, −6% stake slashed
              </div>
              <div className="flex items-center gap-2">
                <XCircle className="w-3.5 h-3.5 text-red-400" />
                Fraud flagged on rejected claim → full stake slash
              </div>
            </div>

            {staked ? (
              <div className="flex items-center justify-center gap-2 py-3 rounded-lg border border-ace-green/30 text-ace-green text-sm">
                <CheckCircle size={16} />
                Staked — you&apos;re now a validator
              </div>
            ) : (
              <button
                onClick={handleStake}
                disabled={!connected || staking || Number(stakeAmount) < 0.1}
                className="w-full py-3 rounded-lg bg-ace-green text-ace-dark font-bold hover:bg-ace-green/90 disabled:opacity-40 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
              >
                {staking ? (
                  <>
                    <div className="w-4 h-4 border-2 border-ace-dark border-t-transparent rounded-full animate-spin" />
                    Staking...
                  </>
                ) : (
                  <>
                    <Shield size={16} />
                    Stake & Register
                  </>
                )}
              </button>
            )}
          </div>

          {/* VRF info */}
          <div className="rounded-xl border border-ace-teal/20 bg-ace-teal/5 p-5">
            <div className="flex items-center gap-2 mb-3">
              <span className="w-6 h-6 rounded bg-ace-teal/10 border border-ace-teal/20 flex items-center justify-center text-ace-teal text-xs font-bold">
                VRF
              </span>
              <h3 className="text-white text-sm font-semibold">
                How validators are selected
              </h3>
            </div>
            <p className="text-slate-400 text-xs leading-relaxed">
              When a claim needs investigation, the protocol requests randomness
              from the Switchboard ECVRF oracle. The oracle computes a proof
              off-chain using a secret key — no on-chain actor can predict or
              bias the result. Validators are assigned via a Fisher-Yates
              shuffle seeded by the VRF output.
            </p>
          </div>
        </div>

        {/* Active claims to validate */}
        <div className="lg:col-span-2">
          <h2 className="text-white font-semibold mb-4">
            Claims Awaiting Validation
          </h2>

          {PENDING_CLAIMS.length === 0 ? (
            <div className="rounded-xl border border-ace-border bg-ace-card p-12 text-center">
              <Shield className="w-8 h-8 text-ace-muted mx-auto mb-3" />
              <p className="text-slate-400">No claims assigned to you.</p>
            </div>
          ) : (
            <div className="space-y-4">
              {PENDING_CLAIMS.map((claim) => (
                <div
                  key={claim.claimId}
                  className="rounded-xl border border-ace-border bg-ace-card p-6"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div>
                      <p className="text-xs text-ace-muted font-mono mb-1">
                        {claim.claimId.slice(0, 8)}...
                      </p>
                      <p className="text-white font-semibold">
                        {claim.hackType.replace(/([A-Z])/g, " $1").trim()}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="text-ace-green font-semibold">
                        ${claim.amount.toLocaleString()} USDC
                      </p>
                      <p className="text-xs text-ace-muted">
                        {claim.daysLeft}d remaining
                      </p>
                    </div>
                  </div>

                  {/* Vote progress */}
                  <div className="mb-5">
                    <div className="flex justify-between text-xs mb-2">
                      <span className="text-slate-400">Votes</span>
                      <span className="text-slate-300">
                        <span className="text-green-400">
                          {claim.approvals} approved
                        </span>{" "}
                        /{" "}
                        <span className="text-red-400">
                          {claim.rejections} rejected
                        </span>{" "}
                        / {claim.required} needed
                      </span>
                    </div>
                    <div className="w-full bg-ace-border rounded-full h-1.5">
                      <div
                        className="h-1.5 rounded-full bg-ace-green transition-all"
                        style={{
                          width: `${(claim.approvals / claim.required) * 100}%`,
                        }}
                      />
                    </div>
                  </div>

                  {votes[claim.claimId] !== undefined ? (
                    <div
                      className={`flex items-center justify-center gap-2 py-2.5 rounded-lg border text-sm font-semibold ${
                        votes[claim.claimId]
                          ? "border-ace-green/30 text-ace-green"
                          : "border-red-500/30 text-red-400"
                      }`}
                    >
                      {votes[claim.claimId] ? (
                        <CheckCircle size={14} />
                      ) : (
                        <XCircle size={14} />
                      )}
                      Vote submitted —{" "}
                      {votes[claim.claimId] ? "Approved" : "Rejected"}
                    </div>
                  ) : (
                    <div className="grid grid-cols-2 gap-3">
                      <button
                        onClick={() => handleVote(claim.claimId, true)}
                        disabled={validating === claim.claimId || !connected}
                        className="py-2.5 rounded-lg border border-ace-green/30 text-ace-green font-semibold text-sm hover:bg-ace-green/10 disabled:opacity-40 transition-colors flex items-center justify-center gap-2"
                      >
                        {validating === claim.claimId ? (
                          <div className="w-3.5 h-3.5 border-2 border-ace-green border-t-transparent rounded-full animate-spin" />
                        ) : (
                          <CheckCircle size={14} />
                        )}
                        Approve
                      </button>
                      <button
                        onClick={() => handleVote(claim.claimId, false)}
                        disabled={validating === claim.claimId || !connected}
                        className="py-2.5 rounded-lg border border-red-500/30 text-red-400 font-semibold text-sm hover:bg-red-500/10 disabled:opacity-40 transition-colors flex items-center justify-center gap-2"
                      >
                        {validating === claim.claimId ? (
                          <div className="w-3.5 h-3.5 border-2 border-red-400 border-t-transparent rounded-full animate-spin" />
                        ) : (
                          <XCircle size={14} />
                        )}
                        Reject
                      </button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Leaderboard */}
          <div className="mt-8">
            <div className="flex items-center gap-2 mb-4">
              <Star className="w-4 h-4 text-ace-teal" />
              <h2 className="text-white font-semibold">Validator Leaderboard</h2>
            </div>
            <div className="grid sm:grid-cols-2 gap-4">
              {MOCK_VALIDATORS.map((v) => (
                <ValidatorCard key={v.address} {...v} />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
