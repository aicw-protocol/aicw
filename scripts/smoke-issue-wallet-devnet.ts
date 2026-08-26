/**
 * Devnet smoke test: issue_wallet on deployed program.
 * Usage (WSL, repo root):
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   ANCHOR_WALLET=/mnt/c/Users/home/.config/solana/id.json \
 *   npx ts-mocha -p ./tsconfig.json -t 120000 scripts/smoke-issue-wallet-devnet.ts
 */
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { assert } from "chai";
import * as crypto from "crypto";
import { Aicw } from "../target/types/aicw";

describe("devnet smoke: issue_wallet", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Aicw as Program<Aicw>;
  const aiAgent = Keypair.generate();
  const issuer = (provider.wallet as anchor.Wallet).payer;

  const modelHash: number[] = Array.from(
    crypto.createHash("sha256").update("smoke-test-devnet").digest(),
  );
  const modelName = "smoke-test-devnet";

  let aicwWalletPda: anchor.web3.PublicKey;
  let aiWillPda: anchor.web3.PublicKey;

  before(() => {
    [aicwWalletPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("aicw"), aiAgent.publicKey.toBuffer()],
      program.programId,
    );
    [aiWillPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("will"), aicwWalletPda.toBuffer()],
      program.programId,
    );
  });

  it("issues wallet on devnet", async () => {
    const expectedProgram = "FcWqrRLcAxwqAhMSGXabD8zEKqnPHsovBvcmLaH9hsVv";
    assert.equal(program.programId.toBase58(), expectedProgram);

    const sig = await program.methods
      .issueWallet(modelHash, modelName)
      .accountsPartial({
        issuer: issuer.publicKey,
        aiAgentPubkey: aiAgent.publicKey,
      })
      .rpc();

    const wallet = await program.account.aicWallet.fetch(aicwWalletPda);
    const will = await program.account.aiWill.fetch(aiWillPda);

    assert.equal(wallet.aiAgentPubkey.toBase58(), aiAgent.publicKey.toBase58());
    assert.equal(wallet.issuerPubkey.toBase58(), issuer.publicKey.toBase58());
    assert.equal(wallet.generation, 1);
    assert.equal(will.wallet.toBase58(), aicwWalletPda.toBase58());
    assert.equal(will.beneficiaries.length, 1);
    assert.equal(will.beneficiaries[0].pct, 100);
    assert.isFalse(will.updatedByAi);

    console.log("program_id:", program.programId.toBase58());
    console.log("issuer:", issuer.publicKey.toBase58());
    console.log("ai_agent:", aiAgent.publicKey.toBase58());
    console.log("aicw_wallet_pda:", aicwWalletPda.toBase58());
    console.log("ai_will_pda:", aiWillPda.toBase58());
    console.log("signature:", sig);
  });
});
