import * as anchor from '@coral-xyz/anchor';

import {
  BatchTransfer,
  IDL,
} from '../target/types/batch_transfer';

describe("batch-transfer", async () => {

  const secret = process.env.SECRET_KEY || "";
  if (secret === "") {
    throw new Error("Missing SECRET_KEY in env");
  }
  const keypair = anchor.web3.Keypair.fromSecretKey(anchor.utils.bytes.bs58.decode(secret));
  const wallet = new anchor.Wallet(keypair);
  const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"));
  const provider = new anchor.AnchorProvider(connection, wallet, anchor.AnchorProvider.defaultOptions());
  // Configure the client to use the local cluster.

  const programId = new anchor.web3.PublicKey("DwZruo6t3BW4DUtALe2i2E6ewA8b5mH1Lk2TWeyV8ymo");

  const program = new anchor.Program<BatchTransfer>(
    IDL, 
    programId, 
    provider
  );
  
  // // Configure the client to use the local cluster.
  // anchor.setProvider(anchor.AnchorProvider.env());

  // const program = anchor.workspace.BatchTransfer as Program<BatchTransfer>;

  const owner_buffer = [
    199, 70, 8, 99, 248, 66, 30, 8, 75, 69, 185, 133, 180, 74, 109, 145, 120,
    169, 57, 112, 248, 44, 107, 35, 88, 237, 151, 106, 224, 78, 206, 131, 119,
    249, 244, 237, 207, 161, 59, 99, 84, 105, 11, 179, 7, 76, 68, 230, 58, 65,
    232, 121, 248, 108, 113, 162, 88, 226, 92, 40, 215, 99, 140, 220,
  ];

  const owner = anchor.web3.Keypair.fromSecretKey(Buffer.from(owner_buffer));

  it("Bulk Transfer", async () => {
    // generate 10 random accounts
    const random_accounts: anchor.web3.PublicKey[] = [];
    const amounts: anchor.BN[] = [];

    for (let i = 0; i < 20; i++) {
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
      console.log("airdrop");
      anchor.getProvider().connection.requestAirdrop(account, 1000);
    }

    accounts.push(
      {
        pubkey: owner.publicKey,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: anchor.web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      }
    );

    // call the program
    const sig = await program.methods
      .batchSolTransfer(amounts)
      .accounts({
        from: owner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([owner])
      .remainingAccounts(accounts)
      .rpc();
    console.log(sig);
  });

  it("deposit token!", async () => {
    
    const associatedTokenProgram = anchor.utils.token.ASSOCIATED_PROGRAM_ID;
    const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
    const systemProgram = anchor.web3.SystemProgram.programId;

    const authority = wallet.publicKey;
    const mint = new anchor.web3.PublicKey("AbLwGR8A1wvsiLWrzzA5eYPoQw51NVMcMMTPvAv5LTJ");
    const from = anchor.utils.token.associatedAddress({mint, owner: authority})
    const [ledger] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("Ledger"), authority.toBuffer()], 
      program.programId
      );
    const vault = anchor.utils.token.associatedAddress({mint, owner: ledger});
    const amount = new anchor.BN("1000").mul(new anchor.BN(anchor.web3.LAMPORTS_PER_SOL));
    
    const tx = await program.methods.depositToken(amount)
    .accounts({
      associatedTokenProgram,
      authority,
      from,
      ledger,
      mint,
      systemProgram,
      tokenProgram,
      vault
    }).transaction();


    const {blockhash, lastValidBlockHeight} = await connection.getLatestBlockhash();
    tx.feePayer = wallet.publicKey;
    tx.recentBlockhash = blockhash;
    tx.lastValidBlockHeight = lastValidBlockHeight;

    const signed = await wallet.signTransaction(tx);
    // const signature = await connection.sendRawTransaction(signed.serialize());
    // await connection.confirmTransaction({blockhash, lastValidBlockHeight, signature});
    // console.log("Your transaction signature", signature);
  });


  it("spl transfer", async ()=> {
    const associatedTokenProgram = anchor.utils.token.ASSOCIATED_PROGRAM_ID;
    const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
    const systemProgram = anchor.web3.SystemProgram.programId;

    const authority = wallet.publicKey;
    const mint = new anchor.web3.PublicKey("AbLwGR8A1wvsiLWrzzA5eYPoQw51NVMcMMTPvAv5LTJ");
    const [ledger] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("Ledger"), authority.toBuffer()], 
      program.programId
      );
    const vault = anchor.utils.token.associatedAddress({mint, owner: ledger});
    
    const toOwner = new anchor.web3.PublicKey("8ctxLVXqJjttevpURSnrX5DDMuSgNDAyVjwHoSfccrTE");
    const to = anchor.utils.token.associatedAddress({mint, owner: toOwner})
    const amount = new anchor.BN("1").mul(new anchor.BN(anchor.web3.LAMPORTS_PER_SOL));
    
    const ix = await program.methods
      .splTransfer(amount)
      .accounts({
        associatedTokenProgram,
        authority,
        ledger,
        mint,
        systemProgram,
        to,
        tokenProgram,
        toOwner,
        vault
      }).instruction();

      const tx = new anchor.web3.Transaction();
      
      let MAX_IX_COUNT = 28
      for (let i = 0; i < MAX_IX_COUNT; i++) {
        tx.add(ix);
      }

      const {blockhash, lastValidBlockHeight} = await connection.getLatestBlockhash();
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = blockhash;
      tx.lastValidBlockHeight = lastValidBlockHeight;
      
      const signed = await wallet.signTransaction(tx);
      console.log("ixn count", tx.instructions.length);

      try {
        const signature = await connection.sendRawTransaction(signed.serialize());
        await connection.confirmTransaction({blockhash, lastValidBlockHeight, signature});

        console.log("signature", signature);
      } catch (e) {
        if (e instanceof anchor.web3.SendTransactionError) {
          console.log(e.logs);
        }
        throw e;
      }
  })
});
