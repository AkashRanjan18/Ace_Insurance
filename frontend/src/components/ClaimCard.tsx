"use client";

import { FC } from "react";
import { Clock, CheckCircle, XCircle, Loader, DollarSign } from "lucide-react";
import clsx from "clsx";

type ClaimStatus =
  | "Pending"
  | "UnderValidation"
  | "Approved"
  | "Rejected"
  | "Distributed";

interface ClaimCardProps {
  claimId: string;
  hackType: string;
  amountRequested: number;
  status: ClaimStatus;
  createdAt: string;
  approvals?: number;
  rejections?: number;
}

const STATUS_CONFIG: Record<
  ClaimStatus,
  { label: string; color: string; icon: FC<{ size?: number }> }
> = {
  Pending: { label: "Pending", color: "status-pending", icon: Clock },
  UnderValidation: {
    label: "Under Validation",
    color: "status-pending",
    icon: Loader,
  },
  Approved: { label: "Approved", color: "status-active", icon: CheckCircle },
  Rejected: { label: "Rejected", color: "status-rejected", icon: XCircle },
  Distributed: {
    label: "Paid Out",
    color: "status-distributed",
    icon: DollarSign,
  },
};

const ClaimCard: FC<ClaimCardProps> = ({
  claimId,
  hackType,
  amountRequested,
  status,
  createdAt,
  approvals = 0,
  rejections = 0,
}) => {
  const config = STATUS_CONFIG[status];
  const Icon = config.icon;

  return (
    <div className="rounded-xl border border-ace-border bg-ace-card p-5 hover:border-ace-green/20 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div>
          <p className="text-xs text-ace-muted mb-1 font-mono">
            {claimId.slice(0, 8)}...{claimId.slice(-4)}
          </p>
          <p className="text-white font-semibold">{hackType.replace(/([A-Z])/g, " $1").trim()}</p>
        </div>
        <span
          className={clsx(
            "flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-full",
            config.color
          )}
        >
          <Icon size={12} />
          {config.label}
        </span>
      </div>

      <div className="flex items-center justify-between text-sm">
        <div>
          <p className="text-ace-muted text-xs">Amount Requested</p>
          <p className="text-ace-green font-semibold">
            ${amountRequested.toLocaleString()} USDC
          </p>
        </div>
        {(status === "UnderValidation" || status === "Approved" || status === "Rejected") && (
          <div className="text-right">
            <p className="text-ace-muted text-xs">Votes</p>
            <p className="text-sm">
              <span className="text-green-400">{approvals} ✓</span>
              {" / "}
              <span className="text-red-400">{rejections} ✗</span>
            </p>
          </div>
        )}
        <div className="text-right">
          <p className="text-ace-muted text-xs">Filed</p>
          <p className="text-slate-400 text-xs">{createdAt}</p>
        </div>
      </div>
    </div>
  );
};

export default ClaimCard;
