import type { Metadata } from "next";
import "./globals.css";
import AppWalletProvider from "@/providers/WalletProvider";
import Navbar from "@/components/Navbar";

export const metadata: Metadata = {
  title: "ACE Insurance — Decentralized Hack Insurance on Solana",
  description:
    "Permissionless on-chain insurance for DeFi hacks. Pay USDC premiums, get covered, file claims — all governed by smart contracts. No KYC, no intermediaries.",
  keywords: ["solana", "defi", "insurance", "hack", "web3"],
  openGraph: {
    title: "ACE Insurance",
    description: "Decentralized hack insurance on Solana",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <AppWalletProvider>
          <div className="min-h-screen bg-ace-dark">
            <Navbar />
            <main>{children}</main>
          </div>
        </AppWalletProvider>
      </body>
    </html>
  );
}
