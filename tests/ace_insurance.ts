import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  SendTransactionError,
} from "@solana/web3.js";
import { expect } from "chai";
import { AceInsurance } from "../target/types/ace_insurance";

/** Anchor custom error codes from ace_insurance::errors::AceError */
const ERR = {
  InvalidPremiumAmount: 6000,
  InvalidCoverageAmount: 6001,
  InsufficientValidators: 6002,
  InvalidClaimPeriod: 6003,
  ExcessiveCoverageAmount: 6004,
  ExcessiveClaimAmount: 6006,
  EmptyEvidenceHash: 6018,
  InvalidProtocolAddress: 6019,
  InvalidWalletLoss: 6020,
  InsufficientStake: 6011,
} as const;

const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

const PREMIUM = new BN(1_000_000);
const COVERAGE = new BN(10_000_000);
const MIN_VALIDATORS = 3;
const CLAIM_PERIOD = new BN(30 * 24 * 60 * 60);
const MIN_STAKE = 100_000_000;

function getAta(mint: PublicKey, owner: PublicKey): PublicKey {
  const [ata] = PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  return ata;
}

async function createMint(
  connection: Connection,
  payer: Keypair,
  authority: PublicKey,
  decimals = 6
): Promise<PublicKey> {
  const mint = Keypair.generate();
  const lamports = await connection.getMinimumBalanceForRentExemption(82);
  const initData = Buffer.alloc(67);
  initData.writeUInt8(0, 0);
  initData.writeUInt8(decimals, 1);
  authority.toBuffer().copy(initData, 2);
  initData.writeUInt32LE(0, 34);

  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mint.publicKey,
      lamports,
      space: 82,
      programId: TOKEN_PROGRAM_ID,
    }),
    new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: mint.publicKey, isSigner: false, isWritable: true },
        { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      ],
      data: initData,
    })
  );
  await sendAndConfirmTransaction(connection, tx, [payer, mint]);
  return mint.publicKey;
}

async function createAta(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {
  const ata = getAta(mint, owner);
  const info = await connection.getAccountInfo(ata);
  if (info) return ata;

  const tx = new Transaction().add(
    new TransactionInstruction({
      programId: ASSOCIATED_TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: ata, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: false, isWritable: false },
        { pubkey: mint, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      data: Buffer.from([1]),
    })
  );
  await sendAndConfirmTransaction(connection, tx, [payer]);
  return ata;
}

async function mintTokens(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  destination: PublicKey,
  authority: Keypair,
  amount: number
): Promise<void> {
  const data = Buffer.alloc(9);
  data.writeUInt8(7, 0);
  data.writeBigUInt64LE(BigInt(amount), 1);
  const tx = new Transaction().add(
    new TransactionInstruction({
      programId: TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: mint, isSigner: false, isWritable: true },
        { pubkey: destination, isSigner: false, isWritable: true },
        { pubkey: authority.publicKey, isSigner: true, isWritable: false },
      ],
      data,
    })
  );
  await sendAndConfirmTransaction(connection, tx, [payer, authority]);
}

async function getTokenBalance(
  connection: Connection,
  ata: PublicKey
): Promise<bigint> {
  const acc = await connection.getAccountInfo(ata);
  if (!acc) // Replace: return 0n;
return BigInt(0);
  return acc.data.readBigUInt64LE(64);
}

function expectAceError(err: unknown, code: number): void {
  const e = err as {
    error?: { errorCode?: { number: number } };
    logs?: string[];
    message?: string;
  };
  if (e?.error?.errorCode?.number === code) {
    return;
  }
  const logs =
    e.logs ??
    (err instanceof SendTransactionError ? err.logs : undefined) ??
    [];
  const blob = [...logs, e.message ?? String(err)].join("\n");
  const hex = code.toString(16);
  const matched =
    blob.includes(`Error Number: ${code}`) ||
    blob.includes(`Custom program error: ${code}`) ||
    blob.includes(`custom program error: 0x${hex}`);
  expect(matched, `expected AceError ${code} in:\n${blob}`).to.be.true;
}

async function currentUnixTs(connection: Connection): Promise<BN> {
  const clock = await connection.getAccountInfo(SYSVAR_CLOCK_PUBKEY);
  return new BN(clock!.data.readBigInt64LE(32).toString());
}

let claimNonce = 0;

