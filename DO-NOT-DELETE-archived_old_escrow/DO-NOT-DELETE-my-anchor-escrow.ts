import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { MyAnchorEscrow } from '../target/types/my_anchor_escrow';


describe('my-anchor-escrow', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyAnchorEscrow as anchor.Program<MyAnchorEscrow>;

  const initiator = anchor.web3.Keypair.generate();
  const taker = anchor.web3.Keypair.generate();

  let initiatorDepositAccount = null; // mintA
  let initiatorReceiveAccount = null; // mintB
  let takerDepositAccount = null;
  let takerReceiveAccount = null;

  const MINTA_REQUIRED = 10000; // amount of mintA that taker wants
  const MINTB_REQUIRED = 50;    // amount of mintB that initiator wants
  
  const mintMaker = anchor.web3.Keypair.generate(); // creator of mintA and mintB. Will pay fees
  const mintAuthority = anchor.web3.Keypair.generate(); // creator of mintAuthority
  let mintA = null;
  let mintB = null;

  const escrowAccount = anchor.web3.Keypair.generate()

  it('Setup: Airdrop', async () => {
    // Airdrop sol funds to initiator and taker
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(initiator.publicKey, 10000000000),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(taker.publicKey, 10000000000),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(mintMaker.publicKey, 10000000000),
      "confirmed"
    );
  })

  it('Setup: Mint', async () => {
    // Create mint tokens (a and b): https://www.quicknode.com/guides/web3-sdks/how-to-mint-an-nft-on-solana
    mintA = await Token.createMint(
      provider.connection,
      mintMaker, // the payer. will pay txn + rent fees
      mintAuthority.publicKey, // authority over token given to this address
      null,
      9, // most mints have 9 decimal places
      TOKEN_PROGRAM_ID,
    );

    mintB = await Token.createMint(
      provider.connection,
      mintMaker,
      mintAuthority.publicKey,
      null,
      9,
      TOKEN_PROGRAM_ID,
    );
  })

  it('Setup: Create Token Accounts', async () => {
    initiatorDepositAccount = await mintA.createAssociatedTokenAccount(
      program.provider.wallet.publicKey, // initiator.publicKey,
    );
    takerReceiveAccount = await mintA.createAssociatedTokenAccount(
      taker.publicKey,
    );
    initiatorReceiveAccount = await mintB.createAssociatedTokenAccount(
      initiator.publicKey,
    );
    takerDepositAccount = await mintB.createAssociatedTokenAccount(
      taker.publicKey,
    );
  });

  it('Setup: Fund Token Accounts', async () => {
    // Mint mintA to initiator and mintB to taker. Doesn't matter if they're associated or not
    await mintA.mintTo(  // txn fee paid by mintMaker
      initiatorDepositAccount,
      mintAuthority.publicKey,
      [mintAuthority],  // signer
      MINTA_REQUIRED,
    );
    await mintB.mintTo(  // txn fee paid by mintMaker
      takerDepositAccount,
      mintAuthority.publicKey,
      [mintAuthority],  // signer
      MINTB_REQUIRED,
    );      
  })

  it('Initialization: Normal case', async () => {
    const [vault_account_pda, vault_account_bump] = await PublicKey.findProgramAddress(
      [escrowAccount.publicKey.toBuffer()],
      program.programId
    );
    
    const tx = await program.rpc.initialize(
      vault_account_bump, 
      new anchor.BN(10),
      new anchor.BN(20),
      {
        accounts: {
          initiator: program.provider.wallet.publicKey, // initiator.publicKey, 
          initiatorTokenAccount: initiatorDepositAccount,
          mintA: mintA.publicKey,
          mintB: mintB.publicKey,
          escrowAccount: escrowAccount.publicKey,
          // vaultAccount: vault_account_pda,  // already a PDA, so we don't need to do `*.publicKey`
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        // (Almost) any time you create an account, the address needs to sign.
        // We are creating vault_account, so the owner (PDA) needs to sign as well.
        // We don't add it as a signer because PDA can't sign from client (because they're not actually public keys)
        signers: [escrowAccount],
      }
    );
    
    console.log("Your transaction signature: ", tx);
  });
});
