import { BN } from 'bn.js';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { BatchTransfer } from '../target/types/batch_transfer';

describe("batch-transfer", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.BatchTransfer as Program<BatchTransfer>;

  it("deposit token!", async () => {
    // Add your test here.
    const associatedTokenProgram = anchor.utils.token.ASSOCIATED_PROGRAM_ID;
    const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
    const systemProgram = anchor.web3.SystemProgram.programId;

    const authority = provider.publicKey;
    const mint = new anchor.web3.PublicKey("AbLwGR8A1wvsiLWrzzA5eYPoQw51NVMcMMTPvAv5LTJ");
    const from = anchor.utils.token.associatedAddress({mint, owner: authority})
    const [ledger] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("Ledger"), authority.toBuffer()], 
      program.programId
      );
    const vault = anchor.utils.token.associatedAddress({mint, owner: ledger});
    const amount = new BN("1000").mul(new BN("1000000000"));
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
    }).rpc();
    console.log("Your transaction signature", tx);
  });
});
