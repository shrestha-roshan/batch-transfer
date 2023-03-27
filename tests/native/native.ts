import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BatchTransfer } from "../../target/types/batch_transfer";

describe("batch-transfer", async () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const BATCH_SEED = "transfer-batch";

  const program = anchor.workspace.BatchTransfer as Program<BatchTransfer>;
  const connection = anchor.getProvider().connection;

  const owner_buffer = [
    199, 70, 8, 99, 248, 66, 30, 8, 75, 69, 185, 133, 180, 74, 109, 145, 120,
    169, 57, 112, 248, 44, 107, 35, 88, 237, 151, 106, 224, 78, 206, 131, 119,
    249, 244, 237, 207, 161, 59, 99, 84, 105, 11, 179, 7, 76, 68, 230, 58, 65,
    232, 121, 248, 108, 113, 162, 88, 226, 92, 40, 215, 99, 140, 220,
  ];
  const mint = new anchor.web3.PublicKey(
    "9xLaH2mj5mX1apnPdWfaU24x5sR9QF1wPU6gbZBAgorf"
  );
  const owner = anchor.web3.Keypair.fromSecretKey(Buffer.from(owner_buffer));

  it("Deposit SOL", async () => {
    const systemProgram = anchor.web3.SystemProgram.programId;

    const [batchVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(BATCH_SEED), owner.publicKey.toBuffer()],
      program.programId
    );
    const amount = new anchor.BN("10").mul(
      new anchor.BN(anchor.web3.LAMPORTS_PER_SOL)
    );

    const sig = await program.methods
      .depositSol(amount)
      .accounts({
        authority: owner.publicKey,
        batchVault,
        systemProgram,
      })
      .signers([owner])
      .rpc();
    console.log("Your transaction signature", sig);
  });

  it("Bulk Transfer Native", async () => {
    const batchVault = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(BATCH_SEED), owner.publicKey.toBuffer()],
      program.programId
    );
    console.log("owner-pda", batchVault[0].toBase58());

    // generate random accounts
    const random_accounts: anchor.web3.PublicKey[] = [];
    const amounts: anchor.BN[] = [];

    //max 22 accounts
    for (let i = 0; i < 22; i++) {
      random_accounts.push(anchor.web3.Keypair.generate().publicKey);
      amounts.push(new anchor.BN(100000000));
    }
    //Array of accout metas
    const accounts: anchor.web3.AccountMeta[] = [];
    for (const account of random_accounts) {
      accounts.push({
        pubkey: account,
        isSigner: false,
        isWritable: true,
      });
    }
    // request air drop for all accounts
    for (const account of random_accounts) {
      anchor.getProvider().connection.requestAirdrop(account, 1000);
    }

    accounts.push({
      pubkey: batchVault[0],
      isSigner: false,
      isWritable: true,
    });

    try {
      // call the program
      const sig = await program.methods
        .batchSolTransfer(amounts)
        .accounts({
          batchVault: batchVault[0],
          fromAuthority: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([])
        .remainingAccounts(accounts)
        .rpc();
      console.log(sig);
    } catch (e) {
      console.log(e);
    }
  });
});