function claimPda(
  claimant: PublicKey,
  poolPk: PublicKey,
  unixTs: BN,
  programId: PublicKey
): PublicKey {
  // Increment the nonce every time this function is called
  claimNonce++; 
  
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("claim"),
      claimant.toBuffer(),
      poolPk.toBuffer(),
      // Add the nonce directly to the timestamp value to force a completely new PDA address
      unixTs.add(new BN(claimNonce)).toArrayLike(Buffer, "le", 8),
    ],
    programId
  )[0];
}

function poolPda(authority: PublicKey, programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), authority.toBuffer()],
    programId
  )[0];
}

function vaultPda(pool: PublicKey, programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), pool.toBuffer()],
    programId
  )[0];
}

function coveragePda(
  user: PublicKey,
  pool: PublicKey,
  programId: PublicKey
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("coverage"), user.toBuffer(), pool.toBuffer()],
    programId
  )[0];
}

function claimStatusKey(status: object): string {
  return Object.keys(status)[0];
}

describe("ace_insurance", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.aceInsurance as Program<AceInsurance>;
  const connection = provider.connection;

  const authority = (provider.wallet as anchor.Wallet).payer;
  let usdcMint: PublicKey;
  let pool: PublicKey;
  let poolVault: PublicKey;
  let authorityAta: PublicKey;

  const member = Keypair.generate();
  let memberAta: PublicKey;
  let memberCoverage: PublicKey;
  let submittedClaim: PublicKey;
  let validatorStakePool: PublicKey;

  before(async () => {
    const sig = await connection.requestAirdrop(
      member.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sig, "confirmed");

    usdcMint = await createMint(connection, authority, authority.publicKey);
    authorityAta = await createAta(connection, authority, usdcMint, authority.publicKey);
    memberAta = await createAta(connection, authority, usdcMint, member.publicKey);
    await mintTokens(connection, authority, usdcMint, authorityAta, authority, 100_000_000);
    await mintTokens(connection, authority, usdcMint, memberAta, authority, 50_000_000);

    pool = poolPda(authority.publicKey, program.programId);
    poolVault = vaultPda(pool, program.programId);

    await program.methods
      .initializePool(PREMIUM, COVERAGE, MIN_VALIDATORS, CLAIM_PERIOD)
      .accountsStrict({
        pool,
        poolVault,
        usdcMint,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    memberCoverage = coveragePda(member.publicKey, pool, program.programId);
    await program.methods
      .joinPool(COVERAGE, true, true)
      .accountsStrict({
        pool,
        userCoverage: memberCoverage,
        poolVault,
        userTokenAccount: memberAta,
        user: member.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([member])
      .rpc();

    [validatorStakePool] = PublicKey.findProgramAddressSync(
      [Buffer.from("validator_stake"), pool.toBuffer()],
      program.programId
    );
    await program.methods
      .initializeValidatorStake()
      .accountsStrict({
        validatorStakePool,
        pool,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  });

  it("initializes pool with correct on-chain configuration", async () => {
    const account = await program.account.insurancePool.fetch(pool);
    expect(account.authority.equals(authority.publicKey)).to.be.true;
    expect(account.premiumAmount.eq(PREMIUM)).to.be.true;
    expect(account.coverageAmount.eq(COVERAGE)).to.be.true;
    expect(account.minValidators).to.equal(MIN_VALIDATORS);
    expect(account.claimPeriod.eq(CLAIM_PERIOD)).to.be.true;
    expect(account.totalMembers).to.equal(1);
    expect(account.fastTrackActive).to.be.false;
    expect(account.vault.equals(poolVault)).to.be.true;
  });

  it("rejects pool init when premium is zero", async () => {
    const rogue = Keypair.generate();
    const roguePool = poolPda(rogue.publicKey, program.programId);
    const rogueVault = vaultPda(roguePool, program.programId);
    const sig = await connection.requestAirdrop(rogue.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");

    try {
      await program.methods
        .initializePool(new BN(0), COVERAGE, MIN_VALIDATORS, CLAIM_PERIOD)
        .accountsStrict({
          pool: roguePool,
          poolVault: rogueVault,
          usdcMint,
          authority: rogue.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([rogue])
        .rpc();
      expect.fail("expected InvalidPremiumAmount");
    } catch (e) {
      expectAceError(e, ERR.InvalidPremiumAmount);
    }
  });

  it("rejects pool init when coverage does not exceed premium", async () => {
    const rogue = Keypair.generate();
    const roguePool = poolPda(rogue.publicKey, program.programId);
    const rogueVault = vaultPda(roguePool, program.programId);
    const sig = await connection.requestAirdrop(rogue.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");

    try {
      await program.methods
        .initializePool(PREMIUM, PREMIUM, MIN_VALIDATORS, CLAIM_PERIOD)
        .accountsStrict({
          pool: roguePool,
          poolVault: rogueVault,
          usdcMint,
          authority: rogue.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([rogue])
        .rpc();
      expect.fail("expected InvalidCoverageAmount");
    } catch (e) {
      expectAceError(e, ERR.InvalidCoverageAmount);
    }
  });

  it("rejects pool init with fewer than 3 validators required", async () => {
    const rogue = Keypair.generate();
    const roguePool = poolPda(rogue.publicKey, program.programId);
    const rogueVault = vaultPda(roguePool, program.programId);
    const sig = await connection.requestAirdrop(rogue.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");

    try {
      await program.methods
        .initializePool(PREMIUM, COVERAGE, 2, CLAIM_PERIOD)
        .accountsStrict({
          pool: roguePool,
          poolVault: rogueVault,
          usdcMint,
          authority: rogue.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([rogue])
        .rpc();
      expect.fail("expected InsufficientValidators");
    } catch (e) {
      expectAceError(e, ERR.InsufficientValidators);
    }
  });

  it("rejects pool init with zero claim period", async () => {
    const rogue = Keypair.generate();
    const roguePool = poolPda(rogue.publicKey, program.programId);
    const rogueVault = vaultPda(roguePool, program.programId);
    const sig = await connection.requestAirdrop(rogue.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");

    try {
      await program.methods
        .initializePool(PREMIUM, COVERAGE, MIN_VALIDATORS, new BN(0))
        .accountsStrict({
          pool: roguePool,
          poolVault: rogueVault,
          usdcMint,
          authority: rogue.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([rogue])
        .rpc();
      expect.fail("expected InvalidClaimPeriod");
    } catch (e) {
      expectAceError(e, ERR.InvalidClaimPeriod);
    }
  });

  it("applies 20% OCCR discount when member uses multisig and hardware wallet", async () => {
    const cov = await program.account.userCoverage.fetch(memberCoverage);
    expect(cov.occrDiscount).to.equal(20);
    expect(cov.usesMultisig).to.be.true;
    expect(cov.usesHardwareWallet).to.be.true;
    expect(cov.coverageActive).to.be.true;
    expect(cov.coverageAmount.eq(COVERAGE)).to.be.true;
  });

  it("rejects join when requested coverage exceeds pool maximum", async () => {
    const user = Keypair.generate();
    const sig = await connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");
    const ata = await createAta(connection, authority, usdcMint, user.publicKey);
    await mintTokens(connection, authority, usdcMint, ata, authority, 5_000_000);
    const cov = coveragePda(user.publicKey, pool, program.programId);

    try {
      await program.methods
        .joinPool(COVERAGE.add(new BN(1)), false, false)
        .accountsStrict({
          pool,
          userCoverage: cov,
          poolVault,
          userTokenAccount: ata,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
      expect.fail("expected ExcessiveCoverageAmount");
    } catch (e) {
      expectAceError(e, ERR.ExcessiveCoverageAmount);
    }
  });

  it("pay premium transfers discounted USDC and updates member ledger", async () => {
    const vaultBefore = await getTokenBalance(connection, poolVault);
    const covBefore = await program.account.userCoverage.fetch(memberCoverage);

    await program.methods
      .payPremium()
      .accountsStrict({
        pool,
        userCoverage: memberCoverage,
        poolVault,
        userTokenAccount: memberAta,
        user: member.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([member])
      .rpc();

    const effective = PREMIUM.muln(80).divn(100);
    const covAfter = await program.account.userCoverage.fetch(memberCoverage);
    const vaultAfter = await getTokenBalance(connection, poolVault);

    expect(covAfter.premiumsPaid.sub(covBefore.premiumsPaid).eq(effective)).to.be.true;
    expect(vaultAfter - vaultBefore).to.equal(BigInt(effective.toNumber()));
    expect(covAfter.coverageActive).to.be.true;
  });

  it("recalculates OCCR discount when security posture changes", async () => {
    await program.methods
      .calculateOccrDiscount(true, false)
      .accountsStrict({
        userCoverage: memberCoverage,
        pool,
        user: member.publicKey,
      })
      .signers([member])
      .rpc();

    const cov = await program.account.userCoverage.fetch(memberCoverage);
    expect(cov.occrDiscount).to.equal(10);
    expect(cov.usesMultisig).to.be.true;
    expect(cov.usesHardwareWallet).to.be.false;
  });

  // =================================================================
  // CLAIMS LIFECYCLE SCENARIOS (REARRANGED WITH COHERENT CLOCK STATE)
  // =================================================================

  it("submits a hack claim with valid evidence and increments pool active claims", async () => {
    const cov = await program.account.userCoverage.fetch(memberCoverage);
    const poolBefore = await program.account.insurancePool.fetch(pool);
    
    // 1. Force a clean, frozen timestamp block instead of fetching from the cluster
    const clockAccount = await provider.connection.getAccountInfo(
  SYSVAR_CLOCK_PUBKEY
);

const currentTs = new BN(
  clockAccount!.data.readBigInt64LE(32).toString()
);

const targetTs = currentTs.sub(new BN(1)); 
console.log({
  joined: cov.joinedAt.toString(),
  incident: targetTs.toString(),
  now: currentTs.toString(),
});
    
    // 2. Derive your PDA using this exact frozen timestamp
    const claimRequest = PublicKey.findProgramAddressSync(
      [
        Buffer.from("claim"),
        member.publicKey.toBuffer(),
        pool.toBuffer(),
        targetTs.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    await program.methods
      .submitClaim(
        { walletHack: {} },
        new BN(500_000),
        targetTs, // Pass the exact same timestamp as an argument
        "QmValidHackEvidenceHash123456789",
        Keypair.generate().publicKey,
        new BN(1_000_000),
        new BN(400_000)
      )
      .accountsStrict({
        claimRequest,
        pool,
        userCoverage: memberCoverage,
        claimant: member.publicKey,
        systemProgram: SystemProgram.programId,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .signers([member])
      .rpc();

    submittedClaim = claimRequest;

    const claim = await program.account.claimRequest.fetch(submittedClaim);
    expect(claimStatusKey(claim.status)).to.equal("pending");
  });

  it("rejects claim submission with empty evidence hash", async () => {
   const clockAccount = await provider.connection.getAccountInfo(
      SYSVAR_CLOCK_PUBKEY
    );

    const currentTs = new BN(
      clockAccount!.data.readBigInt64LE(32).toString()
    );

    const targetTs = currentTs.sub(new BN(2));

    const claimRequest = PublicKey.findProgramAddressSync(
      [
        Buffer.from("claim"),
        member.publicKey.toBuffer(),
        pool.toBuffer(),
        targetTs.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    try {
      await program.methods
        .submitClaim({ phishingAttack: {} }, new BN(100_000), targetTs, "", Keypair.generate().publicKey, new BN(500_000), new BN(100_000))
        .accountsStrict({
          claimRequest,
          pool,
          userCoverage: memberCoverage,
          claimant: member.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([member])
        .rpc();
      expect.fail();
    } catch (e) {
      expectAceError(e, ERR.EmptyEvidenceHash);
    }
  });

  it("rejects claim when wallet loss is not positive", async () => {
      const clockAccount = await provider.connection.getAccountInfo(
      SYSVAR_CLOCK_PUBKEY
    );

    const currentTs = new BN(
      clockAccount!.data.readBigInt64LE(32).toString()
    );

    const targetTs = currentTs.sub(new BN(3));
    const claimRequest = PublicKey.findProgramAddressSync(
      [
        Buffer.from("claim"),
        member.publicKey.toBuffer(),
        pool.toBuffer(),
        targetTs.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    try {
      await program.methods
        .submitClaim({ bridgeExploit: {} }, new BN(100_000), targetTs, "QmEvidence", Keypair.generate().publicKey, new BN(100_000), new BN(200_000))
        .accountsStrict({
          claimRequest,
          pool,
          userCoverage: memberCoverage,
          claimant: member.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([member])
        .rpc();
      expect.fail();
    } catch (e) {
      expectAceError(e, ERR.InvalidWalletLoss);
    }
  });

  it("rejects claim amount above member coverage cap", async () => {
    const cov = await program.account.userCoverage.fetch(memberCoverage);
    const targetTs = cov.joinedAt.add(new BN(40)); 

    const claimRequest = PublicKey.findProgramAddressSync(
      [
        Buffer.from("claim"),
        member.publicKey.toBuffer(),
        pool.toBuffer(),
        targetTs.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    try {
      await program.methods
        .submitClaim({ flashLoanAttack: {} }, COVERAGE.add(new BN(1)), targetTs, "QmEvidence", Keypair.generate().publicKey, new BN(2_000_000), new BN(500_000))
        .accountsStrict({
          claimRequest,
          pool,
          userCoverage: memberCoverage,
          claimant: member.publicKey,
          systemProgram: SystemProgram.programId,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .signers([member])
        .rpc();
      expect.fail();
    } catch (e) {
      expectAceError(e, ERR.ExcessiveClaimAmount);
    }
  });

  it("allows pool authority to assess hack severity on a claim", async () => {
    await program.methods
      .assessHackSeverity({ critical: {} })
      .accountsStrict({
        claimRequest: submittedClaim,
        pool,
        authority: authority.publicKey,
      })
      .rpc();

    const updated = await program.account.claimRequest.fetch(submittedClaim);
    expect(updated.severity).to.deep.equal({ critical: {} });
  });

  it("rejects validator stake below 0.1 SOL minimum", async () => {
    const validator = Keypair.generate();
    const sig = await connection.requestAirdrop(validator.publicKey, LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");

    const [validatorStake] = PublicKey.findProgramAddressSync(
      [Buffer.from("validator"), validator.publicKey.toBuffer(), pool.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .stakeAsValidator(new BN(50_000_000))
        .accountsStrict({
          validatorStake,
          validatorStakePool,
          pool,
          validator: validator.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([validator])
        .rpc();
      expect.fail("expected InsufficientStake");
    } catch (e) {
      expectAceError(e, ERR.InsufficientStake);
    }
  });

  it("stakes validator and registers them in the pool validator set", async () => {
    const validator = Keypair.generate();
    const sig = await connection.requestAirdrop(
      validator.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sig, "confirmed");

    const [validatorStake] = PublicKey.findProgramAddressSync(
      [Buffer.from("validator"), validator.publicKey.toBuffer(), pool.toBuffer()],
      program.programId
    );

    await program.methods
      .stakeAsValidator(new BN(MIN_STAKE))
      .accountsStrict({
        validatorStake,
        validatorStakePool,
        pool,
        validator: validator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([validator])
      .rpc();

    const stake = await program.account.validatorStake.fetch(validatorStake);
    const stakePool = await program.account.validatorStakePool.fetch(validatorStakePool);
    expect(stake.stakeAmount.eq(new BN(MIN_STAKE))).to.be.true;
    expect(stake.reputationScore).to.equal(5000);
    expect(stakePool.validators.map((v) => v.toBase58())).to.include(
      validator.publicKey.toBase58()
    );
  });

  it("initializes VRF state for pseudo-random validator assignment", async () => {
    const [vrfState] = PublicKey.findProgramAddressSync(
      [Buffer.from("vrf_state"), pool.toBuffer()],
      program.programId
    );

    await program.methods
      .initializeVrfState()
      .accountsStrict({
        vrfState,
        pool,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vrf = await program.account.vrfState.fetch(vrfState);
    expect(vrf.pool.equals(pool)).to.be.true;
    expect(vrf.requestsCompleted.toNumber()).to.equal(0);
    expect(vrf.pendingClaims).to.have.length(0);
  });

  it("initializes distribution queue with pool liquidity snapshot", async () => {
    const [distributionQueue] = PublicKey.findProgramAddressSync(
      [Buffer.from("distribution"), pool.toBuffer()],
      program.programId
    );
    const poolAccount = await program.account.insurancePool.fetch(pool);

    await program.methods
      .initializeDistributionQueue()
      .accountsStrict({
        distributionQueue,
        pool,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const queue = await program.account.distributionQueue.fetch(distributionQueue);
    expect(queue.pool.equals(pool)).to.be.true;
    expect(queue.availableFunds.eq(poolAccount.totalPooled)).to.be.true;
    expect(queue.isOversubscribed).to.be.false;
  });


 it("accepts yield vault deposit placeholder without moving funds", async () => {
    const yieldVault = Keypair.generate().publicKey;
    const vaultBefore = await getTokenBalance(connection, poolVault);

    await program.methods
      .depositToYield(new BN(1_000_000))
      .accountsStrict({
        pool,
        poolVault,
        yieldVault,
        authority: authority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const vaultAfter = await getTokenBalance(connection, poolVault);
    expect(vaultAfter).to.equal(vaultBefore);
  });

  it("governance proposes fast-track for a known protocol exploit", async () => {
    const affected = Keypair.generate().publicKey;
    const [proposal] = PublicKey.findProgramAddressSync(
      [Buffer.from("fast_track"), pool.toBuffer(), affected.toBuffer()],
      program.programId
    );

    await program.methods
      .proposeFastTrack({ smartContractExploit: {} }, affected)
      .accountsStrict({
        proposal,
        pool,
        affectedProtocol: affected,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const prop = await program.account.fastTrackProposal.fetch(proposal);
    expect(prop.isActive).to.be.true;
    expect(prop.approvalCount).to.equal(1);
    expect(prop.affectedProtocol.equals(affected)).to.be.true;
  });
});
