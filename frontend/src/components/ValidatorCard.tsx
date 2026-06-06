"use client";

import { FC } from "react";
import { Star, Shield, TrendingUp } from "lucide-react";
import clsx from "clsx";

interface ValidatorCardProps {
  address: string;
  tier: "Standard" | "Auditor";
  stakeAmount: number;
  reputationScore: number;
  validationsCompleted: number;
  successfulValidations: number;
}

const ValidatorCard: FC<ValidatorCardProps> = ({
  address,
  tier,
  stakeAmount,
  reputationScore,
  validationsCompleted,
  successfulValidations,
}) => {
  const accuracy =
    validationsCompleted > 0
      ? Math.round((successfulValidations / validationsCompleted) * 100)
      : 0;

  const repPercent = Math.round((reputationScore / 10000) * 100);

  return (
    <div
      className={clsx(
        "rounded-xl border p-5 bg-ace-card hover:border-ace-green/20 transition-colors",
        tier === "Auditor"
          ? "border-ace-teal/40 bg-ace-teal/5"
          : "border-ace-border"
      )}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div>
          <p className="text-xs text-ace-muted font-mono mb-1">
            {address.slice(0, 6)}...{address.slice(-4)}
          </p>
          <div className="flex items-center gap-2">
            {tier === "Auditor" ? (
              <Star className="w-4 h-4 text-ace-teal" />
            ) : (
              <Shield className="w-4 h-4 text-ace-muted" />
            )}
            <span
              className={clsx(
                "text-sm font-semibold",
                tier === "Auditor" ? "text-ace-teal" : "text-slate-300"
              )}
            >
              {tier}
            </span>
            {tier === "Auditor" && (
              <span className="text-xs bg-ace-teal/10 border border-ace-teal/20 text-ace-teal px-2 py-0.5 rounded-full">
                3× vote weight
              </span>
            )}
          </div>
        </div>
        <div className="text-right">
          <p className="text-xs text-ace-muted">Stake</p>
          <p className="text-white font-semibold text-sm">{stakeAmount} SOL</p>
        </div>
      </div>

      {/* Reputation bar */}
      <div className="mb-4">
        <div className="flex justify-between text-xs mb-1">
          <span className="text-ace-muted">Reputation</span>
          <span className="text-slate-300">
            {reputationScore.toLocaleString()} / 10,000
          </span>
        </div>
        <div className="w-full bg-ace-border rounded-full h-1.5">
          <div
            className="h-1.5 rounded-full bg-gradient-to-r from-ace-green to-ace-teal transition-all"
            style={{ width: `${repPercent}%` }}
          />
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-3">
        <div className="rounded-lg bg-white/3 p-3">
          <p className="text-xs text-ace-muted mb-1">Validations</p>
          <p className="text-white font-semibold">{validationsCompleted}</p>
        </div>
        <div className="rounded-lg bg-white/3 p-3">
          <div className="flex items-center gap-1 mb-1">
            <TrendingUp className="w-3 h-3 text-ace-green" />
            <p className="text-xs text-ace-muted">Accuracy</p>
          </div>
          <p
            className={clsx(
              "font-semibold",
              accuracy >= 80 ? "text-ace-green" : accuracy >= 60 ? "text-yellow-400" : "text-red-400"
            )}
          >
            {accuracy}%
          </p>
        </div>
      </div>
    </div>
  );
};

export default ValidatorCard;
