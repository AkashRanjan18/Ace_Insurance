"use client";

import { FC, ReactNode } from "react";
import clsx from "clsx";

interface StatsCardProps {
  label: string;
  value: string | number;
  sub?: string;
  icon?: ReactNode;
  accent?: boolean;
}

const StatsCard: FC<StatsCardProps> = ({ label, value, sub, icon, accent }) => {
  return (
    <div
      className={clsx(
        "rounded-xl border p-5 bg-ace-card transition-all hover:border-ace-green/30",
        accent ? "border-ace-green/30 glow-green" : "border-ace-border"
      )}
    >
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs text-ace-muted uppercase tracking-widest mb-1">
            {label}
          </p>
          <p
            className={clsx(
              "text-2xl font-bold",
              accent ? "text-ace-green glow-text" : "text-white"
            )}
          >
            {value}
          </p>
          {sub && <p className="text-xs text-slate-500 mt-1">{sub}</p>}
        </div>
        {icon && (
          <div className="text-ace-muted opacity-60">{icon}</div>
        )}
      </div>
    </div>
  );
};

export default StatsCard;
