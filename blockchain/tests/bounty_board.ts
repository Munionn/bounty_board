import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BountyBoard } from "../target/types/bounty_board";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";
import BN from "bn.js";

describe("bounty_board", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.BountyBoard as Program<BountyBoard>;
  const connection = provider.connection;

  const poster = (provider.wallet as anchor.Wallet).payer;
  const developer = Keypair.generate();

  const bountyId = new BN(1);
  const bountyAmount = new BN(0.1 * LAMPORTS_PER_SOL);

  const [globalStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("global")],
    program.programId
  );

  const [userProfilePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("user"), poster.publicKey.toBuffer()],
    program.programId
  );

  const [bountyPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("bounty"), poster.publicKey.toBuffer(), bountyId.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  before(async () => {
    const sig = await connection.requestAirdrop(
      developer.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(sig);
  });

  it("initializes global state", async () => {
    const existing = await program.account.globalState.fetchNullable(globalStatePda);
    if (!existing) {
      await program.methods
        .initialize()
        .accounts({
          payer: poster.publicKey,
          globalState: globalStatePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    const globalState = await program.account.globalState.fetch(globalStatePda);
    expect(globalState.authority.toBase58()).to.equal(poster.publicKey.toBase58());
  });

  it("initializes a user profile", async () => {
    const existing = await program.account.userProfile.fetchNullable(userProfilePda);
    if (!existing) {
      await program.methods
        .initializeUser()
        .accounts({
          payer: poster.publicKey,
          userProfile: userProfilePda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    const userProfile = await program.account.userProfile.fetch(userProfilePda);
    expect(userProfile.authority.toBase58()).to.equal(poster.publicKey.toBase58());
    expect(userProfile.reputation.toNumber()).to.equal(0);
  });

  it("updates user reputation", async () => {
    await program.methods
      .updateUser(new BN(50))
      .accounts({
        payer: poster.publicKey,
        userProfile: userProfilePda,
      })
      .rpc();

    const userProfile = await program.account.userProfile.fetch(userProfilePda);
    expect(userProfile.reputation.toNumber()).to.equal(50);
  });

  it("posts a bounty with escrowed SOL", async () => {
    const existing = await program.account.bountyState.fetchNullable(bountyPda);
    if (!existing) {
      await program.methods
        .postBounty(bountyId, bountyAmount)
        .accounts({
          payer: poster.publicKey,
          bountyState: bountyPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    const bounty = await program.account.bountyState.fetch(bountyPda);
    expect(bounty.id.toNumber()).to.equal(bountyId.toNumber());
    expect(bounty.poster.toBase58()).to.equal(poster.publicKey.toBase58());
    expect(bounty.amount.toNumber()).to.equal(bountyAmount.toNumber());
    expect(bounty.status).to.deep.equal({ open: {} });
  });

  it("submits a task as developer", async () => {
    const bounty = await program.account.bountyState.fetch(bountyPda);
    if ("open" in bounty.status) {
      await program.methods
        .submitTask(bountyId, "ipfs://bafy-integration-test")
        .accounts({
          submitter: developer.publicKey,
          poster: poster.publicKey,
          bountyState: bountyPda,
        })
        .signers([developer])
        .rpc();
    }

    const claimed = await program.account.bountyState.fetch(bountyPda);
    expect(claimed.status).to.deep.equal({ claimed: {} });
    expect(claimed.claimer.toBase58()).to.equal(developer.publicKey.toBase58());
    expect(claimed.submisstionUri).to.equal("ipfs://bafy-integration-test");
  });

  it("approves task and pays the developer", async () => {
    const bounty = await program.account.bountyState.fetch(bountyPda);
    if (!("claimed" in bounty.status)) {
      return;
    }

    const before = await connection.getBalance(developer.publicKey);

    await program.methods
      .approveTask(bountyId)
      .accounts({
        poster: poster.publicKey,
        claimer: developer.publicKey,
        bountyState: bountyPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const closed = await program.account.bountyState.fetch(bountyPda);
    const after = await connection.getBalance(developer.publicKey);

    expect(closed.status).to.deep.equal({ closed: {} });
    expect(closed.amount.toNumber()).to.equal(0);
    expect(after).to.be.greaterThan(before);
  });

  it("cancels an open bounty and refunds the poster", async () => {
    const cancelId = new BN(99);
    const cancelAmount = new BN(0.05 * LAMPORTS_PER_SOL);
    const [cancelBountyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bounty"), poster.publicKey.toBuffer(), cancelId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .postBounty(cancelId, cancelAmount)
      .accounts({
        payer: poster.publicKey,
        bountyState: cancelBountyPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const before = await connection.getBalance(poster.publicKey);

    await program.methods
      .cancelTask(cancelId)
      .accounts({
        poster: poster.publicKey,
        bountyState: cancelBountyPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const cancelled = await program.account.bountyState.fetch(cancelBountyPda);
    const after = await connection.getBalance(poster.publicKey);

    expect(cancelled.status).to.deep.equal({ closed: {} });
    expect(cancelled.amount.toNumber()).to.equal(0);
    expect(after).to.be.greaterThan(before);
  });
});
