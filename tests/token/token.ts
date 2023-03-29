import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  createTransferCheckedInstruction,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
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

  it("deposit token!", async () => {
    const associatedTokenProgram = anchor.utils.token.ASSOCIATED_PROGRAM_ID;
    const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
    const systemProgram = anchor.web3.SystemProgram.programId;

    const authority = owner.publicKey;
    const from = anchor.utils.token.associatedAddress({
      mint,
      owner: authority,
    });
    const [batchVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(BATCH_SEED), authority.toBuffer()],
      program.programId
    );
    const batchVaultTokenAccount = anchor.utils.token.associatedAddress({
      mint,
      owner: batchVault,
    });
    const amount = new anchor.BN("1").mul(
      new anchor.BN(anchor.web3.LAMPORTS_PER_SOL)
    );

    const sig = await program.methods
      .depositToken(amount)
      .accounts({
        associatedTokenProgram,
        authority,
        from,
        batchVault,
        mint,
        systemProgram,
        tokenProgram,
        batchVaultTokenAccount,
      })
      .signers([owner])
      .rpc();

    console.log("Your transaction signature", sig);
  });

  it("Bulk Transfer", async () => {
    const batchVault = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(BATCH_SEED), owner.publicKey.toBuffer()],
      program.programId
    );
    console.log("owner-pda", batchVault[0].toBase58());

    const batchVaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      batchVault[0],
      true
    );

    // generate random accounts
    const random_accounts_ata: anchor.web3.PublicKey[] = [];
    const amounts: anchor.BN[] = [];

    // max 19 accounts - without transfer_checked
    // max 15 accounts - with transfer_checked
    for (let i = 0; i < 15; i++) {
      let account = anchor.web3.Keypair.generate();
      let ata = await createAssociatedTokenAccount(
        connection,
        owner,
        mint,
        account.publicKey
      );
      random_accounts_ata.push(ata);
      amounts.push(new anchor.BN(100000000));
    }

    //Array of accout metas
    const accounts: anchor.web3.AccountMeta[] = [];
    for (const account of random_accounts_ata) {
      accounts.push({
        pubkey: account,
        isSigner: false,
        isWritable: true,
      });
    }

    accounts.push(
      {
        pubkey: batchVault[0],
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: batchVaultTokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: mint,
        isSigner: false,
        isWritable: false,
      }
    );

    try {
      // call the program
      const sig = await program.methods
        .batchTokenTransfer(amounts)
        .accounts({
          batchVaultTokenAccount,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          fromAuthority: owner.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          batchVault: batchVault[0],
          mint: mint,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .remainingAccounts(accounts)
        .rpc();
      console.log(sig);
    } catch (e) {
      console.log(e);
    }
  });
});
