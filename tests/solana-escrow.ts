import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { Program } from '@project-serum/anchor';
import { SolanaEscrow } from '../target/types/solana_escrow';
import * as assert from 'assert';
import { NodeWallet } from '@project-serum/anchor/dist/cjs/provider';

/*
QUESTIONS
- Why have initialization code before `before`?
- Why have decimals places set to 0?
- Why does using `program.provider.wallet.publicKey` not require a signature?
*/

/*
What's missing that I won't add in this program
- Test cases like each anchor constraint from each instruction's Context
- Wrong token program, rent, or system program ID passed in
*/

describe('solana-escrow', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaEscrow as Program<SolanaEscrow>;

  // Why not have these variables declared in `before`? -> So we can access them in `it` functions
  let makerMint: spl.Token;
  let takerMint: spl.Token;
  let randomOtherMint: spl.Token;
  let offerMakersMakerTokens: anchor.web3.PublicKey;
  let offerMakersTakerTokens: anchor.web3.PublicKey;
  let offerTakersMakerTokens: anchor.web3.PublicKey;
  let offerTakersTakerTokens: anchor.web3.PublicKey;
  let offerTakersRandomOtherTokens: anchor.web3.PublicKey;
  let hackersTakerTokens: anchor.web3.PublicKey;
  const offerTaker = anchor.web3.Keypair.generate();
  const hacker = anchor.web3.Keypair.generate();

  // We don't use beforeEach because it would be useless. The state is saved in our Solana network, and beforeEach wouldn't reset that
  before(async () => {
    const wallet = program.provider.wallet as NodeWallet;
    makerMint = await spl.Token.createMint(
      program.provider.connection,
      wallet.payer,  // the payer. will pay txn + rent fees
      wallet.publicKey,  // authority over token given to this address
      wallet.publicKey,  // freeze authority (i think)
      0,  // most mints have 9 decimal places
      spl.TOKEN_PROGRAM_ID,
    );
    takerMint = await spl.Token.createMint(
      program.provider.connection,
      wallet.payer,
      wallet.publicKey,
      wallet.publicKey,
      0,
      spl.TOKEN_PROGRAM_ID
    );
    randomOtherMint = await spl.Token.createMint(
      program.provider.connection,
      wallet.payer,
      wallet.publicKey,
      wallet.publicKey,
      0,
      spl.TOKEN_PROGRAM_ID
    );
    offerMakersMakerTokens = await makerMint.createAssociatedTokenAccount(
      program.provider.wallet.publicKey
    );
    offerMakersTakerTokens = await takerMint.createAssociatedTokenAccount(
      program.provider.wallet.publicKey
    );
    offerTakersMakerTokens = await makerMint.createAssociatedTokenAccount(
      offerTaker.publicKey
    );
    offerTakersTakerTokens = await takerMint.createAssociatedTokenAccount(
      offerTaker.publicKey
    );
    offerTakersRandomOtherTokens = await randomOtherMint.createAssociatedTokenAccount(
      offerTaker.publicKey
    );
    hackersTakerTokens = await takerMint.createAssociatedTokenAccount(
      hacker.publicKey
    );

    await makerMint.mintTo(  // txn fee paid by makerMint's payer
      offerMakersMakerTokens, 
      program.provider.wallet.publicKey, 
      [],  // no signer needed because it's program.provider.wallet
      1000
    );
    await takerMint.mintTo(offerTakersTakerTokens, program.provider.wallet.publicKey, [], 1000);
  })

  it('lets you make and accept offers', async () => {
    const offer = anchor.web3.Keypair.generate();

    const [escrowedMakerTokens, escrowedMakerTokensBump] = await anchor.web3.PublicKey.findProgramAddress(
      [offer.publicKey.toBuffer()],
      program.programId
    );

    const txn = await program.rpc.make(
      escrowedMakerTokensBump,
      new anchor.BN(100),
      new anchor.BN(200),
      {
        accounts: {
          offer: offer.publicKey,
          offerMaker: program.provider.wallet.publicKey,
          offerMakersMakerTokens: offerMakersMakerTokens,
          escrowedMakerTokens: escrowedMakerTokens,
          makerMint: makerMint.publicKey,
          takerMint: takerMint.publicKey,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        // (Almost) any time you create an account, the address needs to sign.
        // We are creating escrowedMakerTokens, so the owner (PDA) needs to sign as well.
        // We don't need to add the PDA as a signer because PDA can't sign from client (because they're not actually public keys). 
        // Solana will realize it's a PDA of the program.
        signers: [offer],  // would need to add offer_maker but it's program.provider.wallet so we don't need to
      }
    );

    // We want to fetch lamport balance, so we can't use the higher-level `makerMint.getAccountInfo` for that.
    // We need to use the lower-level getAccountInfo, because that will tell us the lamport balance
    // Note: lamports and solana balance are not necessarily the same: https://discord.com/channels/428295358100013066/517163444747894795/918629680691675176
    const escrowedMakerTokensAccountInfo = await program.provider.connection.getAccountInfo(
      escrowedMakerTokens
    );
    const minRent = await provider.connection.getMinimumBalanceForRentExemption(
      escrowedMakerTokensAccountInfo.data.length
    )
    assert.equal(escrowedMakerTokensAccountInfo.lamports, minRent)

    // assert.equal(100, (await makerMint.getAccountInfo(escrowedMakerTokens)).amount.toNumber());

    await program.rpc.accept({
      // TODO
    })
  });
});
