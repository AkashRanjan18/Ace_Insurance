"use client";

import Link from "next/link";
import {
  Shield,
  Zap,
  Users,
  Lock,
  ArrowRight,
  CheckCircle,
  Globe,
  TrendingUp,
} from "lucide-react";
import StatsCard from "@/components/StatsCard";

const PROTOCOL_STATS = [
  { label: "Total Pooled", value: "$1.25M", sub: "USDC in vault", accent: true },
  { label: "Active Members", value: "847", sub: "covered wallets" },
  { label: "Active Claims", value: "12", sub: "under validation" },
  { label: "Claims Paid", value: "$84K", sub: "lifetime payouts" },
];

const FEATURES = [
  {
    icon: Shield,
    title: "Permissionless Coverage",
    desc: "No KYC. No approval queue. Connect your wallet, pay a premium, and you're covered. Any Solana wallet qualifies.",
  },
  {
    icon: Zap,
    title: "Switchboard VRF Selection",
    desc: "Validators are selected via Switchboard ECVRF — cryptographically unbiasable. No validator can manipulate who investigates their own claim.",
  },
  {
    icon: Users,
    title: "Stake-Based Validators",
    desc: "Validators stake SOL to join. Correct votes earn reputation; wrong votes slash stake. Auditor-tier validators carry 3× vote weight.",
  },
  {
    icon: Lock,
    title: "Emergency Fast-Track",
    desc: "For verified protocol exploits, auditors vote to activate emergency payouts. 20% of your claim lands in your wallet within hours, not weeks.",
  },
  {
    icon: TrendingUp,
    title: "OCCR Discount System",
    desc: "Use a multisig and hardware wallet? You get up to 20% off your premium — rewarding users who follow security best practices.",
  },
  {
    icon: Globe,
    title: "Fully On-Chain",
    desc: "Every premium, every claim, every vote lives on Solana. No off-chain adjudicators. No black-box decisions. Fully auditable.",
  },
];

const HOW_IT_WORKS = [
  {
    step: "01",
    title: "Join a Pool",
    desc: "Connect wallet, choose coverage amount, pay USDC premium. Multisig or hardware wallet users get up to 20% discount.",
  },
  {
    step: "02",
    title: "Get Hacked? File a Claim",
    desc: "Submit evidence hash (IPFS/Arweave), incident timestamp, and wallet balance before/after. Everything verified on-chain.",
  },
  {
    step: "03",
    title: "Validators Investigate",
    desc: "Switchboard VRF randomly selects a validator set. They review evidence and cast weighted votes with technical analysis.",
  },
  {
    step: "04",
    title: "Receive Payout",
    desc: "Approved claims receive USDC from the pool vault. For verified large exploits, emergency fast-track delivers 20% immediately.",
  },
];

const HACK_TYPES = [
  "Smart Contract Exploit",
  "Wallet Hack",
  "Phishing Attack",
  "Flash Loan Attack",
  "Bridge Exploit",
  "Oracle Manipulation",
  "Rug Pull",
  "Access Control Bypass",
];

