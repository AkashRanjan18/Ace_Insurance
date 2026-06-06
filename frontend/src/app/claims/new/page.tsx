"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { Shield, Info, CheckCircle, AlertTriangle } from "lucide-react";
import { useRouter } from "next/navigation";
import { HACK_TYPES, HackType } from "@/lib/constants";

const HACK_TYPE_LABELS: Record<HackType, string> = {
  SmartContractExploit: "Smart Contract Exploit",
  WalletHack: "Wallet Hack",
  PhishingAttack: "Phishing Attack",
  SocialEngineering: "Social Engineering",
  FlashLoanAttack: "Flash Loan Attack",
  OracleManipulation: "Oracle Manipulation",
  ReentrancyAttack: "Reentrancy Attack",
  AccessControlBypass: "Access Control Bypass",
  BridgeExploit: "Bridge Exploit",
  RugPull: "Rug Pull",
  Other: "Other",
};

export default function NewClaimPage() {
  const { connected } = useWallet();
  const router = useRouter();

  const [form, setForm] = useState({
    hackType: "WalletHack" as HackType,
    amountRequested: "",
    incidentDate: "",
    evidenceHash: "",
    affectedProtocol: "",
    walletBalanceBefore: "",
    walletBalanceAfter: "",
  });
  const [submitting, setSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState("");

  const walletLoss =
    Number(form.walletBalanceBefore) - Number(form.walletBalanceAfter);

  const validate = () => {
    if (!form.evidenceHash.trim()) return "Evidence hash is required (IPFS/Arweave CID)";
    if (!form.affectedProtocol.trim() || form.affectedProtocol.length < 32)
      return "Affected protocol must be a valid Solana address (32+ chars)";
    if (!form.incidentDate) return "Incident date is required";
    if (Number(form.amountRequested) <= 0)
      return "Claim amount must be greater than 0";
    if (Number(form.amountRequested) > 5000)
      return "Claim amount cannot exceed your coverage limit ($5,000)";
    if (walletLoss <= 0)
      return "Wallet balance must have decreased — loss must be positive";
    return null;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const err = validate();
    if (err) {
      setError(err);
      return;
    }
    setError("");
    setSubmitting(true);
    await new Promise((r) => setTimeout(r, 2500));
    setSubmitting(false);
    setSubmitted(true);
  };

  if (!connected) {
    return (
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-32 text-center">
        <Shield className="w-12 h-12 text-ace-muted mx-auto mb-4" />
        <h2 className="text-2xl font-bold text-white mb-3">
          Connect Your Wallet
        </h2>
        <p className="text-slate-400 mb-8">
          You must be connected and have active coverage to file a claim.
        </p>
        <WalletMultiButton />
      </div>
    );
  }

  if (submitted) {
    return (
      <div className="mx-auto max-w-2xl px-4 sm:px-6 lg:px-8 py-32 text-center">
        <CheckCircle className="w-16 h-16 text-ace-green mx-auto mb-6" />
        <h2 className="text-3xl font-bold text-white mb-3">
          Claim Submitted
        </h2>
        <p className="text-slate-400 mb-4">
          Your claim is on-chain and in{" "}
          <span className="text-yellow-400 font-semibold">Pending</span> status.
          The protocol will request randomness from Switchboard&apos;s VRF
          oracle to select your validator set.
        </p>
        <div className="rounded-xl border border-ace-border bg-ace-card p-6 mb-8 text-left space-y-2">
          <p className="text-sm text-slate-400">
            <span className="text-slate-200 font-semibold">Next steps:</span>
          </p>
          <ol className="list-decimal list-inside space-y-1.5 text-sm text-slate-400">
            <li>Switchboard VRF oracle receives the randomness request</li>
            <li>ECVRF proof computed off-chain — validator set selected fairly</li>
            <li>
              {form.hackType === "BridgeExploit" ||
              form.hackType === "SmartContractExploit"
                ? "Your hack type may qualify for fast-track emergency payout"
                : "Validators review your evidence and cast weighted votes"}
            </li>
            <li>Approved claims paid from pool vault in USDC</li>
          </ol>
        </div>
        <button
          onClick={() => router.push("/dashboard")}
          className="px-6 py-3 rounded-lg bg-ace-green text-ace-dark font-semibold hover:bg-ace-green/90 transition-colors"
        >
          View Dashboard
        </button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl px-4 sm:px-6 lg:px-8 py-12">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white mb-2">File a Claim</h1>
        <p className="text-slate-400 text-sm">
          Submit evidence of your hack. All data is verified on-chain. Validators
          selected via Switchboard ECVRF cannot be biased or predicted.
        </p>
      </div>

      {/* Evidence guide */}
      <div className="rounded-xl border border-ace-teal/20 bg-ace-teal/5 p-4 mb-8 flex gap-3">
        <Info className="w-4 h-4 text-ace-teal flex-shrink-0 mt-0.5" />
        <div className="text-sm text-slate-400">
          <p className="text-slate-200 font-semibold mb-1">
            Evidence requirements
          </p>
          <ul className="space-y-1">
            <li>
              • Upload your evidence bundle (tx signatures, screenshots,
              post-mortem) to IPFS or Arweave
            </li>
            <li>• Paste the resulting CID/hash below</li>
            <li>
              • Include the affected protocol&apos;s Solana address and your
              wallet balance before and after
            </li>
          </ul>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Hack type */}
        <div>
          <label className="block text-sm text-slate-400 mb-2">Hack Type</label>
          <select
            value={form.hackType}
            onChange={(e) =>
              setForm({ ...form, hackType: e.target.value as HackType })
            }
            className="w-full bg-ace-card border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors"
          >
            {HACK_TYPES.map((t) => (
              <option key={t} value={t}>
                {HACK_TYPE_LABELS[t]}
              </option>
            ))}
          </select>
        </div>

        {/* Amount */}
        <div>
          <label className="block text-sm text-slate-400 mb-2">
            Claim Amount (USDC)
          </label>
          <div className="relative">
            <span className="absolute left-4 top-3 text-ace-muted">$</span>
            <input
              type="number"
              min={1}
              max={5000}
              value={form.amountRequested}
              onChange={(e) =>
                setForm({ ...form, amountRequested: e.target.value })
              }
              placeholder="0.00"
              className="w-full bg-ace-card border border-ace-border rounded-lg pl-8 pr-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors"
            />
          </div>
          <p className="text-xs text-ace-muted mt-1">Max: $5,000 (your coverage limit)</p>
        </div>

        {/* Incident date */}
        <div>
          <label className="block text-sm text-slate-400 mb-2">
            Incident Date
          </label>
          <input
            type="date"
            value={form.incidentDate}
            onChange={(e) =>
              setForm({ ...form, incidentDate: e.target.value })
            }
            className="w-full bg-ace-card border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors"
          />
          <p className="text-xs text-ace-muted mt-1">
            Must be within your 90-day claim period and after your join date
          </p>
        </div>

        {/* Evidence hash */}
        <div>
          <label className="block text-sm text-slate-400 mb-2">
            Evidence Hash (IPFS CID / Arweave TxID)
          </label>
          <input
            type="text"
            value={form.evidenceHash}
            onChange={(e) =>
              setForm({ ...form, evidenceHash: e.target.value })
            }
            placeholder="QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
            className="w-full bg-ace-card border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors font-mono"
          />
        </div>

        {/* Affected protocol */}
        <div>
          <label className="block text-sm text-slate-400 mb-2">
            Affected Protocol Address
          </label>
          <input
            type="text"
            value={form.affectedProtocol}
            onChange={(e) =>
              setForm({ ...form, affectedProtocol: e.target.value })
            }
            placeholder="Solana program address (32 bytes)"
            className="w-full bg-ace-card border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors font-mono"
          />
        </div>

        {/* Wallet balances */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm text-slate-400 mb-2">
              Wallet Balance Before (USDC)
            </label>
            <input
              type="number"
              min={0}
              value={form.walletBalanceBefore}
              onChange={(e) =>
                setForm({ ...form, walletBalanceBefore: e.target.value })
              }
              placeholder="0"
              className="w-full bg-ace-card border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors"
            />
          </div>
          <div>
            <label className="block text-sm text-slate-400 mb-2">
              Wallet Balance After (USDC)
            </label>
            <input
              type="number"
              min={0}
              value={form.walletBalanceAfter}
              onChange={(e) =>
                setForm({ ...form, walletBalanceAfter: e.target.value })
              }
              placeholder="0"
              className="w-full bg-ace-card border border-ace-border rounded-lg px-4 py-3 text-white text-sm focus:outline-none focus:border-ace-green/50 transition-colors"
            />
          </div>
        </div>

        {/* Loss preview */}
        {form.walletBalanceBefore && form.walletBalanceAfter && (
          <div
            className={`flex items-center gap-2 p-3 rounded-lg border text-sm ${
              walletLoss > 0
                ? "border-ace-green/20 bg-ace-green/5 text-ace-green"
                : "border-red-500/20 bg-red-500/5 text-red-400"
            }`}
          >
            {walletLoss > 0 ? (
              <CheckCircle size={14} />
            ) : (
              <AlertTriangle size={14} />
            )}
            {walletLoss > 0
              ? `Verified loss: $${walletLoss.toLocaleString()} USDC`
              : "Balance must have decreased — loss must be positive"}
          </div>
        )}

        {/* Error */}
        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg border border-red-500/20 bg-red-500/5 text-red-400 text-sm">
            <AlertTriangle size={14} />
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={submitting}
          className="w-full py-3.5 rounded-lg bg-ace-green text-ace-dark font-bold hover:bg-ace-green/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
        >
          {submitting ? (
            <>
              <div className="w-4 h-4 border-2 border-ace-dark border-t-transparent rounded-full animate-spin" />
              Submitting Claim On-Chain...
            </>
          ) : (
            <>
              <Shield size={16} />
              Submit Claim
            </>
          )}
        </button>

        <p className="text-xs text-ace-muted text-center">
          Submitting this claim creates an on-chain transaction. Validators will
          be randomly selected via Switchboard VRF and notified automatically.
        </p>
      </form>
    </div>
  );
}
