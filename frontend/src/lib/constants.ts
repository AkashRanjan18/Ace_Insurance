import { PublicKey } from "@solana/web3.js";
import { clusterApiUrl } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU"
);

export const NETWORK = clusterApiUrl("devnet");

export const SWITCHBOARD_PROGRAM_ID = new PublicKey(
  "2TfB33aLaneQb5TNVs2qSRKwFQbk5b18bBhsmLXrg3M7"
);

export const MOCK_POOL_STATS = {
  totalPooled: 1_250_000,
  totalMembers: 847,
  activeClaims: 12,
  premiumAmount: 50,
  coverageAmount: 5000,
  minValidators: 3,
};

export const HACK_TYPES = [
  "SmartContractExploit",
  "WalletHack",
  "PhishingAttack",
  "SocialEngineering",
  "FlashLoanAttack",
  "OracleManipulation",
  "ReentrancyAttack",
  "AccessControlBypass",
  "BridgeExploit",
  "RugPull",
  "Other",
] as const;

export type HackType = (typeof HACK_TYPES)[number];
