"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import {
  Shield,
  AlertCircle,
  DollarSign,
  Clock,
  CheckCircle,
  ArrowRight,
} from "lucide-react";
import Link from "next/link";
import ClaimCard from "@/components/ClaimCard";

const MOCK_COVERAGE = {
  coverageAmount: 2500,
  premiumPaid: 45,
  lastPayment: "2026-05-28",
  nextDue: "2026-06-28",
  active: true,
  occrDiscount: 10,
  claimsMade: 1,
};

const MOCK_CLAIMS = [
  {
    claimId: "3xKj9pQmNrVv2yAaLBt5uWd8eHfG1cZq",
    hackType: "WalletHack",
    amountRequested: 800,
    status: "Distributed" as const,
    createdAt: "2026-04-12",
    approvals: 4,
    rejections: 1,
  },
  {
    claimId: "7mFsYhCbDnJpR4oTqXzE6aKwG2iNvU9L",
    hackType: "PhishingAttack",
    amountRequested: 1200,
    status: "UnderValidation" as const,
    createdAt: "2026-06-01",
    approvals: 2,
    rejections: 0,
  },
];

export default function DashboardPage() {
  const { connected, publicKey } = useWallet();
  const [paying, setPaying] = useState(false);
  const [paid, setPaid] = useState(false);

  const handlePayPremium = async () => {
    setPaying(true);
    await new Promise((r) => setTimeout(r, 1800));
    setPaying(false);
    setPaid(true);
  };

  if (!connected) {
    return (
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-32 text-center">
        <Shield className="w-12 h-12 text-ace-muted mx-auto mb-4" />
        <h2 className="text-2xl font-bold text-white mb-3">
          Connect Your Wallet
        </h2>
        <p className="text-slate-400 mb-8">
          Connect your Solana wallet to view your coverage and claims.
        </p>
        <WalletMultiButton />
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-12">
      <div className="flex items-center justify-between mb-10">
        <div>
          <h1 className="text-3xl font-bold text-white mb-1">Dashboard</h1>
          <p className="text-ace-muted text-sm font-mono">
            {publicKey?.toBase58().slice(0, 20)}...
          </p>
        </div>
        <Link
          href="/claims/new"
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-ace-green text-ace-dark font-semibold text-sm hover:bg-ace-green/90 transition-colors"
        >
          File Claim <ArrowRight size={14} />
        </Link>
      </div>

      <div className="grid lg:grid-cols-3 gap-8">
        {/* Coverage card */}
        <div className="lg:col-span-1">
          <div
            className={`rounded-xl border p-6 ${
              MOCK_COVERAGE.active
                ? "border-ace-green/30 bg-ace-green/5 glow-green"
                : "border-red-500/30 bg-red-500/5"
            }`}
          >
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-white font-semibold">My Coverage</h2>
              <span
                className={`flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-full ${
                  MOCK_COVERAGE.active ? "status-active" : "status-rejected"
                }`}
              >
                <span className="w-1.5 h-1.5 rounded-full bg-current animate-pulse" />
                {MOCK_COVERAGE.active ? "Active" : "Lapsed"}
              </span>
            </div>

            <div className="text-4xl font-bold text-ace-green glow-text mb-1">
              ${MOCK_COVERAGE.coverageAmount.toLocaleString()}
            </div>
            <p className="text-slate-400 text-sm mb-6">USDC coverage limit</p>

            <div className="space-y-3 mb-6">
              {[
                {
                  icon: DollarSign,
                  label: "Monthly premium",
                  value: `$${MOCK_COVERAGE.premiumPaid} USDC`,
                },
                {
                  icon: CheckCircle,
                  label: "OCCR discount",
                  value: `${MOCK_COVERAGE.occrDiscount}% off`,
                },
                {
                  icon: Clock,
                  label: "Next due",
                  value: MOCK_COVERAGE.nextDue,
                },
                {
                  icon: AlertCircle,
                  label: "Claims filed",
                  value: `${MOCK_COVERAGE.claimsMade}`,
                },
              ].map((item) => {
                const Icon = item.icon;
                return (
                  <div
                    key={item.label}
                    className="flex items-center justify-between text-sm"
                  >
                    <div className="flex items-center gap-2 text-slate-400">
                      <Icon className="w-3.5 h-3.5" />
                      {item.label}
                    </div>
                    <span className="text-slate-200 font-mono text-xs">
                      {item.value}
                    </span>
                  </div>
                );
              })}
            </div>

            {paid ? (
              <div className="flex items-center justify-center gap-2 py-3 rounded-lg border border-ace-green/30 text-ace-green text-sm">
                <CheckCircle size={16} />
                Premium paid — coverage renewed
              </div>
            ) : (
              <button
                onClick={handlePayPremium}
                disabled={paying}
                className="w-full py-2.5 rounded-lg border border-ace-green/30 text-ace-green text-sm font-semibold hover:bg-ace-green/10 disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
              >
                {paying ? (
                  <>
                    <div className="w-3.5 h-3.5 border-2 border-ace-green border-t-transparent rounded-full animate-spin" />
                    Processing...
                  </>
                ) : (
                  "Pay Premium"
                )}
              </button>
            )}
          </div>
        </div>

        {/* Claims */}
        <div className="lg:col-span-2">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-white font-semibold">My Claims</h2>
            <Link
              href="/claims/new"
              className="text-ace-green text-sm hover:underline"
            >
              + New claim
            </Link>
          </div>

          {MOCK_CLAIMS.length === 0 ? (
            <div className="rounded-xl border border-ace-border bg-ace-card p-12 text-center">
              <Shield className="w-8 h-8 text-ace-muted mx-auto mb-3" />
              <p className="text-slate-400">No claims filed yet.</p>
              <p className="text-slate-500 text-sm mt-1">
                If you&apos;ve been hacked, file a claim to start the validation
                process.
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {MOCK_CLAIMS.map((claim) => (
                <ClaimCard key={claim.claimId} {...claim} />
              ))}
            </div>
          )}

          {/* VRF info panel */}
          <div className="mt-6 rounded-xl border border-ace-border bg-ace-card p-5">
            <div className="flex items-start gap-3">
              <div className="w-8 h-8 rounded-lg bg-ace-teal/10 border border-ace-teal/20 flex items-center justify-center flex-shrink-0">
                <span className="text-ace-teal text-xs font-bold">VRF</span>
              </div>
              <div>
                <h3 className="text-white text-sm font-semibold mb-1">
                  Switchboard ECVRF Validator Selection
                </h3>
                <p className="text-slate-400 text-xs leading-relaxed">
                  When your claim enters validation, validators are selected via
                  Switchboard&apos;s ECVRF oracle — a cryptographic random function
                  that cannot be predicted or manipulated by any on-chain actor.
                  No validator can arrange to be assigned to a claim they have
                  a conflict of interest with.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