export default function HomePage() {
  return (
    <div className="bg-grid bg-ace-dark">
      {/* Hero */}
      <section className="relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-ace-green/5 via-transparent to-transparent pointer-events-none" />
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 pt-24 pb-20 text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-ace-green/20 bg-ace-green/5 text-ace-green text-xs mb-8">
            <span className="w-1.5 h-1.5 rounded-full bg-ace-green animate-pulse" />
            Live on Solana Devnet
          </div>

          <h1 className="text-4xl sm:text-6xl lg:text-7xl font-bold text-white mb-6 tracking-tight">
            Decentralized Insurance
            <br />
            <span className="text-ace-green glow-text">for DeFi Hacks</span>
          </h1>

          <p className="text-slate-400 text-lg sm:text-xl max-w-2xl mx-auto mb-10 leading-relaxed">
            Pay USDC premiums. Get covered for bridge exploits, wallet compromises,
            flash loan attacks, and more. Claims validated on-chain by a staked
            validator network. No KYC. No intermediaries.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              href="/pool"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-lg bg-ace-green text-ace-dark font-semibold hover:bg-ace-green/90 transition-colors glow-green"
            >
              Get Covered <ArrowRight size={16} />
            </Link>
            <Link
              href="/validators"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-lg border border-ace-border text-slate-300 hover:border-ace-green/30 hover:text-white transition-colors"
            >
              Become a Validator
            </Link>
          </div>

          {/* Program ID */}
          <p className="mt-10 text-xs text-ace-muted font-mono">
            Program ID:{" "}
            <a
              href="https://explorer.solana.com/address/4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU?cluster=devnet"
              target="_blank"
              rel="noopener noreferrer"
              className="text-ace-green/60 hover:text-ace-green transition-colors"
            >
              4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU
            </a>
          </p>
        </div>
      </section>

      {/* Stats */}
      <section className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-16">
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          {PROTOCOL_STATS.map((stat) => (
            <StatsCard key={stat.label} {...stat} />
          ))}
        </div>
      </section>

      {/* How it works */}
      <section className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-16">
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold text-white mb-3">How It Works</h2>
          <p className="text-slate-400 max-w-xl mx-auto">
            From premium payment to payout, everything happens on-chain in four steps.
          </p>
        </div>
        <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {HOW_IT_WORKS.map((step, i) => (
            <div key={step.step} className="relative">
              {i < HOW_IT_WORKS.length - 1 && (
                <div className="hidden lg:block absolute top-6 left-full w-full h-px bg-gradient-to-r from-ace-border to-transparent z-10" />
              )}
              <div className="rounded-xl border border-ace-border bg-ace-card p-6 hover:border-ace-green/20 transition-colors h-full">
                <div className="text-4xl font-bold text-ace-green/20 mb-4 font-mono">
                  {step.step}
                </div>
                <h3 className="text-white font-semibold mb-2">{step.title}</h3>
                <p className="text-slate-400 text-sm leading-relaxed">{step.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Features */}
      <section className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-16">
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold text-white mb-3">Protocol Features</h2>
          <p className="text-slate-400 max-w-xl mx-auto">
            Built for security-first DeFi users who know their risks.
          </p>
        </div>
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {FEATURES.map((feature) => {
            const Icon = feature.icon;
            return (
              <div
                key={feature.title}
                className="rounded-xl border border-ace-border bg-ace-card p-6 hover:border-ace-green/20 transition-colors group"
              >
                <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-ace-green/10 border border-ace-green/20 mb-4 group-hover:bg-ace-green/15 transition-colors">
                  <Icon className="w-5 h-5 text-ace-green" />
                </div>
                <h3 className="text-white font-semibold mb-2">{feature.title}</h3>
                <p className="text-slate-400 text-sm leading-relaxed">{feature.desc}</p>
              </div>
            );
          })}
        </div>
      </section>

      {/* Coverage types */}
      <section className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-16">
        <div className="rounded-2xl border border-ace-border bg-ace-card p-8 sm:p-12">
          <div className="max-w-3xl mx-auto text-center">
            <h2 className="text-2xl font-bold text-white mb-3">
              10 Hack Types Covered
            </h2>
            <p className="text-slate-400 text-sm mb-8">
              Every claim is classified on-chain. Validators are selected with
              expertise matching the hack type.
            </p>
            <div className="flex flex-wrap justify-center gap-2">
              {HACK_TYPES.map((type) => (
                <span
                  key={type}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-full border border-ace-border bg-white/3 text-slate-300 text-xs hover:border-ace-green/30 hover:text-white transition-colors"
                >
                  <CheckCircle className="w-3 h-3 text-ace-green flex-shrink-0" />
                  {type}
                </span>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-20">
        <div className="text-center">
          <h2 className="text-3xl font-bold text-white mb-4">
            Your wallet. Your coverage.
          </h2>
          <p className="text-slate-400 max-w-lg mx-auto mb-8">
            Start with $50/month USDC premium. Get up to $5,000 in coverage. Cancel
            anytime by stopping premium payments.
          </p>
          <Link
            href="/pool"
            className="inline-flex items-center gap-2 px-8 py-4 rounded-xl bg-ace-green text-ace-dark font-bold text-lg hover:bg-ace-green/90 transition-colors glow-green"
          >
            Get Covered Now <ArrowRight size={18} />
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-ace-border">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-8">
          <div className="flex flex-col sm:flex-row justify-between items-center gap-4">
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-ace-green" />
              <span className="text-slate-400 text-sm">
                ACE Insurance — built on Solana
              </span>
            </div>
            <div className="flex items-center gap-6">
              <a
                href="https://github.com/AkashRanjan18/Ace_Insurance"
                target="_blank"
                rel="noopener noreferrer"
                className="text-slate-400 hover:text-white text-sm transition-colors"
              >
                GitHub
              </a>
              <a
                href="https://explorer.solana.com/address/4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU?cluster=devnet"
                target="_blank"
                rel="noopener noreferrer"
                className="text-slate-400 hover:text-white text-sm transition-colors"
              >
                Explorer
              </a>
              <span className="text-slate-600 text-sm">MIT License</span>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
