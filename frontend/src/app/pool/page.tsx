"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import {
  Shield,
  Users,
  DollarSign,
  Clock,
  CheckCircle,
  Info,
} from "lucide-react";
import StatsCard from "@/components/StatsCard";

const POOL_INFO = {
  premiumAmount: 50,
  coverageAmount: 5000,
  minValidators: 3,
  claimPeriod: "90 days",
  totalPooled: 1250000,
  totalMembers: 847,
  activeClaims: 12,
  apr: "4.2%",
};

export default function PoolPage() {
  const { connected, publicKey } = useWallet();
  const [coverageAmount, setCoverageAmount] = useState(2500);
  const [useMultisig, setUseMultisig] = useState(false);
  const [useHardwareWallet, setUseHardwareWallet] = useState(false);
  const [joining, setJoining] = useState(false);
  const [joined, setJoined] = useState(false);

  const discount = (useMultisig ? 10 : 0) + (useHardwareWallet ? 10 : 0);
  const effectivePremium = Math.round(
    POOL_INFO.premiumAmount * (1 - discount / 100)
  );

  const handleJoin = async () => {
    if (!connected) return;
    setJoining(true);
    await new Promise((r) => setTimeout(r, 2000));
    setJoining(false);
    setJoined(true);
  };

  return (
    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-12">
      <div className="mb-10">
        <h1 className="text-3xl font-bold text-white mb-2">Insurance Pool</h1>
        <p className="text-slate-400">
          Join the ACE Insurance pool and get covered for on-chain hack losses.
        </p>
      </div>

      {/* Pool stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
        <StatsCard
          label="Total Pooled"
          value={`$${(POOL_INFO.totalPooled / 1000).toFixed(0)}K`}
          sub="USDC in vault"
          icon={<DollarSign size={20} />}
          accent
        />
        <StatsCard
          label="Members"
          value={POOL_INFO.totalMembers.toString()}
          sub="active policies"
          icon={<Users size={20} />}
        />
        <StatsCard
          label="Base Premium"
          value={`$${POOL_INFO.premiumAmount}`}
          sub="per month (USDC)"
          icon={<Shield size={20} />}
        />
        <StatsCard
          label="Max Coverage"
          value={`$${POOL_INFO.coverageAmount.toLocaleString()}`}
          sub="per policy"
          icon={<Clock size={20} />}
        />
      </div>

      <div className="grid lg:grid-cols-2 gap-8">
        {/* Pool details */}
        <div className="space-y-4">
          <div className="rounded-xl border border-ace-border bg-ace-card p-6">
            <h2 className="text-white font-semibold mb-4">Pool Parameters</h2>
            <div className="space-y-3">
              {[
                { label: "Minimum Validators", value: `${POOL_INFO.minValidators} (selected via ECVRF)` },
                { label: "Claim Period", value: POOL_INFO.claimPeriod },
                { label: "Active Claims", value: POOL_INFO.activeClaims.toString() },
                {
                  label: "Yield (idle funds)",
                  value: POOL_INFO.apr + " estimated APR",
                },
              ].map((item) => (
                <div
                  key={item.label}
                  className="flex justify-between items-center py-2 border-b border-ace-border/50 last:border-0"
                >
                  <span className="text-slate-400 text-sm">{item.label}</span>
                  <span className="text-slate-200 text-sm font-mono">
                    {item.value}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Coverage types */}
          <div className="rounded-xl border border-ace-border bg-ace-card p-6">
            <h2 className="text-white font-semibold mb-4">What&apos;s Covered</h2>
            <div className="grid grid-cols-2 gap-2">
              {[
                "Wallet Hack",
                "Smart Contract Exploit",
                "Phishing Attack",
                "Bridge Exploit",
                "Flash Loan Attack",
                "Oracle Manipulation",
                "Rug Pull",
                "Access Control Bypass",
              ].map((type) => (
                <div
                  key={type}
                  className="flex items-center gap-2 text-sm text-slate-400"
                >
                  <CheckCircle className="w-3.5 h-3.5 text-ace-green flex-shrink-0" />
                  {type}
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Join form */}
        <div>
          {joined ? (
            <div className="rounded-xl border border-ace-green/30 bg-ace-green/5 glow-green p-8 text-center">
              <CheckCircle className="w-12 h-12 text-ace-green mx-auto mb-4" />
              <h3 className="text-white text-xl font-bold mb-2">
                You&apos;re Covered!
              </h3>
              <p className="text-slate-400 mb-4">
                Coverage of ${coverageAmount.toLocaleString()} USDC is now active for your wallet.
              </p>
              <p className="text-xs text-ace-muted font-mono">
                {publicKey?.toBase58().slice(0, 16)}...
              </p>
            </div>
          ) : (
            <div className="rounded-xl border border-ace-border bg-ace-card p-6">
              <h2 className="text-white font-semibold mb-6">Get Covered</h2>

              {!connected && (
                <div className="mb-6 p-4 rounded-lg border border-ace-border bg-white/3 text-center">
                  <p className="text-slate-400 text-sm mb-3">
                    Connect your wallet to join the pool
                  </p>
                  <WalletMultiButton />
                </div>
              )}

              {/* Coverage slider */}
              <div className="mb-6">
                <div className="flex justify-between mb-2">
                  <label className="text-sm text-slate-400">
                    Coverage Amount
                  </label>
                  <span className="text-white font-semibold font-mono">
                    ${coverageAmount.toLocaleString()} USDC
                  </span>
                </div>
                <input
                  type="range"
                  min={500}
                  max={5000}
                  step={500}
                  value={coverageAmount}
                  onChange={(e) => setCoverageAmount(Number(e.target.value))}
                  className="w-full h-2 bg-ace-border rounded-full appearance-none cursor-pointer accent-[#00ff87]"
                />
                <div className="flex justify-between text-xs text-ace-muted mt-1">
                  <span>$500</span>
                  <span>$5,000</span>
                </div>
              </div>

              {/* OCCR security posture */}
              <div className="mb-6">
                <div className="flex items-center gap-2 mb-3">
                  <label className="text-sm text-slate-400">
                    Security Practices (OCCR Discount)
                  </label>
                  <Info className="w-3.5 h-3.5 text-ace-muted" />
                </div>
                <div className="space-y-2">
                  {[
                    {
                      id: "multisig",
                      label: "I use a multisig wallet",
                      state: useMultisig,
                      setter: setUseMultisig,
                      discount: "−10%",
                    },
                    {
                      id: "hardware",
                      label: "I use a hardware wallet",
                      state: useHardwareWallet,
                      setter: setUseHardwareWallet,
                      discount: "−10%",
                    },
                  ].map((item) => (
                    <label
                      key={item.id}
                      className="flex items-center justify-between p-3 rounded-lg border border-ace-border hover:border-ace-green/20 cursor-pointer transition-colors"
                    >
                      <div className="flex items-center gap-3">
                        <input
                          type="checkbox"
                          checked={item.state}
                          onChange={(e) => item.setter(e.target.checked)}
                          className="w-4 h-4 accent-[#00ff87]"
                        />
                        <span className="text-sm text-slate-300">
                          {item.label}
                        </span>
                      </div>
                      <span className="text-xs text-ace-green font-mono">
                        {item.discount}
                      </span>
                    </label>
                  ))}
                </div>
              </div>

              {/* Premium summary */}
              <div className="rounded-lg bg-white/3 border border-ace-border p-4 mb-6">
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">Base premium</span>
                    <span className="text-slate-300">
                      ${POOL_INFO.premiumAmount}/mo
                    </span>
                  </div>
                  {discount > 0 && (
                    <div className="flex justify-between text-sm">
                      <span className="text-slate-400">OCCR discount</span>
                      <span className="text-ace-green">−{discount}%</span>
                    </div>
                  )}
                  <div className="flex justify-between text-sm pt-2 border-t border-ace-border font-semibold">
                    <span className="text-slate-200">Monthly premium</span>
                    <span className="text-white">${effectivePremium} USDC</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">Coverage</span>
                    <span className="text-ace-green">
                      ${coverageAmount.toLocaleString()} USDC
                    </span>
                  </div>
                </div>
              </div>

              <button
                onClick={handleJoin}
                disabled={!connected || joining}
                className="w-full py-3 px-6 rounded-lg bg-ace-green text-ace-dark font-bold hover:bg-ace-green/90 disabled:opacity-40 disabled:cursor-not-allowed transition-colors glow-green flex items-center justify-center gap-2"
              >
                {joining ? (
                  <>
                    <div className="w-4 h-4 border-2 border-ace-dark border-t-transparent rounded-full animate-spin" />
                    Joining Pool...
                  </>
                ) : (
                  <>
                    <Shield size={16} />
                    Join Pool & Get Covered
                  </>
                )}
              </button>

              <p className="text-xs text-ace-muted text-center mt-3">
                Premium debited monthly from your wallet. Cancel by stopping payments.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
