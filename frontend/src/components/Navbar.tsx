"use client";

import { FC, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { Shield, Menu, X } from "lucide-react";
import clsx from "clsx";

const NAV_LINKS = [
  { href: "/", label: "Home" },
  { href: "/pool", label: "Pool" },
  { href: "/dashboard", label: "Dashboard" },
  { href: "/claims/new", label: "File Claim" },
  { href: "/validators", label: "Validators" },
];

const Navbar: FC = () => {
  const pathname = usePathname();
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <nav className="sticky top-0 z-50 border-b border-ace-border bg-ace-dark/80 backdrop-blur-md">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <Link href="/" className="flex items-center gap-2 group">
            <div className="flex items-center justify-center w-8 h-8 rounded-md bg-ace-green/10 border border-ace-green/20 group-hover:bg-ace-green/20 transition-colors">
              <Shield className="w-4 h-4 text-ace-green" />
            </div>
            <span className="text-ace-green font-bold text-lg tracking-tight glow-text">
              ACE
            </span>
            <span className="text-slate-400 text-sm hidden sm:block">
              Insurance
            </span>
          </Link>

          {/* Desktop nav */}
          <div className="hidden md:flex items-center gap-1">
            {NAV_LINKS.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className={clsx(
                  "px-3 py-2 rounded-md text-sm transition-colors",
                  pathname === link.href
                    ? "text-ace-green bg-ace-green/10"
                    : "text-slate-400 hover:text-slate-200 hover:bg-white/5"
                )}
              >
                {link.label}
              </Link>
            ))}
          </div>

          {/* Wallet + mobile toggle */}
          <div className="flex items-center gap-3">
            <div className="hidden sm:block">
              <WalletMultiButton />
            </div>
            <button
              className="md:hidden p-2 text-slate-400 hover:text-slate-200"
              onClick={() => setMobileOpen(!mobileOpen)}
            >
              {mobileOpen ? <X size={20} /> : <Menu size={20} />}
            </button>
          </div>
        </div>
      </div>

      {/* Mobile menu */}
      {mobileOpen && (
        <div className="md:hidden border-t border-ace-border bg-ace-card">
          <div className="px-4 py-3 space-y-1">
            {NAV_LINKS.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileOpen(false)}
                className={clsx(
                  "block px-3 py-2 rounded-md text-sm transition-colors",
                  pathname === link.href
                    ? "text-ace-green bg-ace-green/10"
                    : "text-slate-400 hover:text-slate-200 hover:bg-white/5"
                )}
              >
                {link.label}
              </Link>
            ))}
            <div className="pt-2">
              <WalletMultiButton />
            </div>
          </div>
        </div>
      )}
    </nav>
  );
};

export default Navbar;
