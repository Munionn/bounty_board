import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BountyBoard } from "../target/types/bounty_board";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("bounty_board", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.BountyBoard as Program<BountyBoard>;
  const provider = anchor.getProvider();

  it("Initializes a user profile!", async () => {
    const payer = (provider.wallet as any).publicKey;

    const [userProfilePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), payer.toBuffer()],
      program.programId
    );

    // Check if account already exists
    const account = await program.account.userProfile.fetchNullable(userProfilePda);
    if (!account) {
      await program.methods
        .initializeUser()
        .accounts({
          userProfile: userProfilePda,
          payer: payer,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    }

    const userProfileAccount = await program.account.userProfile.fetch(userProfilePda);

    expect(userProfileAccount.authority.toBase58()).to.equal(payer.toBase58());
  });

  it("Updates the user reputation!", async () => {
    const payer = (provider.wallet as any).publicKey;

    const [userProfilePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), payer.toBuffer()],
      program.programId
    );

    const newReputation = new anchor.BN(50);
    await program.methods
      .updateUser(newReputation)
      .accounts({
        userProfile: userProfilePda,
        payer: payer,
      })
      .rpc();

    const userProfileAccount = await program.account.userProfile.fetch(userProfilePda);

    expect(userProfileAccount.reputation.toNumber()).to.equal(50);
  });
});
