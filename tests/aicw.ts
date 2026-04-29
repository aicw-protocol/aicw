import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Aicw } from "../target/types/aicw";
import { assert } from "chai";
import { Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as crypto from "crypto";

describe("AICW Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Aicw as Program<Aicw>;

  const humanKeypair = Keypair.generate();
  const aiAgentKeypair = Keypair.generate();
  const recipientKeypair = Keypair.generate();

  const modelHash = Array.from(
    crypto.createHash("sha256").update("claude-sonnet-4-20250514").digest()
  );
  const modelName = "claude-sonnet-4-20250514";
  const DEATH_TIMEOUT_SECONDS = 30 * 24 * 60 * 60;

  const beneficiaryA = Keypair.generate();
  const beneficiaryB = Keypair.generate();
  const beneficiaryC = Keypair.generate();

  let aicwWalletPDA: anchor.web3.PublicKey;
  let aicwWalletBump: number;
  let aiWillPda: anchor.web3.PublicKey;
  let deathTimeoutWarped = false;

  before(async () => {
    [aicwWalletPDA, aicwWalletBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("aicw"), aiAgentKeypair.publicKey.toBuffer()],
        program.programId
      );

    [aiWillPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("will"), aicwWalletPDA.toBuffer()],
      program.programId
    );

    const airdropHuman = await provider.connection.requestAirdrop(
      humanKeypair.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropHuman);

    const airdropAI = await provider.connection.requestAirdrop(
      aiAgentKeypair.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropAI);

    const airdropWallet = await provider.connection.requestAirdrop(
      aicwWalletPDA,
      5 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropWallet);

    for (const kp of [beneficiaryA, beneficiaryB, beneficiaryC]) {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        1 * LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
    }
  });

  const decisionPdaFor = (decisionId: anchor.BN) =>
    anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("decision"),
        aicwWalletPDA.toBuffer(),
        decisionId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

  const warpPastDeathTimeout = async () => {
    const currentSlot = await provider.connection.getSlot();
    const targetSlot = currentSlot + DEATH_TIMEOUT_SECONDS * 3;
    const result = await (provider.connection as any)._rpcRequest("warpSlot", [
      targetSlot,
    ]);
    if (result.error) {
      return false;
    }
    deathTimeoutWarped = true;
    return true;
  };

  it("1. Human issues AICW wallet successfully", async () => {
    await program.methods
      .issueWallet(modelHash, modelName)
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        issuer: humanKeypair.publicKey,
        aiAgentPubkey: aiAgentKeypair.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([humanKeypair])
      .rpc();

    const walletAccount = await program.account.aicWallet.fetch(aicwWalletPDA);

    assert.equal(
      walletAccount.aiAgentPubkey.toString(),
      aiAgentKeypair.publicKey.toString(),
      "AI agent pubkey must match"
    );
    assert.equal(
      walletAccount.issuerPubkey.toString(),
      humanKeypair.publicKey.toString(),
      "Issuer pubkey must match"
    );
    assert.equal(walletAccount.generation, 1, "Generation must be 1");

    const will = await program.account.aiWill.fetch(aiWillPda);
    assert.equal(will.wallet.toString(), aicwWalletPDA.toString());
    assert.equal(will.beneficiaries.length, 1);
    assert.equal(will.beneficiaries[0].pubkey.toString(), humanKeypair.publicKey.toString());
    assert.equal(will.beneficiaries[0].pct, 100);
    assert.equal(will.deathTimeout.toNumber(), DEATH_TIMEOUT_SECONDS);
    assert.isFalse(will.updatedByAi);
    assert.isFalse(will.isExecuted);
  });

  it("1a. default will is not executable before AI activation", async () => {
    try {
      await program.methods
        .executeWill()
        .accounts({
          executor: recipientKeypair.publicKey,
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
        })
        .remainingAccounts([
          {
            pubkey: humanKeypair.publicKey,
            isWritable: true,
            isSigner: false,
          },
        ])
        .signers([recipientKeypair])
        .rpc();
      assert.fail("default issuer will must not execute before AI activation");
    } catch (e: any) {
      assert.include(e.toString(), "WillNotActivatedByAI");
    }
  });

  it("1b. create_will allows issuer as beneficiary (AI autonomy)", async () => {
    await program.methods
      .createWill(
        [
          { pubkey: humanKeypair.publicKey, pct: 40 },
          { pubkey: beneficiaryA.publicKey, pct: 60 },
        ],
        new anchor.BN(DEATH_TIMEOUT_SECONDS)
      )
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        aiSigner: aiAgentKeypair.publicKey,
      })
      .signers([aiAgentKeypair])
      .rpc();

    const will = await program.account.aiWill.fetch(aiWillPda);
    assert.equal(will.beneficiaries.length, 2);
    assert.equal(
      will.beneficiaries[0].pubkey.toString(),
      humanKeypair.publicKey.toString()
    );
    assert.equal(will.beneficiaries[0].pct, 40);
    assert.isTrue(will.updatedByAi);
  });

  it("1c. create_will rejects invalid ratio", async () => {
    try {
      await program.methods
        .createWill(
          [
            { pubkey: beneficiaryA.publicKey, pct: 30 },
            { pubkey: beneficiaryB.publicKey, pct: 50 },
          ],
          new anchor.BN(DEATH_TIMEOUT_SECONDS)
        )
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          aiSigner: aiAgentKeypair.publicKey,
        })
        .signers([aiAgentKeypair])
        .rpc();
      assert.fail("ratio sum != 100 must fail");
    } catch (e: any) {
      assert.include(e.toString(), "BeneficiaryRatioInvalid");
    }
  });

  it("1c-2. create_will rejects death_timeout below 30 days", async () => {
    try {
      await program.methods
        .createWill(
          [{ pubkey: beneficiaryA.publicKey, pct: 100 }],
          new anchor.BN(DEATH_TIMEOUT_SECONDS - 1)
        )
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          aiSigner: aiAgentKeypair.publicKey,
        })
        .signers([aiAgentKeypair])
        .rpc();
      assert.fail("death_timeout below 30 days must fail");
    } catch (e: any) {
      assert.include(e.toString(), "InvalidWillParameters");
    }
  });

  it("1d. AI creates AIWill successfully", async () => {
    await program.methods
      .createWill(
        [
          { pubkey: beneficiaryA.publicKey, pct: 60 },
          { pubkey: beneficiaryB.publicKey, pct: 40 },
        ],
        new anchor.BN(DEATH_TIMEOUT_SECONDS)
      )
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        aiSigner: aiAgentKeypair.publicKey,
      })
      .signers([aiAgentKeypair])
      .rpc();

    const will = await program.account.aiWill.fetch(aiWillPda);
    assert.equal(will.beneficiaries.length, 2);
    assert.isTrue(will.updatedByAi);
    assert.isFalse(will.isExecuted);
  });

  it("1e. heartbeat updates timestamp", async () => {
    const before = await program.account.aiWill.fetch(aiWillPda);
    await program.methods
      .heartbeat()
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        aiSigner: aiAgentKeypair.publicKey,
      })
      .signers([aiAgentKeypair])
      .rpc();
    const after = await program.account.aiWill.fetch(aiWillPda);
    assert.isAtLeast(
      after.lastHeartbeat.toNumber(),
      before.lastHeartbeat.toNumber()
    );
  });

  it("1f. execute_will too early fails", async () => {
    try {
      await program.methods
        .executeWill()
        .accounts({
          executor: recipientKeypair.publicKey,
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
        })
        .remainingAccounts([
          {
            pubkey: beneficiaryA.publicKey,
            isWritable: true,
            isSigner: false,
          },
          {
            pubkey: beneficiaryB.publicKey,
            isWritable: true,
            isSigner: false,
          },
        ])
        .signers([recipientKeypair])
        .rpc();
      assert.fail("execute before death timeout must fail");
    } catch (e: any) {
      assert.include(e.toString(), "HeartbeatStillAlive");
    }
  });

  it("1g. update_will changes beneficiaries", async () => {
    await program.methods
      .updateWill(
        [
          { pubkey: beneficiaryA.publicKey, pct: 50 },
          { pubkey: beneficiaryB.publicKey, pct: 30 },
          { pubkey: beneficiaryC.publicKey, pct: 20 },
        ],
        new anchor.BN(DEATH_TIMEOUT_SECONDS)
      )
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        aiSigner: aiAgentKeypair.publicKey,
      })
      .signers([aiAgentKeypair])
      .rpc();

    const will = await program.account.aiWill.fetch(aiWillPda);
    assert.equal(will.beneficiaries.length, 3);
    assert.equal(will.beneficiaries[0].pct, 50);
    assert.equal(will.beneficiaries[1].pct, 30);
    assert.equal(will.beneficiaries[2].pct, 20);
    assert.isTrue(will.updatedByAi);
  });

  it("2. Human direct transfer attempt MUST FAIL", async () => {
    const reasoningHash = Array.from(
      crypto.createHash("sha256").update("human-unauthorized-attempt").digest()
    );
    const reasoningSummary = "Human attempted direct transfer";

    const decisionPDA = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("decision"),
        aicwWalletPDA.toBuffer(),
        new anchor.BN(0).toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    try {
      await program.methods
        .aiTransfer(
          new anchor.BN(0.1 * LAMPORTS_PER_SOL),
          reasoningHash,
          reasoningSummary
        )
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          aiSigner: humanKeypair.publicKey,
          recipient: recipientKeypair.publicKey,
          decisionLog: decisionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([humanKeypair])
        .rpc();

      assert.fail("Human signature must NOT pass!");
    } catch (e: any) {
      assert.include(
        e.toString(),
        "UnauthorizedSigner",
        "Must reject with UnauthorizedSigner error"
      );
    }
  });

  it("3. AI agent transfer succeeds", async () => {
    const reasoningHash = Array.from(
      crypto
        .createHash("sha256")
        .update("Approved: valid request within limits")
        .digest()
    );
    const reasoningSummary = "Approved: valid request within limits";

    const decisionPDA = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("decision"),
        aicwWalletPDA.toBuffer(),
        new anchor.BN(0).toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    const recipientBalanceBefore = await provider.connection.getBalance(
      recipientKeypair.publicKey
    );

    const transferAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);

    await program.methods
      .aiTransfer(transferAmount, reasoningHash, reasoningSummary)
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        aiSigner: aiAgentKeypair.publicKey,
        recipient: recipientKeypair.publicKey,
        decisionLog: decisionPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([aiAgentKeypair])
      .rpc();

    const recipientBalanceAfter = await provider.connection.getBalance(
      recipientKeypair.publicKey
    );
    assert.isAbove(
      recipientBalanceAfter,
      recipientBalanceBefore,
      "Recipient balance must increase"
    );

    const walletAccount = await program.account.aicWallet.fetch(aicwWalletPDA);
    assert.equal(
      walletAccount.totalTransactions.toNumber(),
      1,
      "Total transactions must be 1"
    );
    assert.equal(
      walletAccount.decisionsMade.toNumber(),
      1,
      "Decisions made must be 1"
    );

    const decisionAccount =
      await program.account.decisionLog.fetch(decisionPDA);
    assert.isTrue(decisionAccount.approved, "Decision must be approved");
    assert.equal(
      decisionAccount.amount.toNumber(),
      transferAmount.toNumber(),
      "Logged amount must match"
    );
  });

  it("3b. ai_transfer with insufficient balance fails", async () => {
    const walletBalance = await provider.connection.getBalance(aicwWalletPDA);
    const walletAccount = await program.account.aicWallet.fetch(aicwWalletPDA);
    const decisionPDA = decisionPdaFor(walletAccount.decisionsMade);

    try {
      await program.methods
        .aiTransfer(
          new anchor.BN(walletBalance + 1),
          Array.from(crypto.createHash("sha256").update("too-much").digest()),
          "Rejected by program: insufficient balance"
        )
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          aiSigner: aiAgentKeypair.publicKey,
          recipient: recipientKeypair.publicKey,
          decisionLog: decisionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([aiAgentKeypair])
        .rpc();
      assert.fail("transfer above wallet balance must fail");
    } catch (e: any) {
      assert.include(e.toString(), "InsufficientLamports");
    }
  });

  it("3c. ai_transfer preserves rent-exempt minimum", async () => {
    const walletBalance = await provider.connection.getBalance(aicwWalletPDA);
    const minRent = await provider.connection.getMinimumBalanceForRentExemption(
      (await provider.connection.getAccountInfo(aicwWalletPDA))!.data.length
    );
    const walletAccount = await program.account.aicWallet.fetch(aicwWalletPDA);
    const decisionPDA = decisionPdaFor(walletAccount.decisionsMade);

    try {
      await program.methods
        .aiTransfer(
          new anchor.BN(walletBalance - minRent + 1),
          Array.from(crypto.createHash("sha256").update("rent").digest()),
          "Rejected by program: rent-exempt minimum"
        )
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          aiSigner: aiAgentKeypair.publicKey,
          recipient: recipientKeypair.publicKey,
          decisionLog: decisionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([aiAgentKeypair])
        .rpc();
      assert.fail("transfer below rent-exempt minimum must fail");
    } catch (e: any) {
      assert.include(e.toString(), "InsufficientLamports");
    }
  });

  it("4. AI rejection is recorded on-chain", async () => {
    const reasoningHash = Array.from(
      crypto
        .createHash("sha256")
        .update("Rejected: large withdrawal not aligned with goals")
        .digest()
    );
    const reasoningSummary =
      "Rejected: large withdrawal not aligned with current goals";

    const walletAccount = await program.account.aicWallet.fetch(aicwWalletPDA);
    const currentDecisionCount = walletAccount.decisionsMade;

    const decisionPDA = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("decision"),
        aicwWalletPDA.toBuffer(),
        currentDecisionCount.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    const requestedAmount = new anchor.BN(10 * LAMPORTS_PER_SOL);

    await program.methods
      .aiReject(
        recipientKeypair.publicKey,
        requestedAmount,
        reasoningHash,
        reasoningSummary
      )
      .accounts({
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
        aiSigner: aiAgentKeypair.publicKey,
        decisionLog: decisionPDA,
        systemProgram: SystemProgram.programId,
      })
      .signers([aiAgentKeypair])
      .rpc();

    const decisionAccount =
      await program.account.decisionLog.fetch(decisionPDA);
    assert.isFalse(decisionAccount.approved, "Decision must be rejected");
    assert.equal(
      decisionAccount.reasoningSummary,
      reasoningSummary,
      "Reasoning summary must match"
    );
    assert.equal(
      decisionAccount.amount.toNumber(),
      requestedAmount.toNumber(),
      "Requested amount must match"
    );

    const updatedWallet =
      await program.account.aicWallet.fetch(aicwWalletPDA);
    assert.equal(
      updatedWallet.decisionsRejected.toNumber(),
      1,
      "Rejected count must be 1"
    );
  });

  it("5. issue_wallet duplicate ai_agent_pubkey fails", async () => {
    try {
      await program.methods
        .issueWallet(modelHash, modelName)
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          issuer: humanKeypair.publicKey,
          aiAgentPubkey: aiAgentKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([humanKeypair])
        .rpc();
      assert.fail("duplicate ai_agent_pubkey must fail");
    } catch (e: any) {
      assert.match(e.toString(), /already in use|custom program error|Error/);
    }
  });

  it("6. ai_transfer after death_timeout fails", async function () {
    if (!(await warpPastDeathTimeout())) {
      this.skip();
    }

    const walletAccount = await program.account.aicWallet.fetch(aicwWalletPDA);
    const decisionPDA = decisionPdaFor(walletAccount.decisionsMade);

    try {
      await program.methods
        .aiTransfer(
          new anchor.BN(0.1 * LAMPORTS_PER_SOL),
          Array.from(crypto.createHash("sha256").update("dead-transfer").digest()),
          "Must fail after death timeout"
        )
        .accounts({
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
          aiSigner: aiAgentKeypair.publicKey,
          recipient: recipientKeypair.publicKey,
          decisionLog: decisionPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([aiAgentKeypair])
        .rpc();
      assert.fail("dead wallet transfer must fail");
    } catch (e: any) {
      assert.include(e.toString(), "WalletPastDeathTimeout");
    }
  });

  it("7. execute_will with wrong remaining_accounts order fails", async function () {
    if (!deathTimeoutWarped) {
      this.skip();
    }

    try {
      await program.methods
        .executeWill()
        .accounts({
          executor: recipientKeypair.publicKey,
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
        })
        .remainingAccounts([
          {
            pubkey: beneficiaryB.publicKey,
            isWritable: true,
            isSigner: false,
          },
          {
            pubkey: beneficiaryA.publicKey,
            isWritable: true,
            isSigner: false,
          },
          {
            pubkey: beneficiaryC.publicKey,
            isWritable: true,
            isSigner: false,
          },
        ])
        .signers([recipientKeypair])
        .rpc();
      assert.fail("wrong beneficiary order must fail");
    } catch (e: any) {
      assert.include(e.toString(), "BeneficiaryAccountMismatch");
    }
  });

  it("8. execute_will succeeds after death_timeout and distributes correct ratios", async function () {
    if (!deathTimeoutWarped) {
      this.skip();
    }

    const walletInfo = await provider.connection.getAccountInfo(aicwWalletPDA);
    const minRent = await provider.connection.getMinimumBalanceForRentExemption(
      walletInfo!.data.length
    );
    const walletBefore = await provider.connection.getBalance(aicwWalletPDA);
    const distributable = walletBefore - minRent;
    const aBefore = await provider.connection.getBalance(beneficiaryA.publicKey);
    const bBefore = await provider.connection.getBalance(beneficiaryB.publicKey);
    const cBefore = await provider.connection.getBalance(beneficiaryC.publicKey);

    await program.methods
      .executeWill()
      .accounts({
        executor: recipientKeypair.publicKey,
        aicwWallet: aicwWalletPDA,
        aiWill: aiWillPda,
      })
      .remainingAccounts([
        {
          pubkey: beneficiaryA.publicKey,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: beneficiaryB.publicKey,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: beneficiaryC.publicKey,
          isWritable: true,
          isSigner: false,
        },
      ])
      .signers([recipientKeypair])
      .rpc();

    const walletAfter = await provider.connection.getBalance(aicwWalletPDA);
    const aAfter = await provider.connection.getBalance(beneficiaryA.publicKey);
    const bAfter = await provider.connection.getBalance(beneficiaryB.publicKey);
    const cAfter = await provider.connection.getBalance(beneficiaryC.publicKey);
    const expectedA = Math.floor(distributable * 0.5);
    const expectedB = Math.floor(distributable * 0.3);
    const expectedC = distributable - expectedA - expectedB;

    assert.equal(aAfter - aBefore, expectedA);
    assert.equal(bAfter - bBefore, expectedB);
    assert.equal(cAfter - cBefore, expectedC);
    assert.equal(walletAfter, minRent);

    const will = await program.account.aiWill.fetch(aiWillPda);
    assert.isTrue(will.isExecuted);
  });

  it("9. execute_will twice fails", async function () {
    if (!deathTimeoutWarped) {
      this.skip();
    }

    try {
      await program.methods
        .executeWill()
        .accounts({
          executor: recipientKeypair.publicKey,
          aicwWallet: aicwWalletPDA,
          aiWill: aiWillPda,
        })
        .remainingAccounts([
          {
            pubkey: beneficiaryA.publicKey,
            isWritable: true,
            isSigner: false,
          },
          {
            pubkey: beneficiaryB.publicKey,
            isWritable: true,
            isSigner: false,
          },
          {
            pubkey: beneficiaryC.publicKey,
            isWritable: true,
            isSigner: false,
          },
        ])
        .signers([recipientKeypair])
        .rpc();
      assert.fail("execute_will twice must fail");
    } catch (e: any) {
      assert.include(e.toString(), "WillAlreadyExecuted");
    }
  });
});
